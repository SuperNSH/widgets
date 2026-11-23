//! Writes what packaging needs into the app's `nsh.json`, as nsh-maven writes `android.libraries`:
//! the App Group entitlement and Info.plist key, the extension entries, and the app's Android library.
//! The app's own keys stay, in their order; only nsh-widgets' entries are replaced.

use serde_json::{Map, Value, json};

use crate::{
    android,
    apple::{MODULE, Platform},
    config::Config,
};

const GROUPS: &str = "com.apple.security.application-groups";

/// What was built, so only those entries are written.
#[derive(Debug, Clone, Copy, Default)]
pub struct Built {
    pub ios: bool,
    pub macos: bool,
    pub android: bool,
}

fn object<'a>(parent: &'a mut Map<String, Value>, key: &str) -> &'a mut Map<String, Value> {
    let slot = parent.entry(key).or_insert_with(|| json!({}));
    if !slot.is_object() {
        *slot = json!({});
    }
    slot.as_object_mut().expect("object")
}

fn array<'a>(parent: &'a mut Map<String, Value>, key: &str) -> &'a mut Vec<Value> {
    let slot = parent.entry(key).or_insert_with(|| json!([]));
    if !slot.is_array() {
        *slot = json!([]);
    }
    slot.as_array_mut().expect("array")
}

fn add_unique(list: &mut Vec<Value>, value: Value) {
    if !list.contains(&value) {
        list.push(value);
    }
}

fn is_ours(entry: &Value) -> bool {
    entry
        .get("path")
        .and_then(Value::as_str)
        .is_some_and(|p| p.ends_with(&format!("{MODULE}.appex")))
}

fn set_extension(platform: &mut Map<String, Value>, entry: Value) {
    let list = array(platform, "extensions");
    list.retain(|e| !is_ours(e));
    list.push(entry);
}

fn path(p: std::path::PathBuf) -> String {
    p.to_string_lossy().replace('\\', "/")
}

/// Updates `nsh` (the parsed `nsh.json`) in place.
pub fn apply(nsh: &mut Value, config: &Config, built: Built) {
    let Some(root) = nsh.as_object_mut() else { return };
    if built.ios {
        let ios = object(root, "ios");
        add_unique(array(object(ios, "entitlements"), GROUPS), json!(config.app_group));
        object(ios, "infoPlist").insert("NshWidgetsAppGroup".into(), json!(config.app_group));
        set_extension(
            ios,
            json!({
                "path": path(Platform::Device.appex()),
                "simulatorPath": path(Platform::Simulator.appex()),
                "entitlements": { GROUPS: [config.app_group] },
            }),
        );
    }
    if built.macos {
        let group = config.macos_group();
        let macos = object(root, "macos");
        add_unique(array(object(macos, "entitlements"), GROUPS), json!(group));
        object(macos, "infoPlist").insert("NshWidgetsAppGroup".into(), json!(group));
        // macOS runs widget extensions only sandboxed.
        set_extension(
            macos,
            json!({
                "path": path(Platform::Mac.appex()),
                "entitlements": { "com.apple.security.app-sandbox": true, GROUPS: [group] },
            }),
        );
    }
    if built.android {
        add_unique(array(object(root, "android"), "libraries"), json!(android::AAR));
    }
}
