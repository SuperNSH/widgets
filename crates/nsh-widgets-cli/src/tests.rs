use serde_json::json;

use crate::{
    android,
    apple::{self, Platform},
    config::{App, Config},
    nshjson,
};

fn config(kinds: usize) -> Config {
    let widgets: Vec<_> = (0..kinds)
        .map(|i| json!({ "kind": format!("K{i}"), "name": format!("Kind \"{i}\""), "families": ["small", "large"] }))
        .collect();
    serde_json::from_value(json!({ "appGroup": "group.dev.example.app", "widgets": widgets })).unwrap()
}

fn app() -> App {
    App {
        id: "dev.example.app".into(),
        name: "Example".into(),
        version: "1.2.0".into(),
        build: "7".into(),
    }
}

#[test]
fn config_problems_are_listed() {
    let bad: Config = serde_json::from_value(json!({
        "appGroup": "dev.example",
        "fonts": ["a.woff"],
        "widgets": [
            { "kind": "1x", "name": "", "families": ["tiny"], "android": { "cells": [9, 1] } },
            { "kind": "1x", "name": "b", "families": [] }
        ]
    }))
    .unwrap();
    let problems = bad.problems().join("\n");
    for expected in [
        "must start with \"group.\"",
        "kind \"1x\": a letter",
        "appears twice",
        "has no name",
        "unknown family \"tiny\"",
        "has no families",
        "android.cells",
        "only .ttf and .otf",
    ] {
        assert!(problems.contains(expected), "missing {expected:?} in\n{problems}");
    }
    assert!(config(3).problems().is_empty());
    assert!(config(17).problems().iter().any(|p| p.contains("at most 16")));
}

#[test]
fn unknown_config_keys_are_errors() {
    let r: Result<Config, _> = serde_json::from_value(json!({ "appGroup": "group.x", "widgets": [], "colour": 1 }));
    assert!(r.is_err());
}

#[test]
fn the_bundle_names_every_kind_and_groups_by_ten() {
    let small = apple::bundle_source(&config(3));
    assert!(small.contains("nshUIConfiguration(\"K0\", \"Kind \\\"0\\\"\", \"\", [.systemSmall, .systemLarge])"));
    assert!(small.contains("@main\nstruct NshWidgetsBundle") && !small.contains("NshKindGroup"));
    let big = apple::bundle_source(&config(12));
    assert!(big.contains("struct NshKindGroup0") && big.contains("NshKindGroup1().body"));
    assert_eq!(big.matches("NshKind11()").count(), 1);
}

#[test]
fn info_plists_carry_the_group_and_fonts() {
    let mut c = config(1);
    c.macos_app_group = Some("TEAM.dev.example.app".into());
    let ios = apple::info_plist(Platform::Simulator, &app(), &c, &["Onest.ttf".into()]);
    assert_eq!(ios["NshWidgetsAppGroup"], "group.dev.example.app");
    assert_eq!(ios["CFBundleIdentifier"], "dev.example.app.widgets");
    assert_eq!(ios["CFBundleSupportedPlatforms"], json!(["iPhoneSimulator"]));
    assert_eq!(ios["UIAppFonts"], json!(["Onest.ttf"]));
    assert_eq!(ios["CFBundleVersion"], "7");
    let mac = apple::info_plist(Platform::Mac, &app(), &c, &[]);
    assert_eq!(mac["NshWidgetsAppGroup"], "TEAM.dev.example.app");
    assert!(mac.get("ATSApplicationFontsPath").is_none());
    let xml = apple::plist(&json!({ "a": "<&>", "b": true, "c": [1, 2.5] }));
    assert!(
        xml.contains("<string>&lt;&amp;&gt;</string>") && xml.contains("<true/>") && xml.contains("<real>2.5</real>")
    );
}

#[test]
fn the_android_manifest_gives_each_kind_a_slot() {
    let c = config(2);
    let manifest = android::manifest(&c);
    assert!(manifest.contains("NshWidgetProvider$Slot1") && manifest.contains("NshWidgetProvider$Slot2"));
    assert!(manifest.contains("android:value=\"K1\"") && manifest.contains("@xml/nsh_widgets_kind_1"));
    assert!(manifest.contains("android:label=\"Kind &quot;0&quot;\""));
    assert!(android::provider_info(0, &c.widgets[0]).contains("android:targetCellWidth=\"2\""));
    assert!(android::strings(&c).contains("nsh_widgets_kind_1_description"));
}

#[test]
fn the_android_library_is_a_valid_aar() {
    let dir = tempfile::tempdir().unwrap();
    let path = android::build(dir.path(), &config(2)).unwrap();
    let mut zip = zip::ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap();
    let mut names: Vec<_> = zip.file_names().map(str::to_owned).collect();
    names.sort();
    assert_eq!(
        names,
        [
            "AndroidManifest.xml",
            "R.txt",
            "res/values/nsh_widgets_kinds.xml",
            "res/xml/nsh_widgets_kind_0.xml",
            "res/xml/nsh_widgets_kind_1.xml"
        ]
    );
    assert!(zip.by_name("R.txt").is_ok());
}

#[test]
fn nsh_json_keeps_the_apps_keys_and_replaces_only_ours() {
    let mut nsh = json!({
        "id": "dev.example.app",
        "ios": {
            "entitlements": { "com.apple.security.application-groups": ["group.other"] },
            "extensions": [{ "path": "share/Share.appex" }, { "path": "old/NshWidgets.appex" }]
        },
        "android": { "libraries": ["libs/x.aar"] }
    });
    let all = nshjson::Built {
        ios: true,
        macos: true,
        android: true,
    };
    nshjson::apply(&mut nsh, &config(1), all);
    let once = nsh.clone();
    nshjson::apply(&mut nsh, &config(1), all);
    assert_eq!(nsh, once, "applying twice changes nothing");
    assert_eq!(
        nsh["ios"]["entitlements"]["com.apple.security.application-groups"],
        json!(["group.other", "group.dev.example.app"])
    );
    let paths: Vec<_> = nsh["ios"]["extensions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["path"].as_str().unwrap())
        .collect();
    assert_eq!(paths, ["share/Share.appex", ".nsh-widgets/iphoneos/NshWidgets.appex"]);
    assert_eq!(
        nsh["macos"]["extensions"][0]["entitlements"]["com.apple.security.app-sandbox"],
        true
    );
    assert_eq!(
        nsh["android"]["libraries"],
        json!(["libs/x.aar", ".nsh-widgets/android/nsh-widgets-app.aar"])
    );
    let mut only_android = json!({ "id": "x" });
    nshjson::apply(
        &mut only_android,
        &config(1),
        nshjson::Built {
            android: true,
            ..Default::default()
        },
    );
    assert!(only_android.get("ios").is_none());
}
