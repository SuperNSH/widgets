//! `widgets.json`, beside the app's `nsh.json`: the App Group and the widget kinds. It stays out of
//! `nsh.json` because NSH warns about keys it does not know.

use std::path::{Path, PathBuf};

use anyhow::{Context as _, bail};
use serde::Deserialize;
use serde_json::Value;

/// How many kinds an app can have: the Android library ships this many provider slots, and one
/// WidgetKit bundle block takes ten widgets, so two blocks hold them.
pub const MAX_KINDS: usize = 16;
/// How many controls an app can have: the Android library ships this many Quick Settings tile slots.
pub const MAX_CONTROLS: usize = 8;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
    /// The App Group the app and its iOS widget extension share.
    pub app_group: String,
    /// The macOS App Group, when it differs: a Mac app signed with a Personal Team needs a group that
    /// starts with the team ID. Defaults to `app_group`.
    #[serde(default)]
    pub macos_app_group: Option<String>,
    /// Font files bundled into the widget extension, relative to widgets.json. A snapshot's `font`
    /// names one of them by family. Android widgets cannot load app fonts and use the system font.
    #[serde(default)]
    pub fonts: Vec<PathBuf>,
    #[serde(default)]
    pub deployment_target: DeploymentTarget,
    pub widgets: Vec<WidgetKind>,
    /// Kinds that can run as iOS Live Activities (lock screen and Dynamic Island). The app starts them
    /// with `Widgets.startActivity`; they draw the kind's snapshot like a widget does.
    #[serde(default)]
    pub activities: Vec<String>,
    /// Control Center and lock-screen controls (iOS 18 and later).
    #[serde(default)]
    pub controls: Vec<Control>,
}

/// A user-chosen option on a widget kind: the widget's edit sheet on iOS, a configure screen on
/// Android. The data under the chosen value's key is laid over the kind's data, and `{option}` names it.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Options {
    pub title: String,
    pub values: Vec<String>,
}

/// A control: a toggle bound to a boolean in a kind's data, or a button that runs an action on it.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Control {
    pub kind: String,
    pub name: String,
    /// "toggle" or "button".
    #[serde(rename = "type")]
    pub kind_of: String,
    /// The widget kind whose data the control reads and changes.
    pub data: String,
    /// toggle: the boolean key.
    #[serde(default)]
    pub key: Option<String>,
    /// button: an action of the vocabulary (inc:key, dec:key, toggle:key).
    #[serde(default)]
    pub action: Option<String>,
    /// An icon of the shared set.
    #[serde(default)]
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeploymentTarget {
    #[serde(default = "ios_default")]
    pub ios: String,
    #[serde(default = "macos_default")]
    pub macos: String,
}

impl Default for DeploymentTarget {
    fn default() -> Self {
        DeploymentTarget {
            ios: ios_default(),
            macos: macos_default(),
        }
    }
}

fn ios_default() -> String {
    "17.0".into()
}
fn macos_default() -> String {
    "14.0".into()
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WidgetKind {
    /// The kind: the directory the app writes, the WidgetKit kind, the Android slot's meta-data.
    pub kind: String,
    /// The name in the widget gallery.
    pub name: String,
    #[serde(default)]
    pub description: String,
    /// "small", "medium", "large" (and "extraLarge", iPad and Mac only). Defaults to small and medium.
    #[serde(default = "families_default")]
    pub families: Vec<String>,
    #[serde(default)]
    pub android: AndroidKind,
    #[serde(default)]
    pub options: Option<Options>,
}

fn families_default() -> Vec<String> {
    vec!["small".into(), "medium".into()]
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AndroidKind {
    /// Home-screen cells [columns, rows] the widget starts at. Defaults from its smallest family.
    #[serde(default)]
    pub cells: Option<[u32; 2]>,
}

pub const FAMILIES: &[&str] = &[
    "small",
    "medium",
    "large",
    "extraLarge",
    "circular",
    "rectangular",
    "inline",
];
/// Lock-screen (accessory) families: iOS only; macOS and Android leave them out.
pub const LOCK_SCREEN: &[&str] = &["circular", "rectangular", "inline"];

impl WidgetKind {
    /// [columns, rows] on Android: small 2x2, medium 4x2, large 4x4.
    pub fn cells(&self) -> [u32; 2] {
        if let Some(cells) = self.android.cells {
            return cells;
        }
        let home = self.families.iter().find(|f| !LOCK_SCREEN.contains(&f.as_str()));
        match home.map(String::as_str) {
            Some("small") => [2, 2],
            Some("large" | "extraLarge") => [4, 4],
            _ => [4, 2],
        }
    }
}

impl Config {
    pub fn macos_group(&self) -> &str {
        self.macos_app_group.as_deref().unwrap_or(&self.app_group)
    }

    /// Reads and checks `widgets.json`.
    pub fn read(path: &Path) -> anyhow::Result<Config> {
        let text = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        let config: Config = serde_json::from_str(&text).with_context(|| format!("{}", path.display()))?;
        let problems = config.problems();
        if !problems.is_empty() {
            bail!("{}:\n  {}", path.display(), problems.join("\n  "));
        }
        Ok(config)
    }

    /// Everything wrong with the config, for `build` (which stops) and `check` (which lists them).
    pub fn problems(&self) -> Vec<String> {
        let mut out = Vec::new();
        if self.app_group.is_empty() {
            out.push("appGroup is empty".into());
        } else if !self.app_group.starts_with("group.") {
            out.push(format!(
                "appGroup \"{}\" must start with \"group.\" on iOS",
                self.app_group
            ));
        }
        if self.widgets.is_empty() {
            out.push("widgets is empty: add at least one kind".into());
        }
        if self.widgets.len() > MAX_KINDS {
            out.push(format!("{} widget kinds; at most {MAX_KINDS}", self.widgets.len()));
        }
        let mut seen = std::collections::BTreeSet::new();
        for w in &self.widgets {
            if !valid_kind(&w.kind) {
                out.push(format!("kind \"{}\": a letter, then letters, digits, _ or -", w.kind));
            }
            if !seen.insert(w.kind.as_str()) {
                out.push(format!("kind \"{}\" appears twice", w.kind));
            }
            if w.name.trim().is_empty() {
                out.push(format!("kind \"{}\" has no name", w.kind));
            }
            if w.families.is_empty() {
                out.push(format!("kind \"{}\" has no families", w.kind));
            }
            for f in &w.families {
                if !FAMILIES.contains(&f.as_str()) {
                    out.push(format!(
                        "kind \"{}\": unknown family \"{f}\" (small, medium, large, extraLarge, circular, rectangular, inline)",
                        w.kind
                    ));
                }
            }
            if let Some([c, r]) = w.android.cells
                && (!(1..=5).contains(&c) || !(1..=5).contains(&r))
            {
                out.push(format!("kind \"{}\": android.cells must be 1 to 5 each", w.kind));
            }
        }
        for a in &self.activities {
            if !valid_kind(a) {
                out.push(format!("activity \"{a}\": a letter, then letters, digits, _ or -"));
            }
        }
        for w in &self.widgets {
            if let Some(o) = &w.options {
                let mut seen = std::collections::BTreeSet::new();
                if o.title.trim().is_empty() || o.values.is_empty() || o.values.len() > 20 {
                    out.push(format!("kind \"{}\": options need a title and 1 to 20 values", w.kind));
                }
                for v in &o.values {
                    if v.trim().is_empty() || !seen.insert(v) || v.contains('|') {
                        out.push(format!(
                            "kind \"{}\": option values must be unique, not empty, and free of |",
                            w.kind
                        ));
                    }
                }
            }
        }
        if self.controls.len() > MAX_CONTROLS {
            out.push(format!(
                "{} controls; at most {MAX_CONTROLS} (Android tile slots)",
                self.controls.len()
            ));
        }
        let mut control_kinds = std::collections::BTreeSet::new();
        for c in &self.controls {
            if !valid_kind(&c.kind) || !control_kinds.insert(c.kind.as_str()) {
                out.push(format!(
                    "control \"{}\": a unique kind of letters, digits, _ or -",
                    c.kind
                ));
            }
            if !valid_kind(&c.data) {
                out.push(format!("control \"{}\": data names a widget kind", c.kind));
            }
            match (c.kind_of.as_str(), &c.key, &c.action) {
                ("toggle", Some(k), None) if valid_key(k) => {}
                ("button", None, Some(a)) if valid_action(a) => {}
                ("toggle", ..) => out.push(format!(
                    "control \"{}\": a toggle needs a \"key\" and no \"action\"",
                    c.kind
                )),
                ("button", ..) => out.push(format!(
                    "control \"{}\": a button needs an \"action\" (inc:, dec: or toggle:) and no \"key\"",
                    c.kind
                )),
                (t, ..) => out.push(format!("control \"{}\": type \"{t}\" is not toggle or button", c.kind)),
            }
            if let Some(i) = &c.icon
                && !crate::render::ICONS.contains(&i.as_str())
            {
                out.push(format!("control \"{}\": icon \"{i}\" is not in the shared set", c.kind));
            }
        }
        for font in &self.fonts {
            let ok = font
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| matches!(e, "ttf" | "otf"));
            if !ok {
                out.push(format!("font {}: only .ttf and .otf", font.display()));
            }
        }
        out
    }
}

pub fn valid_key(key: &str) -> bool {
    key.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
}

pub fn valid_action(action: &str) -> bool {
    action
        .split_once(':')
        .is_some_and(|(verb, key)| matches!(verb, "inc" | "dec" | "toggle") && valid_key(key))
}

pub fn valid_kind(kind: &str) -> bool {
    let mut chars = kind.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        && kind.len() <= 64
}

/// What `nsh-widgets` needs of `nsh.json`.
#[derive(Debug, Clone)]
pub struct App {
    pub id: String,
    pub name: String,
    pub version: String,
    pub build: String,
}

impl App {
    pub fn read(nsh_json: &Path) -> anyhow::Result<App> {
        let text = std::fs::read_to_string(nsh_json).with_context(|| format!("read {}", nsh_json.display()))?;
        let v: Value = serde_json::from_str(&text).with_context(|| format!("{}", nsh_json.display()))?;
        let field = |k: &str| v.get(k).and_then(Value::as_str).map(str::to_owned);
        let id = field("id").with_context(|| format!("{} has no \"id\"", nsh_json.display()))?;
        Ok(App {
            name: field("name").unwrap_or_else(|| id.clone()),
            version: field("version").unwrap_or_else(|| "1.0.0".into()),
            build: v
                .get("build")
                .map(|b| b.to_string().trim_matches('"').to_owned())
                .unwrap_or_else(|| "1".into()),
            id,
        })
    }
}
