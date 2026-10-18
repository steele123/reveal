//! Reveal moved from a per-machine MSI install (Program Files, needs admin) to a per-user NSIS
//! install. The updater installs the NSIS copy alongside the old one, so the old MSI install is
//! offered for removal from the app. Removing it needs one admin prompt, shown by Windows Installer.

use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyInstall {
    pub product_code: String,
    pub version: String,
    pub location: String,
}

const PRODUCT_NAME: &str = "Reveal";

/// Finds an MSI install of Reveal that is not the copy currently running.
#[cfg(windows)]
pub fn find() -> Option<LegacyInstall> {
    use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

    let uninstall = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall")
        .ok()?;
    let current_exe = std::env::current_exe().ok()?;

    uninstall.enum_keys().flatten().find_map(|product_code| {
        let entry = uninstall.open_subkey(&product_code).ok()?;
        let display_name: String = entry.get_value("DisplayName").ok()?;
        let windows_installer: u32 = entry.get_value("WindowsInstaller").unwrap_or(0);
        let location: String = entry.get_value("InstallLocation").unwrap_or_default();
        let install = LegacyInstall {
            version: entry.get_value("DisplayVersion").unwrap_or_default(),
            product_code,
            location,
        };

        is_legacy_install(&display_name, windows_installer, &install, &current_exe)
            .then_some(install)
    })
}

#[cfg(not(windows))]
pub fn find() -> Option<LegacyInstall> {
    None
}

fn is_legacy_install(
    display_name: &str,
    windows_installer: u32,
    install: &LegacyInstall,
    current_exe: &std::path::Path,
) -> bool {
    display_name == PRODUCT_NAME
        && windows_installer == 1
        && install.product_code.starts_with('{')
        && !install.location.is_empty()
        // Never offer to uninstall the copy that is running.
        && !current_exe
            .to_string_lossy()
            .to_lowercase()
            .starts_with(&install.location.to_lowercase())
}

/// Runs the Windows Installer uninstall for the old copy and waits for it to finish.
/// Windows Installer shows the admin prompt; returns false if it was declined or failed.
#[cfg(windows)]
pub fn remove(install: &LegacyInstall) -> anyhow::Result<bool> {
    let status = std::process::Command::new("msiexec.exe")
        .args(["/x", &install.product_code, "/passive", "/norestart"])
        .status()?;
    log_info!("Legacy MSI uninstall exited with {status}");
    Ok(status.success())
}

#[cfg(not(windows))]
pub fn remove(_install: &LegacyInstall) -> anyhow::Result<bool> {
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn install(location: &str) -> LegacyInstall {
        LegacyInstall {
            product_code: "{2291C3A3-4251-4D1D-AD7F-1A45F6089A20}".to_string(),
            version: "1.4.3".to_string(),
            location: location.to_string(),
        }
    }

    #[test]
    fn detects_the_old_program_files_install_from_the_per_user_copy() {
        assert!(is_legacy_install(
            "Reveal",
            1,
            &install(r"C:\Program Files\Reveal\"),
            Path::new(r"C:\Users\me\AppData\Local\Reveal\Reveal.exe"),
        ));
    }

    #[test]
    fn ignores_the_running_copy_and_unrelated_entries() {
        let program_files = install(r"C:\Program Files\Reveal\");
        assert!(!is_legacy_install(
            "Reveal",
            1,
            &program_files,
            Path::new(r"C:\Program Files\Reveal\Reveal.exe"),
        ));
        assert!(!is_legacy_install(
            "Reveal",
            1,
            &program_files,
            Path::new(r"c:\program files\reveal\Reveal.exe"),
        ));
        let per_user_exe = Path::new(r"C:\Users\me\AppData\Local\Reveal\Reveal.exe");
        assert!(!is_legacy_install(
            "Reveal",
            0,
            &program_files,
            per_user_exe
        ));
        assert!(!is_legacy_install(
            "Revealer",
            1,
            &program_files,
            per_user_exe
        ));
        assert!(!is_legacy_install("Reveal", 1, &install(""), per_user_exe));
    }
}
