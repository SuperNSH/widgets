//! Linux, Windows and the web have no widget host yet. `NSH_WIDGETS_DIR` points the store at a
//! directory, so the data path can be used and tested there; nothing redraws.

use crate::WidgetError;

pub(crate) fn root(_group: Option<String>) -> Result<String, WidgetError> {
    std::env::var("NSH_WIDGETS_DIR").map_err(|_| WidgetError::Unsupported)
}

pub(crate) fn reload(_kind: &str) -> Result<(), WidgetError> {
    Ok(())
}

pub(crate) fn observe(_group: Option<String>) {}

pub(crate) fn change_count() -> i64 {
    0
}
