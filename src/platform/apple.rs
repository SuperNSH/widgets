//! iOS and macOS: the store is in the App Group container both the app and the widget extension can
//! open; WidgetCenter redraws; a Darwin notification from the widget wakes the app.

use std::ffi::{CStr, CString, c_char, c_void};

use crate::WidgetError;

unsafe extern "C" {
    fn nsh_widgets_reload_kind(kind: *const c_char);
    fn nsh_widgets_container_path(group: *const c_char) -> *mut c_char;
    fn nsh_widgets_info_group() -> *mut c_char;
    fn nsh_widgets_observe(name: *const c_char);
    fn nsh_widgets_change_count() -> isize;
    fn nsh_widgets_activity_start(kind: *const c_char, data: *const c_char) -> i32;
    fn nsh_widgets_activity_update(kind: *const c_char, data: *const c_char) -> i32;
    fn nsh_widgets_activity_end(kind: *const c_char) -> i32;
    fn free(p: *mut c_void);
}

/// Takes ownership of a string the bridge `strdup`ed.
fn owned(p: *mut c_char) -> Option<String> {
    if p.is_null() {
        return None;
    }
    // SAFETY: the bridge returns a NUL-terminated strdup result, freed here once.
    let s = unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned();
    unsafe { free(p.cast()) };
    Some(s)
}

fn resolve(group: Option<String>) -> Result<String, WidgetError> {
    // SAFETY: no arguments; the result is owned.
    group
        .or_else(|| owned(unsafe { nsh_widgets_info_group() }))
        .ok_or(WidgetError::NoGroupConfigured)
}

pub(crate) fn root(group: Option<String>) -> Result<String, WidgetError> {
    let group = resolve(group)?;
    let c = CString::new(group.clone()).map_err(|_| WidgetError::NoAppGroup(group.clone()))?;
    // SAFETY: c lives across the call; the result is owned.
    owned(unsafe { nsh_widgets_container_path(c.as_ptr()) }).ok_or(WidgetError::NoAppGroup(group))
}

pub(crate) fn reload(kind: &str) -> Result<(), WidgetError> {
    let c = CString::new(kind).map_err(|_| WidgetError::InvalidKind(kind.to_owned()))?;
    // SAFETY: c lives across the call.
    unsafe { nsh_widgets_reload_kind(c.as_ptr()) };
    Ok(())
}

/// Starts counting the widget's "<group>.nsh.widgets.action" Darwin notifications.
pub(crate) fn observe(group: Option<String>) {
    if let Ok(group) = resolve(group)
        && let Ok(name) = CString::new(format!("{group}.nsh.widgets.action"))
    {
        // SAFETY: name lives across the call; the bridge copies it.
        unsafe { nsh_widgets_observe(name.as_ptr()) };
    }
}

pub(crate) fn change_count() -> i64 {
    // SAFETY: no arguments.
    (unsafe { nsh_widgets_change_count() }) as i64
}

fn activity_result(code: i32, kind: &str) -> Result<(), WidgetError> {
    match code {
        0 => Ok(()),
        2 => Err(WidgetError::ActivitiesDisabled),
        3 => Err(WidgetError::ActivityRefused),
        4 => Err(WidgetError::NoActivity(kind.to_owned())),
        _ => Err(WidgetError::Unsupported),
    }
}

fn c(s: &str) -> Result<CString, WidgetError> {
    CString::new(s).map_err(|_| WidgetError::InvalidJson("a NUL byte in the activity's data".into()))
}

pub(crate) fn activity_start(kind: &str, data: &str) -> Result<(), WidgetError> {
    let (k, d) = (c(kind)?, c(data)?);
    // SAFETY: both strings live across the call; the bridge copies them.
    activity_result(unsafe { nsh_widgets_activity_start(k.as_ptr(), d.as_ptr()) }, kind)
}

pub(crate) fn activity_update(kind: &str, data: &str) -> Result<(), WidgetError> {
    let (k, d) = (c(kind)?, c(data)?);
    // SAFETY: as above.
    activity_result(unsafe { nsh_widgets_activity_update(k.as_ptr(), d.as_ptr()) }, kind)
}

pub(crate) fn activity_end(kind: &str) -> Result<(), WidgetError> {
    let k = c(kind)?;
    // SAFETY: as above.
    activity_result(unsafe { nsh_widgets_activity_end(k.as_ptr()) }, kind)
}
