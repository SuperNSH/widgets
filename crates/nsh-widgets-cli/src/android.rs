//! Writes the app's own Android library (`.nsh-widgets/android/nsh-widgets-app.aar`): a provider slot
//! per kind in its manifest, with the kind as meta-data, and each kind's `appwidget-provider` info.
//! The renderer itself is in the nsh-widgets crate's AAR.

use std::{fmt::Write as _, io::Write as _, path::Path};

use anyhow::Context as _;

use crate::config::{Config, WidgetKind};

pub const AAR: &str = ".nsh-widgets/android/nsh-widgets-app.aar";

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// The manifest: slot `i + 1` draws kind `i`.
pub fn manifest(config: &Config) -> String {
    let mut out = String::from(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<manifest xmlns:android=\"http://schemas.android.com/apk/res/android\" package=\"dev.supernsh.widgets.app\">\n    <uses-sdk android:minSdkVersion=\"26\"/>\n    <application>\n",
    );
    for (i, w) in config.widgets.iter().enumerate() {
        let _ = write!(
            out,
            "        <receiver android:name=\"dev.supernsh.widgets.NshWidgetProvider$Slot{slot}\" android:exported=\"true\" android:label=\"{label}\">\n            <intent-filter><action android:name=\"android.appwidget.action.APPWIDGET_UPDATE\"/></intent-filter>\n            <meta-data android:name=\"android.appwidget.provider\" android:resource=\"@xml/nsh_widgets_kind_{i}\"/>\n            <meta-data android:name=\"dev.supernsh.widgets.kind\" android:value=\"{kind}\"/>\n        </receiver>\n",
            slot = i + 1,
            label = xml_escape(&w.name),
            kind = xml_escape(&w.kind),
        );
    }
    out.push_str("    </application>\n</manifest>\n");
    out
}

/// One kind's `appwidget-provider`: sizes from its cells, resizable, drawn from the shared root layout.
pub fn provider_info(i: usize, w: &WidgetKind) -> String {
    let [cols, rows] = w.cells();
    // Android's sizing formula: 73dp per cell minus 16dp, at least 40dp.
    let dp = |cells: u32| (73 * cells).saturating_sub(16).max(40);
    format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<appwidget-provider xmlns:android=\"http://schemas.android.com/apk/res/android\"\n    android:minWidth=\"{w}dp\" android:minHeight=\"{h}dp\" android:targetCellWidth=\"{cols}\" android:targetCellHeight=\"{rows}\"\n    android:minResizeWidth=\"40dp\" android:minResizeHeight=\"40dp\"\n    android:updatePeriodMillis=\"0\" android:initialLayout=\"@layout/nsh_widgets_root\"\n    android:resizeMode=\"horizontal|vertical\" android:widgetCategory=\"home_screen\"\n    android:description=\"@string/nsh_widgets_kind_{i}_description\"/>\n",
        w = dp(cols),
        h = dp(rows),
    )
}

pub fn strings(config: &Config) -> String {
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<resources>\n");
    for (i, w) in config.widgets.iter().enumerate() {
        let text = if w.description.is_empty() {
            &w.name
        } else {
            &w.description
        };
        let _ = writeln!(
            out,
            "    <string name=\"nsh_widgets_kind_{i}_description\">{}</string>",
            xml_escape(text).replace('\'', "\\'")
        );
    }
    out.push_str("</resources>\n");
    out
}

pub fn r_txt(config: &Config) -> String {
    let mut out = String::new();
    for i in 0..config.widgets.len() {
        let _ = writeln!(out, "int string nsh_widgets_kind_{i}_description 0x7f03{i:04x}");
    }
    for i in 0..config.widgets.len() {
        let _ = writeln!(out, "int xml nsh_widgets_kind_{i} 0x7f04{i:04x}");
    }
    out
}

/// Writes the AAR to `<app>/.nsh-widgets/android/nsh-widgets-app.aar`.
pub fn build(app_dir: &Path, config: &Config) -> anyhow::Result<std::path::PathBuf> {
    let path = app_dir.join(AAR);
    std::fs::create_dir_all(path.parent().expect("parent"))?;
    let file = std::fs::File::create(&path).with_context(|| format!("create {}", path.display()))?;
    let mut zip = zip::ZipWriter::new(file);
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .last_modified_time(zip::DateTime::default());
    let mut add = |name: &str, body: &str| -> anyhow::Result<()> {
        zip.start_file(name, opts)?;
        zip.write_all(body.as_bytes())?;
        Ok(())
    };
    add("AndroidManifest.xml", &manifest(config))?;
    add("R.txt", &r_txt(config))?;
    add("res/values/nsh_widgets_kinds.xml", &strings(config))?;
    for (i, w) in config.widgets.iter().enumerate() {
        add(&format!("res/xml/nsh_widgets_kind_{i}.xml"), &provider_info(i, w))?;
    }
    zip.finish()?;
    Ok(path)
}
