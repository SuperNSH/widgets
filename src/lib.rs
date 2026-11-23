//! Home-screen widgets for Native Shell (NSH) apps.
//!
//! The app describes each widget once, as a tree of primitives (Widget Tree IR v1, written from TSX
//! with the `nsh-widgets` package or from Rust as JSON), and feeds it data. A generic renderer draws the
//! tree natively: SwiftUI in the WidgetKit extension `nsh-widgets build` makes for iOS and macOS, and
//! RemoteViews in the Android provider this crate's AAR carries. Buttons in the widget change the data
//! without the app; the app hears about them through [`take_actions`] and [`next_change`].
//!
//! ```no_run
//! use serde_json::json;
//! nsh_widgets::update_data("Counter", &json!({ "count": 3 }))?;
//! # Ok::<(), nsh_widgets::WidgetError>(())
//! ```

use std::sync::Mutex;

use nsh_core::prelude::*;
use serde_json::Value;

mod platform;
pub mod store;

pub use store::Store;

/// Why a widget call failed. [`WidgetError::code`] is stable; the message is for people.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum WidgetError {
    #[error("\"{0}\" is not a widget kind (a letter, then letters, digits, _ or -)")]
    InvalidKind(String),
    #[error("\"{0}\" is not a widget action (inc:key, dec:key or toggle:key)")]
    InvalidAction(String),
    #[error("{0}")]
    InvalidJson(String),
    #[error("the app cannot open the App Group \"{0}\": is it in the app's entitlements? Run nsh-widgets build")]
    NoAppGroup(String),
    #[error("no App Group set: run nsh-widgets build, or call configure(group)")]
    NoGroupConfigured,
    #[error("{0}")]
    Io(String),
    #[error("widgets are not available on this platform")]
    Unsupported,
    #[error("Live Activities are turned off for this app in Settings")]
    ActivitiesDisabled,
    #[error("no Live Activity of kind \"{0}\" is running")]
    NoActivity(String),
    #[error(
        "ActivityKit refused to start the Live Activity: is the kind listed under \"activities\" in widgets.json, and was nsh-widgets build run?"
    )]
    ActivityRefused,
}

impl WidgetError {
    /// A stable code for each failure.
    pub fn code(&self) -> &'static str {
        match self {
            WidgetError::InvalidKind(_) => "invalid_kind",
            WidgetError::InvalidAction(_) => "invalid_action",
            WidgetError::InvalidJson(_) => "invalid_json",
            WidgetError::NoAppGroup(_) => "no_app_group",
            WidgetError::NoGroupConfigured => "no_group_configured",
            WidgetError::Io(_) => "io",
            WidgetError::Unsupported => "unsupported",
            WidgetError::ActivitiesDisabled => "activities_disabled",
            WidgetError::NoActivity(_) => "no_activity",
            WidgetError::ActivityRefused => "activity_refused",
        }
    }
}

static GROUP: Mutex<Option<String>> = Mutex::new(None);

/// Uses `group` as the App Group on Apple platforms instead of the `NshWidgetsAppGroup` Info.plist key
/// `nsh-widgets build` writes. On Linux and Windows it names the store directory `nsh-widgets show`
/// reads. Android ignores it: the widget runs in the app's own process there.
pub fn configure(group: impl Into<String>) {
    *GROUP.lock().unwrap_or_else(|e| e.into_inner()) = Some(group.into());
}

fn group() -> Option<String> {
    GROUP.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

/// The store this app's widgets read.
pub fn store() -> Result<Store, WidgetError> {
    platform::root(group()).map(Store::new)
}

/// Writes `kind`'s UI tree (Widget Tree IR v1) and redraws the widget.
pub fn set_ui(kind: &str, snapshot: &Value) -> Result<(), WidgetError> {
    store()?.set_ui(kind, snapshot)?;
    platform::reload(kind)
}

/// Replaces `kind`'s data and redraws the widget.
pub fn set_data(kind: &str, data: &Value) -> Result<(), WidgetError> {
    store()?.set_data(kind, data)?;
    platform::reload(kind)
}

/// Merges `patch` into `kind`'s data, keeping what the widget changed, redraws, and returns the data.
pub fn update_data(kind: &str, patch: &Value) -> Result<Value, WidgetError> {
    let data = store()?.update_data(kind, patch)?;
    platform::reload(kind)?;
    Ok(data)
}

/// `kind`'s data, as the widget last left it.
pub fn data(kind: &str) -> Result<Value, WidgetError> {
    store()?.data(kind)
}

/// The button presses queued since the last call, oldest first: `{id, kind, action, at}` each.
pub fn take_actions() -> Result<Vec<Value>, WidgetError> {
    store()?.take_actions()
}

/// Redraws `kind`'s placed widgets.
pub fn reload(kind: &str) -> Result<(), WidgetError> {
    store::check_kind(kind)?;
    platform::reload(kind)
}

/// Starts `kind` as an iOS Live Activity, drawn from the kind's snapshot (`root` on the lock screen
/// and the expanded Dynamic Island, `accessory` in the compact one) with `data` over its stored data.
/// A running activity of the same kind is replaced. Elsewhere: [`WidgetError::Unsupported`].
pub fn start_activity(kind: &str, data: &Value) -> Result<(), WidgetError> {
    store::check_kind(kind)?;
    if !data.is_object() {
        return Err(WidgetError::InvalidJson("activity data must be a JSON object".into()));
    }
    platform::activity_start(kind, &data.to_string())
}

/// Replaces the running `kind` activity's data.
pub fn update_activity(kind: &str, data: &Value) -> Result<(), WidgetError> {
    store::check_kind(kind)?;
    if !data.is_object() {
        return Err(WidgetError::InvalidJson("activity data must be a JSON object".into()));
    }
    platform::activity_update(kind, &data.to_string())
}

/// Ends the running `kind` activity, if any.
pub fn end_activity(kind: &str) -> Result<(), WidgetError> {
    store::check_kind(kind)?;
    platform::activity_end(kind)
}

/// Resolves with `kind`'s data the next time a widget button changes it.
pub async fn next_change(kind: &str) -> Result<Value, WidgetError> {
    let store = store()?;
    store::check_kind(kind)?;
    platform::observe(group());
    let seen = (store.change_token(kind), platform::change_count());
    loop {
        timer(std::time::Duration::from_millis(250)).await;
        if (store.change_token(kind), platform::change_count()) != seen {
            return store.data(kind);
        }
    }
}

fn js(result: Result<impl ToString, WidgetError>) -> Result<String, String> {
    result.map(|v| v.to_string()).map_err(|e| format!("{}: {e}", e.code()))
}

fn parse(json: &str) -> Result<Value, WidgetError> {
    serde_json::from_str(json).map_err(|e| WidgetError::InvalidJson(e.to_string()))
}

/// The TSX side (`nsh-widgets/native`). JSON goes over as strings; errors are `"<code>: <message>"`.
/// The `nsh-widgets` TypeScript package wraps it.
#[module]
pub struct NativeWidgets;

#[module]
impl NativeWidgets {
    /// Sets the App Group on Apple platforms, or the store's name on Linux and Windows (see [`configure`]).
    pub fn configure(group: String) {
        configure(group);
    }

    /// Writes `kind`'s snapshot (Widget Tree IR v1, as JSON) and redraws it.
    pub fn set_ui(kind: String, json: String) -> Result<String, String> {
        js(parse(&json).and_then(|v| set_ui(&kind, &v)).map(|()| "null"))
    }

    /// Replaces `kind`'s data (a JSON object) and redraws it.
    pub fn set_data(kind: String, json: String) -> Result<String, String> {
        js(parse(&json).and_then(|v| set_data(&kind, &v)).map(|()| "null"))
    }

    /// Merges a JSON object into `kind`'s data, redraws, and returns the data as JSON.
    pub fn update_data(kind: String, json: String) -> Result<String, String> {
        js(parse(&json).and_then(|v| update_data(&kind, &v)))
    }

    /// `kind`'s data as JSON.
    pub fn data(kind: String) -> Result<String, String> {
        js(data(&kind))
    }

    /// The queued button presses as a JSON array, each returned once.
    pub fn take_actions() -> Result<String, String> {
        js(take_actions().map(Value::from))
    }

    /// Redraws `kind`'s placed widgets.
    pub fn reload(kind: String) -> Result<String, String> {
        js(reload(&kind).map(|()| "null"))
    }

    /// Starts `kind` as an iOS Live Activity with `json` (an object) as its data.
    pub fn start_activity(kind: String, json: String) -> Result<String, String> {
        js(parse(&json).and_then(|v| start_activity(&kind, &v)).map(|()| "null"))
    }

    /// Replaces the running `kind` activity's data.
    pub fn update_activity(kind: String, json: String) -> Result<String, String> {
        js(parse(&json).and_then(|v| update_activity(&kind, &v)).map(|()| "null"))
    }

    /// Ends the running `kind` activity.
    pub fn end_activity(kind: String) -> Result<String, String> {
        js(end_activity(&kind).map(|()| "null"))
    }

    /// Resolves with `kind`'s data (JSON) the next time a widget button changes it.
    pub async fn next_change(kind: String) -> Result<String, String> {
        js(next_change(&kind).await)
    }
}

#[native_module("nsh-widgets/native")]
fn native(registry: &mut Registry) {
    registry.module::<NativeWidgets>();
}
