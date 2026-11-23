//! `doctor`'s signing report: what device and Mac packages need beyond the simulator. Notes, not
//! failures, since simulator and Android builds need none of it.

use std::{path::Path, process::Command};

use serde_json::Value;

use crate::config::{App, Config};

/// A provisioning profile installed on this Mac.
#[derive(Debug)]
pub struct Profile {
    pub name: String,
    pub expires: String,
    /// `TEAMID.bundle.id`, possibly with a `*` wildcard.
    pub app_id: String,
    pub groups: Vec<String>,
}

fn profiles() -> Vec<Profile> {
    let Some(home) = std::env::var_os("HOME") else {
        return Vec::new();
    };
    let home = Path::new(&home);
    let dirs = [
        home.join("Library/MobileDevice/Provisioning Profiles"),
        home.join("Library/Developer/Xcode/UserData/Provisioning Profiles"),
    ];
    let mut out = Vec::new();
    for file in dirs.iter().flat_map(std::fs::read_dir).flatten().flatten() {
        let path = file.path();
        let Ok(decoded) = Command::new("security").args(["cms", "-D", "-i"]).arg(&path).output() else {
            continue;
        };
        let field = |key: &str, format: &str| {
            let mut child = Command::new("plutil")
                .args(["-extract", key, format, "-o", "-", "-"])
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn()
                .ok()?;
            use std::io::Write as _;
            child.stdin.take()?.write_all(&decoded.stdout).ok()?;
            let o = child.wait_with_output().ok()?;
            o.status
                .success()
                .then(|| String::from_utf8_lossy(&o.stdout).trim().to_owned())
        };
        let entitlements: Value = field("Entitlements", "json")
            .and_then(|j| serde_json::from_str(&j).ok())
            .unwrap_or_default();
        let app_id = entitlements["application-identifier"]
            .as_str()
            .or_else(|| entitlements["com.apple.application-identifier"].as_str())
            .unwrap_or("")
            .to_owned();
        let groups = entitlements["com.apple.security.application-groups"]
            .as_array()
            .map(|a| a.iter().filter_map(Value::as_str).map(str::to_owned).collect())
            .unwrap_or_default();
        out.push(Profile {
            name: field("Name", "raw").unwrap_or_default(),
            expires: field("ExpirationDate", "raw").unwrap_or_default(),
            app_id,
            groups,
        });
    }
    out
}

/// Whether a profile's application identifier covers `bundle`.
pub fn covers(app_id: &str, bundle: &str) -> bool {
    let Some((_, id)) = app_id.split_once('.') else {
        return false;
    };
    match id.strip_suffix('*') {
        Some(prefix) => bundle.starts_with(prefix),
        None => id == bundle,
    }
}

pub fn report(app: &Path) {
    let identities = Command::new("security")
        .args(["find-identity", "-v", "-p", "codesigning"])
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter(|l| l.contains(") "))
                .count()
        })
        .unwrap_or(0);
    if identities == 0 {
        println!(
            "note no code signing identity: iOS device and signed Mac packages need one (a free Personal Team works; add it in Xcode > Settings > Accounts)"
        );
    } else {
        println!("ok   code signing identities: {identities}");
    }
    let (Ok(info), Ok(config)) = (
        App::read(&app.join("nsh.json")),
        Config::read(&app.join("widgets.json")),
    ) else {
        println!(
            "--   pass --app DIR (with nsh.json and widgets.json) to check provisioning profiles for its extension"
        );
        return;
    };
    let profiles = profiles();
    let extension = format!("{}.widgets", info.id);
    for (what, bundle) in [("app", info.id.as_str()), ("widget extension", extension.as_str())] {
        match profiles.iter().find(|p| covers(&p.app_id, bundle)) {
            Some(p) => {
                let group = if p.groups.iter().any(|g| g == &config.app_group || g.ends_with('*')) {
                    "with"
                } else {
                    "WITHOUT"
                };
                println!(
                    "ok   {what} {bundle}: profile \"{}\" (expires {}), {group} the App Group {}",
                    p.name, p.expires, config.app_group
                );
            }
            None => println!(
                "note {what} {bundle}: no provisioning profile. iOS device packages need one for the app and one for the extension, each with the App Group {}. Create them in Xcode or the developer portal; the simulator needs none",
                config.app_group
            ),
        }
    }
}
