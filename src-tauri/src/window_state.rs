//! Remembers the window's size and place between runs. The content grows with the window, so
//! this is also what keeps a bigger UI after a restart or an update.

use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};
use tauri::{LogicalSize, Manager, PhysicalPosition, Window};

const FILE_NAME: &str = "window.json";
/// The smallest size the window allows, as in tauri.conf.json.
const MIN_WIDTH: f64 = 600.0;
const MIN_HEIGHT: f64 = 500.0;
/// Resizing and moving send many events; save once they stop.
const SAVE_DELAY: Duration = Duration::from_millis(500);
/// How much of the title bar has to be on a screen for the saved place to be used.
const VISIBLE_GRIP: i32 = 40;

static CHANGES: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WindowState {
    /// The inner size in logical pixels, so it means the same at any display scale.
    width: f64,
    height: f64,
    /// The outer top-left corner in physical pixels, as Windows places windows.
    x: i32,
    y: i32,
    #[serde(default)]
    maximized: bool,
}

/// A screen's area in physical pixels: x, y, width, height.
type Screen = (i32, i32, u32, u32);

/// Puts the window back where it was, at the size it was. Call before showing it.
pub fn restore(window: &Window) {
    let Some(path) = state_path(window) else {
        return;
    };
    let Some(saved) = read(&path) else {
        return;
    };
    let screens: Vec<Screen> = window
        .available_monitors()
        .unwrap_or_default()
        .iter()
        .map(|monitor| {
            let (position, size) = (monitor.position(), monitor.size());
            (position.x, position.y, size.width, size.height)
        })
        .collect();

    let placed = on_a_screen(&saved, &screens);
    if placed {
        if let Err(error) = window.set_position(PhysicalPosition::new(saved.x, saved.y)) {
            log_warn!("Couldn't put the window back where it was: {error}");
        }
    }
    let (width, height) = fit_to_screen(saved, window);
    if let Err(error) = window.set_size(LogicalSize::new(width, height)) {
        log_warn!("Couldn't restore the window's size: {error}");
    }
    if !placed {
        log_info!("The window's saved place is off every screen; centering it");
        let _ = window.center();
    }
    if saved.maximized {
        let _ = window.maximize();
    }
    log_info!("Restored the window at {width:.0}x{height:.0}");
}

/// Saves the window's size and place once it stops changing.
pub fn changed(window: &Window) {
    let change = CHANGES.fetch_add(1, Ordering::AcqRel) + 1;
    let window = window.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(SAVE_DELAY).await;
        if CHANGES.load(Ordering::Acquire) == change {
            save(&window);
        }
    });
}

fn save(window: &Window) {
    // Hidden in the tray, minimized, or still being restored: nothing worth keeping.
    if !window.is_visible().unwrap_or(false) || window.is_minimized().unwrap_or(true) {
        return;
    }
    let Some(path) = state_path(window) else {
        return;
    };
    let maximized = window.is_maximized().unwrap_or(false);
    // A maximized window's size is the screen's; keep the size to go back to instead.
    let state = match (maximized, read(&path)) {
        (true, Some(previous)) => WindowState { maximized: true, ..previous },
        _ => {
            let (Ok(scale), Ok(size), Ok(position)) =
                (window.scale_factor(), window.inner_size(), window.outer_position())
            else {
                return;
            };
            let size = size.to_logical::<f64>(scale);
            WindowState {
                width: size.width,
                height: size.height,
                x: position.x,
                y: position.y,
                maximized,
            }
        }
    };
    if let Err(error) = write(&path, &state) {
        log_warn!("Couldn't save the window's size: {error}");
    }
}

fn state_path(window: &Window) -> Option<PathBuf> {
    let directory = window.app_handle().path_resolver().app_config_dir()?;
    std::fs::create_dir_all(&directory).ok()?;
    Some(directory.join(FILE_NAME))
}

/// The saved state, if there is one that makes sense.
fn read(path: &Path) -> Option<WindowState> {
    let json = std::fs::read_to_string(path).ok()?;
    let state: WindowState = serde_json::from_str(&json).ok()?;
    let sane = |value: f64, min: f64| value.is_finite() && value >= min && value <= 20_000.0;
    (sane(state.width, MIN_WIDTH) && sane(state.height, MIN_HEIGHT)).then_some(state)
}

/// Writes next to the file and swaps it in, so a crash can't leave half a file.
fn write(path: &Path, state: &WindowState) -> std::io::Result<()> {
    let temp = path.with_extension("json.tmp");
    std::fs::write(&temp, serde_json::to_string(state)?)?;
    std::fs::rename(&temp, path)
}

/// Whether enough of the title bar would be on some screen to drag the window by, so a screen
/// that's since been unplugged can't leave it out of reach.
fn on_a_screen(state: &WindowState, screens: &[Screen]) -> bool {
    let (left, top) = (state.x + VISIBLE_GRIP, state.y + VISIBLE_GRIP / 2);
    screens.iter().any(|&(x, y, width, height)| {
        let (right, bottom) = (x + width as i32, y + height as i32);
        left >= x && left < right && top >= y && top < bottom
    })
}

/// The saved size, shrunk to fit the screen the window is on now.
fn fit_to_screen(state: WindowState, window: &Window) -> (f64, f64) {
    let screen = window.current_monitor().ok().flatten().map(|monitor| {
        monitor.size().to_logical::<f64>(monitor.scale_factor())
    });
    match screen {
        Some(screen) => (
            state.width.min(screen.width).max(MIN_WIDTH),
            state.height.min(screen.height).max(MIN_HEIGHT),
        ),
        None => (state.width, state.height),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(x: i32, y: i32) -> WindowState {
        WindowState { width: 900.0, height: 900.0, x, y, maximized: false }
    }

    #[test]
    fn uses_the_saved_place_only_when_the_title_bar_is_on_a_screen() {
        let screens = [(0, 0, 1920, 1080), (1920, 0, 2560, 1440)];
        assert!(on_a_screen(&state(100, 100), &screens));
        assert!(on_a_screen(&state(3000, 200), &screens), "on the second screen");
        assert!(on_a_screen(&state(-30, 0), &screens), "partly off the left edge");
        assert!(!on_a_screen(&state(-2000, 100), &screens), "a screen that's gone");
        assert!(!on_a_screen(&state(500, 1070), &screens), "title bar below the screen");
        assert!(!on_a_screen(&state(100, 100), &[]));
    }

    #[test]
    fn ignores_a_missing_broken_or_tiny_saved_size() {
        let directory = std::env::temp_dir().join(format!("reveal-window-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join(FILE_NAME);

        assert_eq!(read(&path), None);
        std::fs::write(&path, "{\"width\": 9").unwrap();
        assert_eq!(read(&path), None);
        std::fs::write(&path, r#"{"width":100,"height":680,"x":0,"y":0}"#).unwrap();
        assert_eq!(read(&path), None);

        write(&path, &state(10, 20)).unwrap();
        assert_eq!(read(&path), Some(state(10, 20)));
        std::fs::remove_dir_all(&directory).unwrap();
    }
}
