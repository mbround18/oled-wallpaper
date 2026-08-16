use std::fmt::{Display, Formatter};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

const LOCK_FILE: &str = "wallpaper.lock";
const AUTOSTART_FILE: &str = "ninja.boop.OledWallpaper.desktop";
const FLATPAK_APP_ID: &str = "ninja.boop.OledWallpaper";

// ─── Execution context detection ─────────────────────────────────────────────

/// Returns true when the process is running inside a Flatpak sandbox.
pub fn is_flatpak() -> bool {
    // FLATPAK_ID is set by the Flatpak runtime for sandboxed apps.
    std::env::var("FLATPAK_ID")
        .map(|id| id == FLATPAK_APP_ID)
        .unwrap_or(false)
        || Path::new("/.flatpak-info").exists()
}

/// The `Exec=` command that will actually launch the wallpaper at login.
pub fn autostart_exec() -> String {
    if is_flatpak() {
        format!("flatpak run {FLATPAK_APP_ID}")
    } else {
        // Use the real binary path so it works regardless of PATH at session start.
        std::env::current_exe()
            .ok()
            .and_then(|p| {
                // current_exe is oled-config — the wallpaper binary lives alongside it
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
    pub is_flatpak: bool,
}

pub fn autostart_info() -> AutostartInfo {
    let path = autostart_path();
    let exec_line = autostart_exec();
    let file_exists = path.exists();

    let exec_reachable = if is_flatpak() {
        // Check flatpak is on PATH and the app is installed
        which_ok("flatpak")
            && std::process::Command::new("flatpak")
                .args(["info", FLATPAK_APP_ID])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
    } else {
        let bin: &str = exec_line.trim();
        // Absolute path check first, then PATH lookup
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
        is_flatpak: is_flatpak(),
    }
}

fn which_ok(cmd: &str) -> bool {
    std::env::var_os("PATH")
        .map(|paths| {
            std::env::split_paths(&paths)
                .any(|dir| dir.join(cmd).exists())
        })
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
