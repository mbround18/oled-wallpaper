use std::fmt::{Display, Formatter};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

const LOCK_FILE: &str = "wallpaper.lock";
const AUTOSTART_FILE: &str = "ninja.boop.OledWallpaper.desktop";
const AUTOSTART_DESKTOP: &str = r#"[Desktop Entry]
Type=Application
Name=OLED Wallpaper
Comment=Start OLED Wallpaper as the desktop background at login
Exec=oled-wallpaper
Terminal=false
X-GNOME-Autostart-enabled=true
NoDisplay=false
StartupNotify=false
Categories=Utility;Graphics;
"#;

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
        fs::write(path, AUTOSTART_DESKTOP)?;
    } else if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}
