//! The files the app and its widgets share, under `<root>/nsh/widgets/`:
//!
//! - `<kind>/snapshot.json`: the widget's UI tree (Widget Tree IR v1), written by the app.
//! - `<kind>/data.json`: `{"schema":1,"data":{..}}`, written by the app and by widget buttons.
//! - `actions.jsonl`: one event per widget button press, appended by the widget, drained by the app.
//!
//! Every write goes to a temporary file first and is renamed into place, so a widget never reads half
//! a file. The renderers in Swift and Java read and write the same layout.

use std::{
    fs,
    io::Write as _,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use serde_json::{Map, Value, json};

use crate::WidgetError;

/// The schema of `data.json`'s envelope.
pub const DATA_SCHEMA: u64 = 1;
/// The Widget Tree IR version `snapshot.json` holds.
pub const IR_VERSION: u64 = 1;

/// The shared files of one app's widgets.
#[derive(Debug, Clone)]
pub struct Store {
    base: PathBuf,
}

impl Store {
    /// The store under `root` (the App Group container on Apple platforms, the app's files directory
    /// on Android).
    pub fn new(root: impl AsRef<Path>) -> Self {
        Store {
            base: root.as_ref().join("nsh").join("widgets"),
        }
    }

    /// `<root>/nsh/widgets`.
    pub fn base(&self) -> &Path {
        &self.base
    }

    fn dir(&self, kind: &str) -> Result<PathBuf, WidgetError> {
        check_kind(kind)?;
        Ok(self.base.join(kind))
    }

    /// Writes `kind`'s UI tree. It must be a Widget Tree IR v1 document: `{"v":1,"root":{..}}`.
    pub fn set_ui(&self, kind: &str, snapshot: &Value) -> Result<(), WidgetError> {
        check_snapshot(snapshot)?;
        write_atomic(&self.dir(kind)?, "snapshot.json", snapshot.to_string().as_bytes())
    }

    /// Replaces `kind`'s data with `data`, a JSON object.
    pub fn set_data(&self, kind: &str, data: &Value) -> Result<(), WidgetError> {
        if !data.is_object() {
            return Err(WidgetError::InvalidJson("widget data must be a JSON object".into()));
        }
        let envelope = json!({ "schema": DATA_SCHEMA, "data": data });
        write_atomic(&self.dir(kind)?, "data.json", envelope.to_string().as_bytes())
    }

    /// Merges `patch`'s top-level keys into `kind`'s data and returns the result. Keys the widget
    /// changed and the patch does not name keep the widget's value.
    pub fn update_data(&self, kind: &str, patch: &Value) -> Result<Value, WidgetError> {
        let Value::Object(patch) = patch else {
            return Err(WidgetError::InvalidJson(
                "a widget data patch must be a JSON object".into(),
            ));
        };
        let mut data = self.data_map(kind)?;
        for (key, value) in patch {
            data.insert(key.clone(), value.clone());
        }
        let data = Value::Object(data);
        self.set_data(kind, &data)?;
        Ok(data)
    }

    /// `kind`'s data, or `{}` before anything wrote it.
    pub fn data(&self, kind: &str) -> Result<Value, WidgetError> {
        self.data_map(kind).map(Value::Object)
    }

    fn data_map(&self, kind: &str) -> Result<Map<String, Value>, WidgetError> {
        let path = self.dir(kind)?.join("data.json");
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Map::new()),
            Err(e) => return Err(WidgetError::Io(format!("{}: {e}", path.display()))),
        };
        let envelope: Value =
            serde_json::from_str(&text).map_err(|e| WidgetError::InvalidJson(format!("{}: {e}", path.display())))?;
        Ok(envelope
            .get("data")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default())
    }

    /// `kind`'s UI tree, when the app wrote one.
    pub fn snapshot(&self, kind: &str) -> Result<Option<Value>, WidgetError> {
        let path = self.dir(kind)?.join("snapshot.json");
        match fs::read_to_string(&path) {
            Ok(text) => serde_json::from_str(&text)
                .map(Some)
                .map_err(|e| WidgetError::InvalidJson(format!("{}: {e}", path.display()))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(WidgetError::Io(format!("{}: {e}", path.display()))),
        }
    }

    /// Takes every queued button event, oldest first. Each is returned once: the inbox is moved aside
    /// before it is read, so a press that lands meanwhile starts a new inbox.
    pub fn take_actions(&self) -> Result<Vec<Value>, WidgetError> {
        let inbox = self.base.join("actions.jsonl");
        let taking = self.base.join("actions.jsonl.taking");
        match fs::rename(&inbox, &taking) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound && !taking.exists() => return Ok(Vec::new()),
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(io(&inbox, e)),
            Err(_) => {} // a previous take stopped after the rename: finish it
        }
        let text = fs::read_to_string(&taking).map_err(|e| io(&taking, e))?;
        let events = text
            .lines()
            .filter(|line| !line.trim().is_empty())
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();
        fs::remove_file(&taking).map_err(|e| io(&taking, e))?;
        Ok(events)
    }

    /// Runs one button action on `kind`'s data, as the widget renderers do, and queues its event.
    /// Returns the new data. Used by hosts without a widget process of their own, and by tests.
    pub fn apply(&self, kind: &str, action: &str) -> Result<Value, WidgetError> {
        let (verb, key) = parse_action(action)?;
        let mut data = self.data_map(kind)?;
        let next = match verb {
            "inc" => json!(data.get(key).and_then(Value::as_i64).unwrap_or(0) + 1),
            "dec" => json!(data.get(key).and_then(Value::as_i64).unwrap_or(0) - 1),
            _ => json!(!data.get(key).and_then(Value::as_bool).unwrap_or(false)),
        };
        data.insert(key.to_owned(), next);
        let data = Value::Object(data);
        self.set_data(kind, &data)?;
        self.push_action(kind, action)?;
        Ok(data)
    }

    fn push_action(&self, kind: &str, action: &str) -> Result<(), WidgetError> {
        fs::create_dir_all(&self.base).map_err(|e| io(&self.base, e))?;
        let at = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let line = json!({ "id": format!("{at}-{kind}-{action}"), "kind": kind, "action": action, "at": at });
        let path = self.base.join("actions.jsonl");
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| io(&path, e))?;
        writeln!(file, "{line}").map_err(|e| io(&path, e))
    }

    /// Changes whenever `kind`'s data.json is rewritten: its modification time in nanoseconds, or 0.
    pub fn change_token(&self, kind: &str) -> u128 {
        let Ok(dir) = self.dir(kind) else { return 0 };
        fs::metadata(dir.join("data.json"))
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    }
}

fn io(path: &Path, e: std::io::Error) -> WidgetError {
    WidgetError::Io(format!("{}: {e}", path.display()))
}

fn write_atomic(dir: &Path, name: &str, bytes: &[u8]) -> Result<(), WidgetError> {
    fs::create_dir_all(dir).map_err(|e| io(dir, e))?;
    let tmp = dir.join(format!("{name}.tmp"));
    fs::write(&tmp, bytes).map_err(|e| io(&tmp, e))?;
    let path = dir.join(name);
    fs::rename(&tmp, &path).map_err(|e| io(&path, e))
}

/// A widget kind: a letter, then letters, digits, `_` or `-`. It names a directory and a WidgetKit kind.
pub fn check_kind(kind: &str) -> Result<(), WidgetError> {
    let mut chars = kind.chars();
    let ok = chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        && kind.len() <= 64;
    if ok {
        Ok(())
    } else {
        Err(WidgetError::InvalidKind(kind.to_owned()))
    }
}

/// Splits a button action of the vocabulary every renderer runs alone: `inc:key`, `dec:key`, `toggle:key`.
pub fn parse_action(action: &str) -> Result<(&str, &str), WidgetError> {
    let invalid = || WidgetError::InvalidAction(action.to_owned());
    let (verb, key) = action.split_once(':').ok_or_else(invalid)?;
    let key_ok = key.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.');
    match verb {
        "inc" | "dec" | "toggle" if key_ok => Ok((verb, key)),
        _ => Err(invalid()),
    }
}

/// The node types IR v1 has. A renderer draws an unknown type as nothing.
pub const NODE_TYPES: &[&str] = &[
    "col", "row", "text", "button", "spacer", "progress", "bars", "icon", "timer", "clock", "toggle", "rotate",
];

/// Checks a snapshot's envelope and that every node has a known type.
pub fn check_snapshot(snapshot: &Value) -> Result<(), WidgetError> {
    let invalid = |why: String| WidgetError::InvalidJson(format!("snapshot: {why}"));
    if snapshot.get("v").and_then(Value::as_u64) != Some(IR_VERSION) {
        return Err(invalid(format!("\"v\" must be {IR_VERSION}")));
    }
    let root = snapshot.get("root").ok_or_else(|| invalid("no \"root\" node".into()))?;
    let mut stack = vec![("root".to_owned(), root)];
    if let Some(accessory) = snapshot.get("accessory") {
        stack.push(("accessory".to_owned(), accessory));
    }
    while let Some((at, node)) = stack.pop() {
        let t = node
            .get("t")
            .and_then(Value::as_str)
            .ok_or_else(|| invalid(format!("{at} has no \"t\"")))?;
        if !NODE_TYPES.contains(&t) {
            return Err(invalid(format!("{at} has the unknown type \"{t}\"")));
        }
        if let Some(Value::String(action)) = node.get("action")
            && !action.starts_with("open:")
        {
            parse_action(action)?;
        }
        if let Some(children) = node.get("children").and_then(Value::as_array) {
            for (i, child) in children.iter().enumerate() {
                stack.push((format!("{at}.children[{i}]"), child));
            }
        }
    }
    Ok(())
}
