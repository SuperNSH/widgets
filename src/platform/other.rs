//! Linux, Windows and the web: no system API draws these widgets. The store lives in a
//! directory that `nsh-widgets show` reads (for terminals and status bars such as Waybar):
//! `NSH_WIDGETS_DIR` when set, else `<data dir>/nsh-widgets/<group>` once the app calls
//! `configure(group)`. The data dir is `$XDG_DATA_HOME` or `~/.local/share`, `%LOCALAPPDATA%` on Windows.

use std::path::PathBuf;

use crate::WidgetError;

pub(crate) fn root(group: Option<String>) -> Result<String, WidgetError> {
    if let Ok(dir) = std::env::var("NSH_WIDGETS_DIR") {
        return Ok(dir);
    }
    let group = group.ok_or(WidgetError::NoGroupConfigured)?;
    let base = data_home().ok_or(WidgetError::Unsupported)?;
    Ok(base.join("nsh-widgets").join(group).to_string_lossy().into_owned())
}

/// The per-user data directory, as `nsh-widgets show` computes it too.
fn data_home() -> Option<PathBuf> {
    let var = |k: &str| std::env::var_os(k).filter(|v| !v.is_empty()).map(PathBuf::from);
    if cfg!(windows) {
        var("LOCALAPPDATA")
    } else {
        var("XDG_DATA_HOME").or_else(|| var("HOME").map(|h| h.join(".local/share")))
    }
}

pub(crate) fn reload(_kind: &str) -> Result<(), WidgetError> {
    Ok(())
}

pub(crate) fn observe(_group: Option<String>) {}

pub(crate) fn change_count() -> i64 {
    0
}

/// Live Activities exist only on iOS.
pub(crate) fn activity_start(_kind: &str, _data: &str) -> Result<(), WidgetError> {
    Err(WidgetError::Unsupported)
}

pub(crate) fn activity_update(_kind: &str, _data: &str) -> Result<(), WidgetError> {
    Err(WidgetError::Unsupported)
}

pub(crate) fn activity_end(_kind: &str) -> Result<(), WidgetError> {
    Err(WidgetError::Unsupported)
}
