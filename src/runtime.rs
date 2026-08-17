use std::fmt::{Display, Formatter};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

const LOCK_FILE: &str = "wallpaper.lock";
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
pub fn autostart_exec() -> String {
    if flatpak_app_installed() {
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
    pub exec_line: String,
    pub file_exists: bool,
    /// Whether the exec command/binary is actually reachable right now.
    pub exec_reachable: bool,
    pub via_flatpak: bool,
}

pub fn autostart_info() -> AutostartInfo {
    let via_flatpak = flatpak_app_installed();
    let exec_line = autostart_exec();
    let path = autostart_path();
    let file_exists = path.exists();

    let exec_reachable = if via_flatpak {
        // flatpak_app_installed() already confirmed this
        true
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
        via_flatpak,
    }
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

pub struct WallpaperInstanceGuard {
    lock_path: PathBuf,
}

impl Drop for WallpaperInstanceGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.lock_path);
    }
}

fn runtime_dir() -> PathBuf {
    crate::config::Config::config_dir()
}

fn lock_path() -> PathBuf {
    runtime_dir().join(LOCK_FILE)
}

fn autostart_path() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".config")
        .join("autostart")
        .join(AUTOSTART_FILE)
}

fn pid_alive(pid: u32) -> bool {
    Path::new(&format!("/proc/{pid}")).exists()
}

fn read_lock_pid(path: &Path) -> Option<u32> {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok())
}

fn write_lock(path: &Path) -> Result<(), std::io::Error> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    writeln!(file, "{}", std::process::id())?;
    file.flush()?;
    Ok(())
}

pub fn acquire_wallpaper_lock() -> Result<WallpaperInstanceGuard, WallpaperLockError> {
    fs::create_dir_all(runtime_dir())?;
    let path = lock_path();

    for _ in 0..2 {
        match write_lock(&path) {
            Ok(()) => return Ok(WallpaperInstanceGuard { lock_path: path }),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                let existing = read_lock_pid(&path);
                if let Some(pid) = existing {
                    if pid_alive(pid) {
                        return Err(WallpaperLockError::AlreadyRunning { pid: Some(pid) });
                    }
                }
                let _ = fs::remove_file(&path);
            }
            Err(e) => return Err(WallpaperLockError::Io(e)),
        }
    }

    Err(WallpaperLockError::AlreadyRunning { pid: None })
}

pub fn wallpaper_status() -> WallpaperStatus {
    let path = lock_path();
    if !path.exists() {
        return WallpaperStatus {
            running: false,
            pid: None,
        };
    }

    let pid = read_lock_pid(&path);
    if let Some(pid) = pid {
        if pid_alive(pid) {
            return WallpaperStatus {
                running: true,
                pid: Some(pid),
            };
        }
    }

    let _ = fs::remove_file(&path);
    WallpaperStatus {
        running: false,
        pid: None,
    }
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

/// Kill the running wallpaper (if any) then immediately relaunch it.
/// Returns an error string if the relaunch failed.
pub fn restart_wallpaper() -> Result<(), String> {
    // Kill current instance using the PID in the lock file
    let status = wallpaper_status();
    if let Some(pid) = status.pid {
        let _ = std::process::Command::new("kill")
            .arg("-TERM")
            .arg(pid.to_string())
            .status();
        // Give the process a moment to exit and release the lock
        std::thread::sleep(std::time::Duration::from_millis(400));
    }

    // Choose launch command: Flatpak or native sibling binary
    let (cmd, args): (&str, Vec<&str>) = if flatpak_app_installed() {
        ("flatpak", vec!["run", "--command=oled-wallpaper", FLATPAK_APP_ID])
    } else {
        ("oled-wallpaper", vec![])
    };

    std::process::Command::new(cmd)
        .args(&args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Failed to launch wallpaper: {e}"))
}
