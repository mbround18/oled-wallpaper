use std::fmt::{Display, Formatter};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};

const LOCK_FILE: &str = "wallpaper.lock";
const RESTART_SIGNAL_FILE: &str = "restart.signal";
const AUTOSTART_FILE: &str = "ninja.boop.OledWallpaper.desktop";
const FLATPAK_APP_ID: &str = "ninja.boop.OledWallpaper";

// ─── Execution context detection ─────────────────────────────────────────────

/// True if this process is currently running *inside* the Flatpak sandbox.
pub fn is_flatpak() -> bool {
    std::env::var("FLATPAK_ID")
        .map(|id| id == FLATPAK_APP_ID)
        .unwrap_or(false)
        || Path::new("/.flatpak-info").exists()
}

/// True if the Flatpak app is installed in the user or system installation.
/// This works whether or not we are currently running inside the sandbox.
pub fn flatpak_app_installed() -> bool {
    which_ok("flatpak")
        && (
            // user install (preferred)
            std::process::Command::new("flatpak")
                .args(["--user", "info", FLATPAK_APP_ID])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
            ||
            // system-wide install fallback
            std::process::Command::new("flatpak")
                .args(["info", FLATPAK_APP_ID])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        )
}

/// The `Exec=` command that will actually launch the wallpaper at login.
/// Prefers Flatpak if installed; falls back to native binary path.
///
/// `is_flatpak()` is checked first: when this process is itself running
/// inside the Flatpak sandbox, the `flatpak` CLI binary isn't present there
/// (sandboxed apps don't get host tools), so `flatpak_app_installed()`'s
/// subprocess check always fails and would otherwise fall through to
/// `/app/bin/oled-wallpaper` — a path that only exists inside the sandbox's
/// own mount namespace, not on the host. `is_flatpak()` needs no subprocess
/// and is reliable from inside the sandbox, so it takes priority here.
pub fn autostart_exec() -> String {
    if is_flatpak() || flatpak_app_installed() {
        format!("flatpak run {FLATPAK_APP_ID}")
    } else {
        // Try to resolve the wallpaper binary alongside the current executable
        std::env::current_exe()
            .ok()
            .and_then(|p| {
                let wallpaper = p.with_file_name("oled-wallpaper");
                if wallpaper.exists() {
                    Some(wallpaper.to_string_lossy().into_owned())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "oled-wallpaper".to_string())
    }
}

fn autostart_desktop_content() -> String {
    let exec = autostart_exec();
    format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=OLED Wallpaper\n\
         Comment=Start OLED Wallpaper as the desktop background at login\n\
         Exec={exec}\n\
         Terminal=false\n\
         X-GNOME-Autostart-enabled=true\n\
         NoDisplay=false\n\
         StartupNotify=false\n\
         Categories=Utility;Graphics;\n"
    )
}

// ─── Autostart verification ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AutostartInfo {
    pub path: PathBuf,
    pub exec_line: String, // what *should* be in the file
    pub file_exists: bool,
    pub exec_reachable: bool,
    pub exec_stale: bool, // file exists but has an old/different Exec= line
    pub via_flatpak: bool,
}

pub fn autostart_info() -> AutostartInfo {
    // See autostart_exec()'s doc comment: flatpak_app_installed() alone can't
    // detect our own install from inside our own sandbox (no `flatpak` CLI
    // there), which would otherwise show a false "unreachable" warning.
    let via_flatpak = is_flatpak() || flatpak_app_installed();
    let exec_line = autostart_exec();
    let path = autostart_path();
    let file_exists = path.exists();

    // Stale = file exists but doesn't contain the correct Exec= line
    let exec_stale = if file_exists {
        fs::read_to_string(&path)
            .map(|c| !c.lines().any(|l| l.trim() == format!("Exec={exec_line}")))
            .unwrap_or(false)
    } else {
        false
    };

    let exec_reachable = if via_flatpak {
        true // flatpak_app_installed() already confirmed this
    } else {
        let bin = exec_line.trim();
        if bin.starts_with('/') {
            Path::new(bin).exists()
        } else {
            which_ok(bin)
        }
    };

    AutostartInfo {
        path,
        exec_line,
        file_exists,
        exec_reachable,
        exec_stale,
        via_flatpak,
    }
}

/// If an autostart file exists with a stale Exec= line, silently rewrite it.
/// Returns true if it was updated.
pub fn heal_autostart_if_stale() -> bool {
    let info = autostart_info();
    if info.file_exists && info.exec_stale && set_autostart_enabled(true).is_ok() {
        tracing::info!("Autostart healed: Exec={}", info.exec_line);
        return true;
    }
    false
}

fn which_ok(cmd: &str) -> bool {
    std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|dir| dir.join(cmd).exists()))
        .unwrap_or(false)
}

#[derive(Debug, Clone, Copy)]
pub struct WallpaperStatus {
    pub running: bool,
    pub pid: Option<u32>,
}

#[derive(Debug)]
pub enum WallpaperLockError {
    AlreadyRunning { pid: Option<u32> },
    Io(std::io::Error),
}

impl Display for WallpaperLockError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyRunning { pid: Some(pid) } => {
                write!(f, "wallpaper is already running (pid {pid})")
            }
            Self::AlreadyRunning { pid: None } => write!(f, "wallpaper is already running"),
            Self::Io(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for WallpaperLockError {}

impl From<std::io::Error> for WallpaperLockError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

/// Holds the open lock file (and thus its `flock`) for the process's lifetime.
/// The OS releases the flock automatically when this file descriptor closes —
/// on a clean drop, on any signal-based termination (SIGTERM/SIGKILL don't run
/// Rust destructors, but the kernel closes fds and releases flocks regardless),
/// and even across a Flatpak sandbox boundary, since flock has no concept of
/// PIDs or PID namespaces at all. See the module-level doc comment on
/// `acquire_wallpaper_lock` for why this replaced PID-based locking.
pub struct WallpaperInstanceGuard {
    _file: File,
}

fn runtime_dir() -> PathBuf {
    crate::config::Config::config_dir()
}

fn lock_path() -> PathBuf {
    runtime_dir().join(LOCK_FILE)
}

fn restart_signal_path() -> PathBuf {
    runtime_dir().join(RESTART_SIGNAL_FILE)
}

/// Try to take a non-blocking exclusive flock on `file`. `Ok(true)` = we hold
/// it now; `Ok(false)` = someone else holds it; `Err` = unexpected OS error.
fn try_flock(file: &File) -> Result<bool, std::io::Error> {
    let rc = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if rc == 0 {
        Ok(true)
    } else {
        let err = std::io::Error::last_os_error();
        if err.kind() == std::io::ErrorKind::WouldBlock {
            Ok(false)
        } else {
            Err(err)
        }
    }
}

fn unflock(file: &File) {
    unsafe {
        libc::flock(file.as_raw_fd(), libc::LOCK_UN);
    }
}

fn autostart_path() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".config")
        .join("autostart")
        .join(AUTOSTART_FILE)
}

fn read_lock_pid(path: &Path) -> Option<u32> {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok())
}

/// Acquire the single-instance lock via advisory `flock`, not a PID stored in
/// the file.
///
/// This used to store the current PID in the lock file and have later callers
/// check `/proc/{pid}` to decide if the old instance was still alive. That is
/// meaningless under Flatpak: every `flatpak run` gets its own private PID
/// namespace, and the launched process almost always lands on PID 2 there. A
/// wallpaper started via the autostart entry's `flatpak run` would write "2"
/// to the lock file; the Configurator, running in a *different* sandbox
/// instance (or natively), would then check `/proc/2` in *its own* namespace
/// — where PID 2 is some unrelated process (on a bare host, the kthreadd
/// kernel thread, which is always alive). That check always returned "yes,
/// still running," permanently, regardless of whether the real wallpaper
/// process was alive at all — the exact "auto locked, never starts again"
/// bug this replaced. `flock` has no notion of PIDs or namespaces: it's held
/// on the shared lock file's inode itself (visible to every sandbox instance
/// via the existing `~/.config/oled-wallpaper` filesystem permission) and is
/// released by the kernel the moment the holding file descriptor closes, for
/// any reason — clean exit, SIGTERM, SIGKILL, or a crash — with no signal
/// handler or cleanup code required.
pub fn acquire_wallpaper_lock() -> Result<WallpaperInstanceGuard, WallpaperLockError> {
    fs::create_dir_all(runtime_dir())?;
    let path = lock_path();
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&path)?;

    if !try_flock(&file)? {
        // Best-effort only: the PID in the file may belong to a different
        // sandbox instance's namespace and not resolve to anything meaningful
        // here. It's kept purely as a diagnostic hint for error messages/UI.
        let pid = read_lock_pid(&path);
        return Err(WallpaperLockError::AlreadyRunning { pid });
    }

    // We hold the flock now. Stamp our own PID for display purposes only.
    file.set_len(0)?;
    {
        let mut f = &file;
        f.write_all(format!("{}\n", std::process::id()).as_bytes())?;
        f.flush()?;
    }
    Ok(WallpaperInstanceGuard { _file: file })
}

pub fn wallpaper_status() -> WallpaperStatus {
    let path = lock_path();
    let file = match OpenOptions::new().read(true).write(true).open(&path) {
        Ok(f) => f,
        Err(_) => {
            return WallpaperStatus {
                running: false,
                pid: None,
            }
        }
    };

    match try_flock(&file) {
        Ok(true) => {
            // We just took the lock ourselves - nobody else holds it. Release
            // it immediately since we're only checking status, not running.
            unflock(&file);
            WallpaperStatus {
                running: false,
                pid: None,
            }
        }
        Ok(false) => WallpaperStatus {
            running: true,
            pid: read_lock_pid(&path),
        },
        Err(_) => WallpaperStatus {
            running: false,
            pid: None,
        },
    }
}

/// Ask a running wallpaper instance to exit on its own (its render loop polls
/// for this each frame). Cross-sandbox `kill -TERM <pid>` can't work here for
/// the same reason PID-based lock checking couldn't: the PID is only
/// meaningful inside its own sandbox's PID namespace. This is a cooperative
/// signal written to the same shared, bind-mounted config directory instead.
pub fn request_restart() -> Result<(), std::io::Error> {
    fs::write(restart_signal_path(), "")
}

/// Checked by the wallpaper's render loop once per frame.
pub fn restart_requested() -> bool {
    restart_signal_path().exists()
}

/// Consume the restart request so it doesn't immediately refire.
pub fn clear_restart_request() {
    let _ = fs::remove_file(restart_signal_path());
}

pub fn autostart_enabled() -> bool {
    autostart_path().exists()
}

pub fn set_autostart_enabled(enabled: bool) -> Result<(), std::io::Error> {
    let path = autostart_path();
    if enabled {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, autostart_desktop_content())?;
    } else if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

// ─── Wallpaper restart ────────────────────────────────────────────────────────

/// Ask the running wallpaper (if any) to exit, wait for it to actually
/// release its lock, then relaunch it. Returns an error string if the
/// existing instance didn't exit in time or the relaunch failed.
///
/// This no longer kills by PID (see `acquire_wallpaper_lock`'s doc comment
/// for why that's unreliable, sometimes permanently, under Flatpak).
/// Instead it writes a cooperative restart-request file the wallpaper's
/// render loop polls each frame, then waits for `wallpaper_status()` to
/// confirm the flock was actually released before spawning a new instance -
/// spawning too early would just make the new instance fail to acquire the
/// lock itself.
pub fn restart_wallpaper() -> Result<(), String> {
    if wallpaper_status().running {
        request_restart().map_err(|e| format!("Failed to signal restart: {e}"))?;

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            if !wallpaper_status().running {
                break;
            }
            if std::time::Instant::now() >= deadline {
                clear_restart_request();
                return Err("Timed out waiting for the running wallpaper to exit".to_string());
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    // Resolve the wallpaper binary the same way regardless of Flatpak or
    // native: try a sibling of the current executable first (covers both
    // `/app/bin/oled-wallpaper` next to `/app/bin/oled-config` inside the
    // sandbox, and a native sibling install), else fall back to bare PATH
    // lookup (covers /app/bin being on PATH inside the sandbox too). This
    // deliberately does NOT shell out to `flatpak run`: the `flatpak` CLI
    // binary isn't present inside the sandbox itself, so that would always
    // fail to spawn when called from a running, sandboxed oled-config.
    let bin = std::env::current_exe()
        .ok()
        .and_then(|p| {
            let sibling = p.with_file_name("oled-wallpaper");
            sibling.exists().then_some(sibling)
        })
        .unwrap_or_else(|| PathBuf::from("oled-wallpaper"));

    std::process::Command::new(bin)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Failed to launch wallpaper: {e}"))
}
