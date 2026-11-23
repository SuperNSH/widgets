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
    let small = apple::bundle_source(&config(3), Platform::Simulator);
    assert!(small.contains("nshUIConfiguration(\"K0\", \"Kind \\\"0\\\"\", \"\", [.systemSmall, .systemLarge])"));
    assert!(small.contains("@main\nstruct NshWidgetsBundle") && !small.contains("NshKindGroup"));
    let big = apple::bundle_source(&config(12), Platform::Simulator);
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

#[test]
fn lock_screen_families_are_ios_only() {
    let c: Config = serde_json::from_value(json!({ "appGroup": "group.x", "widgets": [
        { "kind": "Steps", "name": "Steps", "families": ["rectangular", "small"] },
        { "kind": "Ring", "name": "Ring", "families": ["circular"] }
    ]}))
    .unwrap();
    assert!(c.problems().is_empty());
    let ios = apple::bundle_source(&c, Platform::Device);
    assert!(ios.contains("[.accessoryRectangular, .systemSmall]") && ios.contains("[.accessoryCircular]"));
    let mac = apple::bundle_source(&c, Platform::Mac);
    assert!(mac.contains("[.systemSmall]") && !mac.contains("accessory") && !mac.contains("NshKind1()"));
    assert_eq!(
        c.widgets[0].cells(),
        [2, 2],
        "Android sizes come from the first home-screen family"
    );
}

mod render_tests {
    use serde_json::{Map, Value, json};

    use crate::render::{self, Ctx};

    fn data(v: Value) -> Map<String, Value> {
        v.as_object().unwrap().clone()
    }

    fn ctx<'a>(d: &'a Map<String, Value>, family: &'a str) -> Ctx<'a> {
        Ctx {
            data: d,
            family,
            dark: false,
            now_ms: 10_000_000,
            clock: "09:41".into(),
        }
    }

    #[test]
    fn templates_fill_like_the_renderers() {
        let d = data(json!({ "n": 3.0, "p": 0.5, "s": "hi", "b": true }));
        assert_eq!(render::fill("{n} {p} {s} {b} {missing}", &d), "3 0.5 hi true {missing}");
    }

    #[test]
    fn text_follows_the_tree() {
        let d = data(json!({ "count": 2, "p": 0.3, "week": [1, 4, 2], "t1": true, "at": 9_000_000 }));
        let tree = json!({ "t": "col", "children": [
            { "t": "row", "children": [
                { "t": "icon", "name": "bolt" }, { "t": "text", "text": "TITLE" },
                { "t": "text", "text": "note", "style": { "hide": ["small"] } } ] },
            { "t": "text", "text": "{count}" },
            { "t": "progress", "key": "p" },
            { "t": "bars", "key": "week" },
            { "t": "toggle", "key": "t1", "text": "Ship" },
            { "t": "row", "children": [{ "t": "timer", "key": "at" }, { "t": "clock" }, { "t": "button", "text": "+1", "action": "inc:count" }] },
            { "t": "rotate", "interval": 5, "children": [{ "t": "text", "text": "a" }, { "t": "text", "text": "b" }] }
        ]});
        let medium = render::text_lines(&tree, &ctx(&d, "medium"));
        assert_eq!(
            medium,
            [
                "⚡ TITLE note",
                "2",
                "▓▓▓░░░░░░░ 30%",
                "▃█▅",
                "☑ Ship",
                "16:40 09:41 [+1]",
                "a"
            ]
        );
        assert_eq!(render::text_lines(&tree, &ctx(&d, "small"))[0], "⚡ TITLE");
    }

    #[test]
    fn html_frames_each_family_with_the_root_background() {
        let d = data(json!({ "steps": 9039 }));
        let snap = json!({ "v": 1, "root": { "t": "col", "style": { "gradient": { "from": ["#10b981", "#047857"], "to": ["#2563eb", "#1e40af"], "dir": "br" }, "pad": [8, 8, 8, 8] },
            "children": [{ "t": "text", "text": "{steps}", "style": { "fg": ["#ffffff", "#ffffff"], "size": 30, "weight": "bold" } }] },
            "accessory": { "t": "text", "text": "{steps} steps" } });
        let small = render::widget_html(&snap, &ctx(&d, "small"));
        assert!(
            small.contains("width:170px;height:170px")
                && small.contains("linear-gradient(to bottom right, #10b981, #2563eb)")
        );
        assert!(
            small.contains("padding:8px 8px 8px 8px") && small.contains(">9039<") && small.contains("font-weight:700")
        );
        let lock = render::widget_html(&snap, &ctx(&d, "rectangular"));
        assert!(lock.contains("9039 steps") && !lock.contains("linear-gradient"));
    }

    #[test]
    fn waybar_puts_the_lock_screen_tree_on_the_bar() {
        let w = crate::source::Widget {
            kind: "Steps".into(),
            snapshot: json!({ "v": 1, "root": { "t": "col", "children": [{ "t": "text", "text": "STEPS" }, { "t": "text", "text": "{steps}" }] },
                "accessory": { "t": "text", "text": "{steps} steps" } }),
            data: data(json!({ "steps": 12 })),
            families: vec![],
        };
        let out: Value =
            serde_json::from_str(&crate::source::show_text(&w, "waybar", 0, "09:41".into()).unwrap()).unwrap();
        assert_eq!(
            out,
            json!({ "text": "12 steps", "tooltip": "STEPS\n12", "class": "Steps" })
        );
        assert_eq!(
            crate::source::show_text(&w, "text", 0, String::new()).unwrap(),
            "STEPS\n12"
        );
        assert!(crate::source::show_text(&w, "xml", 0, String::new()).is_err());
    }
}
