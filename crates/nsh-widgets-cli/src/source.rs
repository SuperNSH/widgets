//! Where `preview` and `show` read widgets from, and the two commands themselves.

use std::{
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{Context as _, bail};
use serde_json::{Map, Value, json};

use crate::{
    Args,
    config::{Config, LOCK_SCREEN},
    render::{self, Ctx},
};

#[derive(Debug, Default)]
pub struct Source {
    pub simulator: bool,
    pub dir: Option<PathBuf>,
    pub snapshot: Option<PathBuf>,
    pub data: Option<PathBuf>,
}

/// One widget to draw.
pub struct Widget {
    pub kind: String,
    pub snapshot: Value,
    pub data: Map<String, Value>,
    pub families: Vec<String>,
}

fn read_json(path: &Path) -> anyhow::Result<Value> {
    let text = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| path.display().to_string())
}

fn config(args: &Args) -> Option<Config> {
    let path = args.app.join("widgets.json");
    path.is_file().then(|| Config::read(&path)).transpose().ok().flatten()
}

/// The per-user data directory, as the crate's desktop store computes it.
pub fn data_home() -> Option<PathBuf> {
    let var = |k: &str| std::env::var_os(k).filter(|v| !v.is_empty()).map(PathBuf::from);
    if cfg!(windows) {
        var("LOCALAPPDATA")
    } else {
        var("XDG_DATA_HOME").or_else(|| var("HOME").map(|h| h.join(".local/share")))
    }
}

/// The booted iOS simulator's container for App Group `group`.
fn simulator_group(group: &str) -> anyhow::Result<PathBuf> {
    let out = Command::new("xcrun")
        .args(["simctl", "list", "devices", "booted", "-j"])
        .output()?;
    let list: Value = serde_json::from_slice(&out.stdout).context("simctl list")?;
    let home = std::env::var_os("HOME").context("HOME")?;
    for devices in list["devices"].as_object().into_iter().flat_map(|m| m.values()) {
        for d in devices.as_array().into_iter().flatten() {
            let Some(udid) = d["udid"].as_str() else { continue };
            let groups = Path::new(&home)
                .join("Library/Developer/CoreSimulator/Devices")
                .join(udid)
                .join("data/Containers/Shared/AppGroup");
            for entry in std::fs::read_dir(&groups).into_iter().flatten().flatten() {
                let meta = entry.path().join(".com.apple.mobile_container_manager.metadata.plist");
                let id = Command::new("plutil")
                    .args(["-extract", "MCMMetadataIdentifier", "raw"])
                    .arg(&meta)
                    .output();
                if id.is_ok_and(|o| String::from_utf8_lossy(&o.stdout).trim() == group) {
                    return Ok(entry.path());
                }
            }
        }
    }
    bail!("no booted simulator has the App Group {group}: run the app there first")
}

/// The store root (the directory holding `nsh/widgets`) the arguments name.
fn store_root(args: &Args) -> anyhow::Result<PathBuf> {
    if let Some(dir) = &args.source.dir {
        return Ok(dir.clone());
    }
    if args.source.simulator {
        let config = config(args).context("--simulator needs the app's widgets.json (--app DIR)")?;
        return simulator_group(&config.app_group);
    }
    if let Some(group) = &args.group {
        return Ok(data_home()
            .context("no data directory")?
            .join("nsh-widgets")
            .join(group));
    }
    if let Some(dir) = std::env::var_os("NSH_WIDGETS_DIR") {
        return Ok(dir.into());
    }
    bail!("say where the widgets are: --simulator, --dir, --group or --snapshot (see nsh-widgets help)")
}

/// Every widget the arguments name, with its data and sizes.
pub fn load(args: &Args) -> anyhow::Result<Vec<Widget>> {
    let config = config(args);
    let families_of = |kind: &str| {
        config
            .as_ref()
            .and_then(|c| c.widgets.iter().find(|w| w.kind == kind))
            .map(|w| w.families.clone())
            .unwrap_or_else(|| vec!["small".into(), "medium".into(), "large".into()])
    };
    if let Some(path) = &args.source.snapshot {
        let kind = args.kind.clone().unwrap_or_else(|| "preview".into());
        let data = match &args.source.data {
            Some(d) => read_json(d)?,
            None => json!({}),
        };
        let data = data
            .get("data")
            .unwrap_or(&data)
            .as_object()
            .cloned()
            .unwrap_or_default();
        return Ok(vec![Widget {
            families: families_of(&kind),
            kind,
            snapshot: read_json(path)?,
            data,
        }]);
    }
    let base = store_root(args)?.join("nsh/widgets");
    let kinds: Vec<String> = match (&args.kind, &config) {
        (Some(k), _) => vec![k.clone()],
        (None, Some(c)) => c.widgets.iter().map(|w| w.kind.clone()).collect(),
        (None, None) => {
            let mut k: Vec<String> = std::fs::read_dir(&base)
                .with_context(|| format!("read {}", base.display()))?
                .flatten()
                .filter(|e| e.path().join("snapshot.json").is_file())
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect();
            k.sort();
            k
        }
    };
    let mut out = Vec::new();
    for kind in kinds {
        let dir = base.join(&kind);
        let Ok(snapshot) = read_json(&dir.join("snapshot.json")) else {
            eprintln!("note: {kind} has no snapshot.json yet: the app has not published it");
            continue;
        };
        let data = read_json(&dir.join("data.json")).unwrap_or(json!({}));
        let data = data.get("data").and_then(Value::as_object).cloned().unwrap_or_default();
        out.push(Widget {
            families: families_of(&kind),
            kind,
            snapshot,
            data,
        });
    }
    Ok(out)
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn clock() -> String {
    Command::new("date")
        .arg("+%H:%M")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            let s = now_ms() / 1000;
            format!("{:02}:{:02}", s / 3600 % 24, s / 60 % 60)
        })
}

/// An HTML page with every widget in each of its sizes, light and dark.
pub fn page(widgets: &[Widget]) -> String {
    let mut body = String::new();
    let (now, clock) = (now_ms(), clock());
    for w in widgets {
        body.push_str(&format!("<h2>{}</h2>", w.kind.replace('<', "&lt;")));
        for dark in [false, true] {
            body.push_str(&format!("<div class=\"row {}\">", if dark { "dark" } else { "light" }));
            for family in &w.families {
                let ctx = Ctx {
                    data: &w.data,
                    family,
                    dark,
                    now_ms: now,
                    clock: clock.clone(),
                };
                let label = if LOCK_SCREEN.contains(&family.as_str()) {
                    format!("{family} (lock screen)")
                } else {
                    family.clone()
                };
                body.push_str(&format!(
                    "<figure>{}<figcaption>{label}</figcaption></figure>",
                    render::widget_html(&w.snapshot, &ctx)
                ));
            }
            body.push_str("</div>");
        }
    }
    format!(
        "<!doctype html><meta charset=\"utf-8\"><title>nsh-widgets preview</title><style>\
body{{font:14px -apple-system,system-ui,sans-serif;margin:24px;background:#f2f2f7}}\
h2{{font-size:15px;margin:28px 0 8px}}.row{{display:flex;flex-wrap:wrap;gap:20px;padding:16px;border-radius:16px;margin-bottom:8px}}\
.light{{background:#e9e9ee}}.dark{{background:#111}}.dark figcaption{{color:#aaa}}\
figure{{margin:0}}figcaption{{font-size:12px;color:#555;margin-top:6px}}.w{{box-sizing:border-box;overflow:hidden;box-shadow:0 2px 8px rgba(0,0,0,.15)}}\
</style><p>Drawn from the IR by nsh-widgets preview, with the app's data. The native renderers are the reference.</p>{body}"
    )
}

pub fn preview(args: &Args) -> anyhow::Result<()> {
    let widgets = load(args)?;
    if widgets.is_empty() {
        bail!("no widget has a snapshot yet: open the app so it publishes them");
    }
    let out = args
        .out
        .clone()
        .unwrap_or_else(|| args.app.join(".nsh-widgets/preview.html"));
    if let Some(dir) = out.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(&out, page(&widgets))?;
    println!("wrote {} ({} widgets)", out.display(), widgets.len());
    Ok(())
}

/// One widget as text: lines, Waybar's JSON (`text` on the bar, the rest as `tooltip`), or the IR
/// itself with its data (`ir`), which the KDE Plasma host draws.
pub fn show_text(w: &Widget, format: &str, now: i64, clock: String) -> anyhow::Result<String> {
    if format == "ir" {
        return Ok(json!({ "snapshot": w.snapshot, "data": w.data, "now": now }).to_string());
    }
    let family = if format == "waybar" { "inline" } else { "medium" };
    let ctx = Ctx {
        data: &w.data,
        family,
        dark: false,
        now_ms: now,
        clock,
    };
    let root = if family == "inline" {
        w.snapshot.get("accessory").unwrap_or(&w.snapshot["root"])
    } else {
        &w.snapshot["root"]
    };
    let lines = render::text_lines(root, &ctx);
    Ok(match format {
        "text" => lines.join("\n"),
        "waybar" => {
            let full = render::text_lines(
                &w.snapshot["root"],
                &Ctx {
                    family: "medium",
                    ..ctx
                },
            );
            json!({ "text": lines.join(" "), "tooltip": full.join("\n"), "class": w.kind }).to_string()
        }
        other => bail!("--format {other}: use text, waybar or ir"),
    })
}

pub fn show(args: &Args) -> anyhow::Result<()> {
    if args.kind.is_none() {
        bail!("show needs a widget kind: nsh-widgets show KIND");
    }
    let mut last = String::new();
    loop {
        let widgets = load(args)?;
        let w = widgets.first().context("that widget has no snapshot yet")?;
        let text = show_text(w, &args.format, now_ms(), clock())?;
        if text != last {
            println!("{text}");
            last = text;
        }
        if !args.follow {
            return Ok(());
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}

/// Runs one button action (inc:key, dec:key, toggle:key) on a kind's data in a store, as the widget
/// renderers do, and queues its event for the app. For hosts without a widget process: KDE Plasma.
pub fn apply(args: &Args, action: &str) -> anyhow::Result<()> {
    let kind = args
        .kind
        .as_deref()
        .context("apply needs a widget kind: nsh-widgets apply KIND ACTION")?;
    if !crate::config::valid_kind(kind) {
        bail!("{kind:?} is not a widget kind");
    }
    if !crate::config::valid_action(action) {
        bail!("{action:?} is not an action (inc:key, dec:key or toggle:key)");
    }
    let base = store_root(args)?.join("nsh/widgets");
    let dir = base.join(kind);
    std::fs::create_dir_all(&dir)?;
    let mut data = read_json(&dir.join("data.json"))
        .ok()
        .and_then(|v| v.get("data").and_then(Value::as_object).cloned())
        .unwrap_or_default();
    let (verb, key) = action.split_once(':').expect("checked");
    let next = match verb {
        "inc" => json!(data.get(key).and_then(Value::as_i64).unwrap_or(0) + 1),
        "dec" => json!(data.get(key).and_then(Value::as_i64).unwrap_or(0) - 1),
        _ => json!(!data.get(key).and_then(Value::as_bool).unwrap_or(false)),
    };
    data.insert(key.to_owned(), next);
    let tmp = dir.join("data.json.tmp");
    std::fs::write(&tmp, json!({ "schema": 1, "data": data }).to_string())?;
    std::fs::rename(&tmp, dir.join("data.json"))?;
    let at = now_ms();
    let event = json!({ "id": format!("{at}-{kind}-{action}"), "kind": kind, "action": action, "at": at });
    use std::io::Write as _;
    let mut inbox = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(base.join("actions.jsonl"))?;
    writeln!(inbox, "{event}")?;
    Ok(())
}
