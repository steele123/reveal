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

/// The folder holding reveal.log, once logging has started.
pub fn log_directory() -> Option<PathBuf> {
    LOG_PATH.get()?.parent().map(PathBuf::from)
}

pub fn write(level: &str, message: fmt::Arguments<'_>) {
    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    let line = format!("{timestamp} [{level}] {}", redact(&message.to_string()));

    if let Some(file) = LOG_FILE.get() {
        let mut file = file.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let _ = writeln!(file, "{line}");
        let _ = file.flush();
    }

    #[cfg(debug_assertions)]
    eprintln!("{line}");
}

/// Keeps logs shareable: hides player account IDs (puuids, which appear in League Client
/// request URLs) and the Windows user folder, which often contains a real name.
fn redact(message: &str) -> String {
    let message = match std::env::var("USERPROFILE") {
        Ok(profile) if !profile.is_empty() => message.replace(&profile, "%USERPROFILE%"),
        _ => message.to_string(),
    };
    redact_uuids(&message)
}

fn redact_uuids(message: &str) -> String {
    const UUID_LENGTH: usize = 36;
    let bytes = message.as_bytes();
    let mut result = String::with_capacity(message.len());
    let mut start = 0;
    let mut index = 0;
    while index + UUID_LENGTH <= bytes.len() {
        if is_uuid(&bytes[index..index + UUID_LENGTH]) {
            result.push_str(&message[start..index]);
            result.push_str("<id>");
            index += UUID_LENGTH;
            start = index;
        } else {
            index += 1;
        }
    }
    result.push_str(&message[start..]);
    result
}

/// "8-4-4-4-12" hex digits, e.g. 0b1f7c2e-1234-5678-9abc-def012345678.
fn is_uuid(bytes: &[u8]) -> bool {
    bytes.iter().enumerate().all(|(index, byte)| match index {
        8 | 13 | 18 | 23 => *byte == b'-',
        _ => byte.is_ascii_hexdigit(),
    })
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

#[cfg(test)]
mod tests {
    use super::redact_uuids;

    #[test]
    fn hides_account_ids_in_request_urls() {
        assert_eq!(
            redact_uuids(
                "Match history lookup failed: error sending request for url (https://127.0.0.1:62467/lol-match-history/v1/products/lol/0b1f7c2e-1234-5678-9abc-DEF012345678/matches?begIndex=0)"
            ),
            "Match history lookup failed: error sending request for url (https://127.0.0.1:62467/lol-match-history/v1/products/lol/<id>/matches?begIndex=0)"
        );
    }

    #[test]
    fn leaves_other_text_alone() {
        for message in [
            "Champ Select participant count increased to 5 from 4",
            "0b1f7c2e-1234-5678-9abc",
            "not-a-uuid-at-all-but-thirty-six-chars",
            "Prénom 🎮 0b1f7c2e_1234_5678_9abc_def012345678",
        ] {
            assert_eq!(redact_uuids(message), message);
        }
    }
}
