use anyhow::{Context, Result};
use chrono::{SecondsFormat, Utc};
use std::{
    fmt,
    fs::{self, File, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

const MAX_LOG_BYTES: u64 = 2 * 1024 * 1024;
const LOG_DIRECTORY_NAME: &str = "logs";
const LOG_FILE_NAME: &str = "reveal.log";
const PREVIOUS_LOG_FILE_NAME: &str = "reveal.previous.log";

static LOG_FILE: OnceLock<Mutex<File>> = OnceLock::new();
static LOG_PATH: OnceLock<PathBuf> = OnceLock::new();

pub fn init() -> Result<PathBuf> {
    if let Some(path) = LOG_PATH.get() {
        return Ok(path.clone());
    }

    let app_data = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let log_directory = app_data.join("Reveal").join(LOG_DIRECTORY_NAME);
    fs::create_dir_all(&log_directory).with_context(|| {
        format!(
            "failed to create log directory at {}",
            log_directory.display()
        )
    })?;

    let log_path = log_directory.join(LOG_FILE_NAME);
    rotate_if_needed(&log_path)?;

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .with_context(|| format!("failed to open log file at {}", log_path.display()))?;

    let _ = LOG_FILE.set(Mutex::new(file));
    let _ = LOG_PATH.set(log_path.clone());
    install_panic_hook();

    Ok(log_path)
}

fn rotate_if_needed(log_path: &PathBuf) -> Result<()> {
    let Ok(metadata) = fs::metadata(log_path) else {
        return Ok(());
    };
    if metadata.len() < MAX_LOG_BYTES {
        return Ok(());
    }

    let previous_path = log_path.with_file_name(PREVIOUS_LOG_FILE_NAME);
    if previous_path.exists() {
        fs::remove_file(&previous_path).with_context(|| {
            format!(
                "failed to remove previous log file at {}",
                previous_path.display()
            )
        })?;
    }
    fs::rename(log_path, &previous_path).with_context(|| {
        format!(
            "failed to rotate log file from {} to {}",
            log_path.display(),
            previous_path.display()
        )
    })?;

    Ok(())
}

fn install_panic_hook() {
    #[cfg(debug_assertions)]
    let previous_hook = std::panic::take_hook();
    #[cfg(not(debug_assertions))]
    let _ = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        write("ERROR", format_args!("Unhandled panic: {panic_info}"));

        #[cfg(debug_assertions)]
        previous_hook(panic_info);
    }));
}

pub fn write(level: &str, message: fmt::Arguments<'_>) {
    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let line = format!("{timestamp} [{level}] {message}");

    if let Some(file) = LOG_FILE.get() {
        let mut file = file.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let _ = writeln!(file, "{line}");
        let _ = file.flush();
    }

    #[cfg(debug_assertions)]
    eprintln!("{line}");
}

pub fn sanitize_frontend_message(message: &str) -> String {
    message
        .replace(['\r', '\n'], " ")
        .chars()
        .take(2_000)
        .collect()
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::logging::write("INFO", format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::logging::write("WARN", format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::logging::write("ERROR", format_args!($($arg)*))
    };
}
