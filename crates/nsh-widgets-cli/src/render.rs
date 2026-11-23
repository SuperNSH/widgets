//! Draws a Widget Tree IR v1 snapshot outside a widget host: as HTML for `nsh-widgets preview`, and as
//! text for `nsh-widgets show` (terminals and status bars). The native renderers stay the reference;
//! this follows them closely enough to check a layout and its data without a device.

use std::fmt::Write as _;

use serde_json::{Map, Value};

/// What a drawing depends on besides the tree.
pub struct Ctx<'a> {
    pub data: &'a Map<String, Value>,
    /// small, medium, large, or circular, rectangular, inline.
    pub family: &'a str,
    pub dark: bool,
    /// Milliseconds since the Unix epoch.
    pub now_ms: i64,
    /// The local time, "HH:MM", for `clock`.
    pub clock: String,
}

/// Fills `{key}` templates from the widget's data, as the renderers do.
pub fn fill(text: &str, data: &Map<String, Value>) -> String {
    let mut out = text.to_owned();
    for (k, v) in data {
        let needle = format!("{{{k}}}");
        if !out.contains(&needle) {
            continue;
        }
        let s = match v {
            Value::String(s) => s.clone(),
            Value::Number(n) => match n.as_f64() {
                Some(f) if f.fract() == 0.0 && f.abs() < 1e15 => format!("{}", f as i64),
                _ => n.to_string(),
            },
            other => other.to_string(),
        };
        out = out.replace(&needle, &s);
    }
    out
}

fn style(node: &Value) -> &Map<String, Value> {
    static EMPTY: std::sync::OnceLock<Map<String, Value>> = std::sync::OnceLock::new();
    node.get("style")
        .and_then(Value::as_object)
        .unwrap_or_else(|| EMPTY.get_or_init(Map::new))
}

fn hidden(node: &Value, family: &str) -> bool {
    style(node)
        .get("hide")
        .and_then(Value::as_array)
        .is_some_and(|h| h.iter().any(|f| f.as_str() == Some(family)))
}

fn children(node: &Value) -> &[Value] {
    node.get("children")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn number(node: &Value, data: &Map<String, Value>) -> Option<f64> {
    node.get("key")
        .and_then(Value::as_str)
        .and_then(|k| data.get(k))
        .and_then(Value::as_f64)
}

/// The shared icon set, as the renderers draw it.
pub const ICONS: &[&str] = &[
    "star",
    "bolt",
    "check",
    "play",
    "pause",
    "flame",
    "plus",
    "minus",
    "arrow-up",
    "arrow-down",
    "heart",
    "home",
    "person",
    "bell",
    "clock",
    "calendar",
    "cloud",
    "sun",
    "moon",
    "drop",
    "music",
];

/// The SF Symbol each shared icon draws as on Apple platforms.
pub fn sf_symbol(name: &str) -> &'static str {
    match name {
        "star" => "star.fill",
        "bolt" => "bolt.fill",
        "check" => "checkmark.circle.fill",
        "play" => "play.fill",
        "pause" => "pause.fill",
        "flame" => "flame.fill",
        "plus" => "plus",
        "minus" => "minus",
        "arrow-up" => "arrow.up",
        "arrow-down" => "arrow.down",
        "heart" => "heart.fill",
        "home" => "house.fill",
        "person" => "person.fill",
        "bell" => "bell.fill",
        "clock" => "clock.fill",
        "calendar" => "calendar",
        "cloud" => "cloud.fill",
        "sun" => "sun.max.fill",
        "moon" => "moon.fill",
        "drop" => "drop.fill",
        "music" => "music.note",
        _ => "questionmark.circle",
    }
}

/// A glyph for each icon of the shared set.
pub fn icon_glyph(name: &str) -> &'static str {
    match name {
        "star" => "★",
        "bolt" => "⚡",
        "check" => "✓",
        "play" => "▶",
        "pause" => "⏸",
        "flame" => "🔥",
        "plus" => "+",
        "minus" => "−",
        "arrow-up" => "↑",
        "arrow-down" => "↓",
        "heart" => "♥",
        "home" => "⌂",
        "person" => "👤",
        "bell" => "🔔",
        "clock" => "🕒",
        "calendar" => "📅",
        "cloud" => "☁",
        "sun" => "☀",
        "moon" => "☾",
        "drop" => "💧",
        "music" => "♪",
        _ => "?",
    }
}

/// `h:mm:ss` or `m:ss`, as widget timers show a duration.
pub fn duration(ms: i64) -> String {
    let s = ms.max(0) / 1000;
    let (h, m, s) = (s / 3600, s / 60 % 60, s % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

fn timer(node: &Value, ctx: &Ctx) -> String {
    let at = number(node, ctx.data).unwrap_or(ctx.now_ms as f64) as i64;
    if node.get("mode").and_then(Value::as_str) == Some("down") {
        duration(at - ctx.now_ms)
    } else {
        duration(ctx.now_ms - at)
    }
}

fn rotate_child<'a>(node: &'a Value, ctx: &Ctx) -> Option<&'a Value> {
    let kids = children(node);
    if kids.is_empty() {
        return None;
    }
    let step = (node.get("interval").and_then(Value::as_f64).unwrap_or(60.0).max(1.0) * 1000.0) as i64;
    Some(&kids[(ctx.now_ms / step) as usize % kids.len()])
}

/// The data each list item draws with: the widget's data plus the item's fields as `item.field`.
fn list_items(node: &Value, data: &Map<String, Value>) -> Vec<Map<String, Value>> {
    let max = node.get("max").and_then(Value::as_u64).unwrap_or(8) as usize;
    node.get("key")
        .and_then(Value::as_str)
        .and_then(|k| data.get(k))
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .take(max)
                .map(|item| {
                    let mut d = data.clone();
                    for (k, v) in item.as_object().into_iter().flatten() {
                        d.insert(format!("item.{k}"), v.clone());
                    }
                    d
                })
                .collect()
        })
        .unwrap_or_default()
}

fn bool_at(node: &Value, data: &Map<String, Value>) -> bool {
    node.get("key")
        .and_then(Value::as_str)
        .and_then(|k| data.get(k))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn values(node: &Value, data: &Map<String, Value>) -> Vec<f64> {
    node.get("key")
        .and_then(Value::as_str)
        .and_then(|k| data.get(k))
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_f64).collect())
        .unwrap_or_default()
}

/// The widget as lines of text: a column is lines, a row is one line.
pub fn text_lines(node: &Value, ctx: &Ctx) -> Vec<String> {
    if hidden(node, ctx.family) {
        return Vec::new();
    }
    let text = || fill(node.get("text").and_then(Value::as_str).unwrap_or(""), ctx.data);
    match node.get("t").and_then(Value::as_str).unwrap_or("") {
        "col" => children(node).iter().flat_map(|c| text_lines(c, ctx)).collect(),
        "row" => {
            let parts: Vec<String> = children(node)
                .iter()
                .map(|c| text_lines(c, ctx).join(" "))
                .filter(|s| !s.is_empty())
                .collect();
            if parts.is_empty() {
                Vec::new()
            } else {
                vec![parts.join(" ")]
            }
        }
        "text" => vec![text()],
        "button" => vec![format!("[{}]", text())],
        "progress" => {
            let v = number(node, ctx.data).unwrap_or(0.0).clamp(0.0, 1.0);
            let filled = (v * 10.0).round() as usize;
            vec![format!(
                "{}{} {}%",
                "▓".repeat(filled),
                "░".repeat(10 - filled),
                (v * 100.0).round()
            )]
        }
        "bars" => {
            let vals = values(node, ctx.data);
            let top = vals.iter().cloned().fold(1.0, f64::max);
            let blocks = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
            vec![
                vals.iter()
                    .map(|v| blocks[((v / top) * 7.0).round().clamp(0.0, 7.0) as usize])
                    .collect(),
            ]
        }
        "icon" => vec![icon_glyph(node.get("name").and_then(Value::as_str).unwrap_or("")).to_owned()],
        "timer" => vec![timer(node, ctx)],
        "clock" => vec![ctx.clock.clone()],
        "toggle" => vec![format!(
            "{} {}",
            if bool_at(node, ctx.data) { "☑" } else { "☐" },
            text()
        )],
        "rotate" => rotate_child(node, ctx).map(|c| text_lines(c, ctx)).unwrap_or_default(),
        "list" => match children(node).first() {
            Some(item) => list_items(node, ctx.data)
                .iter()
                .flat_map(|d| {
                    text_lines(
                        item,
                        &Ctx {
                            data: d,
                            clock: ctx.clock.clone(),
                            ..*ctx
                        },
                    )
                })
                .collect(),
            None => Vec::new(),
        },
        _ => Vec::new(),
    }
}

fn pick(pair: Option<&Value>, dark: bool) -> Option<&str> {
    let a = pair?.as_array()?;
    a.get(usize::from(dark)).or_else(|| a.first())?.as_str()
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// The CSS background of a style: its gradient, else its colour.
pub fn background(s: &Map<String, Value>, dark: bool) -> Option<String> {
    if let Some(g) = s.get("gradient") {
        let dir = match g.get("dir").and_then(Value::as_str) {
            Some("r") => "to right",
            Some("br") => "to bottom right",
            _ => "to bottom",
        };
        let (from, to) = (pick(g.get("from"), dark)?, pick(g.get("to"), dark)?);
        return Some(format!("linear-gradient({dir}, {from}, {to})"));
    }
    pick(s.get("bg"), dark).map(str::to_owned)
}

fn box_css(s: &Map<String, Value>, dark: bool, out: &mut String) {
    if let Some(p) = s.get("pad").and_then(Value::as_array) {
        let n: Vec<f64> = p.iter().filter_map(Value::as_f64).collect();
        if n.len() == 4 {
            let _ = write!(out, "padding:{}px {}px {}px {}px;", n[0], n[1], n[2], n[3]);
        }
    }
    if let Some(bg) = background(s, dark) {
        let _ = write!(out, "background:{bg};");
    }
    if let Some(r) = s.get("radius").and_then(Value::as_f64) {
        let _ = write!(out, "border-radius:{}px;", r.min(200.0));
    }
}

fn text_css(s: &Map<String, Value>, dark: bool, out: &mut String) {
    let size = s.get("size").and_then(Value::as_f64).unwrap_or(14.0);
    let weight = match s.get("weight").and_then(Value::as_str) {
        Some("medium") => 500,
        Some("semibold") => 600,
        Some("bold") => 700,
        _ => 400,
    };
    let _ = write!(
        out,
        "font-size:{size}px;font-weight:{weight};white-space:nowrap;overflow:hidden;text-overflow:ellipsis;"
    );
    if let Some(fg) = pick(s.get("fg"), dark) {
        let _ = write!(out, "color:{fg};");
    }
}

/// The widget's tree as HTML, inside a frame the size of its family.
pub fn html(node: &Value, ctx: &Ctx, out: &mut String) {
    if hidden(node, ctx.family) {
        return;
    }
    let s = style(node);
    let mut css = String::new();
    let text = || esc(&fill(node.get("text").and_then(Value::as_str).unwrap_or(""), ctx.data));
    match node.get("t").and_then(Value::as_str).unwrap_or("") {
        t @ ("col" | "row") => {
            let gap = s.get("gap").and_then(Value::as_f64).unwrap_or(0.0);
            let align = match (t, s.get("align").and_then(Value::as_str)) {
                (_, Some("center")) => "center",
                (_, Some("end")) => "flex-end",
                ("row", Some("start")) => "flex-start",
                ("row", _) => "center",
                _ => "stretch",
            };
            let dir = if t == "row" { "row" } else { "column" };
            let _ = write!(
                css,
                "display:flex;flex-direction:{dir};gap:{gap}px;align-items:{align};min-width:0;"
            );
            if s.get("grow").and_then(Value::as_bool) == Some(true) {
                css.push_str("flex:1;");
            }
            box_css(s, ctx.dark, &mut css);
            text_css_inherit(s, ctx.dark, &mut css);
            let _ = write!(out, "<div style=\"{css}\">");
            for c in children(node) {
                html(c, ctx, out);
            }
            out.push_str("</div>");
        }
        "text" | "button" => {
            text_css(s, ctx.dark, &mut css);
            box_css(s, ctx.dark, &mut css);
            let _ = write!(out, "<span style=\"{css}\">{}</span>", text());
        }
        "spacer" => out.push_str("<div style=\"flex:1\"></div>"),
        "progress" => {
            let v = number(node, ctx.data).unwrap_or(0.0).clamp(0.0, 1.0) * 100.0;
            let h = s.get("height").and_then(Value::as_f64).unwrap_or(8.0);
            let fg = pick(s.get("fg"), ctx.dark).unwrap_or("#2563eb");
            let bg = pick(s.get("bg"), ctx.dark).unwrap_or("rgba(127,127,127,.2)");
            let _ = write!(
                out,
                "<div style=\"height:{h}px;border-radius:{h}px;background:{bg};align-self:stretch\"><div style=\"width:{v}%;height:100%;border-radius:{h}px;background:{fg}\"></div></div>"
            );
        }
        "bars" => {
            let vals = values(node, ctx.data);
            let top = vals.iter().cloned().fold(1.0, f64::max);
            let h = s.get("height").and_then(Value::as_f64).unwrap_or(48.0);
            let gap = s.get("gap").and_then(Value::as_f64).unwrap_or(4.0);
            let fg = pick(s.get("fg"), ctx.dark).unwrap_or("#2563eb");
            let _ = write!(
                out,
                "<div style=\"display:flex;align-items:flex-end;gap:{gap}px;height:{h}px;align-self:stretch\">"
            );
            for v in vals {
                let _ = write!(
                    out,
                    "<div style=\"flex:1;height:{}px;border-radius:3px;background:{fg}\"></div>",
                    (h * v / top).max(2.0)
                );
            }
            out.push_str("</div>");
        }
        "icon" => {
            let size = s
                .get("height")
                .or_else(|| s.get("size"))
                .and_then(Value::as_f64)
                .unwrap_or(18.0);
            let fg = pick(s.get("fg"), ctx.dark).unwrap_or("currentColor");
            let name = node.get("name").and_then(Value::as_str).unwrap_or("");
            let _ = write!(
                out,
                "<span style=\"font-size:{size}px;line-height:1;color:{fg}\">{}</span>",
                icon_glyph(name)
            );
        }
        "timer" | "clock" => {
            text_css(s, ctx.dark, &mut css);
            let value = if node["t"] == "timer" {
                timer(node, ctx)
            } else {
                ctx.clock.clone()
            };
            let _ = write!(
                out,
                "<span style=\"{css}font-variant-numeric:tabular-nums\">{value}</span>"
            );
        }
        "toggle" => {
            text_css(s, ctx.dark, &mut css);
            let on = bool_at(node, ctx.data);
            let mark = if on { "✓" } else { "○" };
            let strike = if on {
                "text-decoration:line-through;opacity:.55;"
            } else {
                ""
            };
            let _ = write!(
                out,
                "<div style=\"display:flex;gap:8px;align-items:center\"><span style=\"{css}\">{mark}</span><span style=\"{css}{strike}\">{}</span></div>",
                text()
            );
        }
        "rotate" => {
            if let Some(c) = rotate_child(node, ctx) {
                html(c, ctx, out);
            }
        }
        "list" => {
            let gap = s.get("gap").and_then(Value::as_f64).unwrap_or(0.0);
            let _ = write!(
                out,
                "<div style=\"display:flex;flex-direction:column;gap:{gap}px;min-width:0\">"
            );
            if let Some(item) = children(node).first() {
                for d in list_items(node, ctx.data) {
                    html(
                        item,
                        &Ctx {
                            data: &d,
                            clock: ctx.clock.clone(),
                            ..*ctx
                        },
                        out,
                    );
                }
            }
            out.push_str("</div>");
        }
        _ => {}
    }
}

/// Containers pass their text colour down, as SwiftUI's foreground style does.
fn text_css_inherit(s: &Map<String, Value>, dark: bool, out: &mut String) {
    if let Some(fg) = pick(s.get("fg"), dark) {
        let _ = write!(out, "color:{fg};");
    }
}

/// Point size of a family's frame (an iPhone's): small, medium, large; lock-screen families.
pub fn frame(family: &str) -> (u32, u32) {
    match family {
        "small" => (170, 170),
        "large" | "extraLarge" => (364, 382),
        "circular" => (76, 76),
        "rectangular" => (172, 76),
        "inline" => (234, 26),
        _ => (364, 170),
    }
}

/// One widget, framed: the root's background becomes the frame's, its padding the frame's (16 by default).
pub fn widget_html(snapshot: &Value, ctx: &Ctx) -> String {
    let lock = matches!(ctx.family, "circular" | "rectangular" | "inline");
    let root = if lock {
        snapshot.get("accessory").unwrap_or(&snapshot["root"])
    } else {
        &snapshot["root"]
    };
    let (w, h) = frame(ctx.family);
    let s = style(root);
    let bg = if lock {
        "rgba(0,0,0,.35)".to_owned()
    } else {
        background(s, ctx.dark).unwrap_or_else(|| if ctx.dark { "#1c1c1e".into() } else { "#ffffff".into() })
    };
    let pad = if lock {
        "4px".to_owned()
    } else {
        match s
            .get("pad")
            .and_then(Value::as_array)
            .map(|p| p.iter().filter_map(Value::as_f64).collect::<Vec<_>>())
        {
            Some(n) if n.len() == 4 => format!("{}px {}px {}px {}px", n[0], n[1], n[2], n[3]),
            _ => "16px".to_owned(),
        }
    };
    let radius = if ctx.family == "circular" {
        "50%"
    } else if lock {
        "12px"
    } else {
        "22px"
    };
    let fg = if lock || ctx.dark { "#ffffff" } else { "#000000" };
    let font = snapshot
        .get("font")
        .and_then(Value::as_str)
        .map(|f| format!("'{}',", esc(f)))
        .unwrap_or_default();
    let mut inner = Value::Object(root.as_object().cloned().unwrap_or_default());
    if let Some(st) = inner.get_mut("style").and_then(Value::as_object_mut) {
        for k in ["bg", "gradient", "pad", "radius"] {
            st.remove(k);
        }
    }
    let mut body = String::new();
    html(&inner, ctx, &mut body);
    format!(
        "<div class=\"w\" style=\"width:{w}px;height:{h}px;background:{bg};padding:{pad};border-radius:{radius};color:{fg};font-family:{font}-apple-system,system-ui,sans-serif;\"><div style=\"display:flex;flex-direction:column;height:100%\">{body}</div></div>"
    )
}
