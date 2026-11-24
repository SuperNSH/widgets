//! What the desktop hosts that run `nsh-widgets` as a program share: the kind, the store it reads and
//! the files they are written as.

use std::path::{Path, PathBuf};

use anyhow::{Context as _, bail};
use serde_json::{Value, json};

use crate::Args;

const COMMON: &str = include_str!("../js/common.js");
pub const ST: &str = include_str!("../js/st.js");
pub const GTK: &str = include_str!("../js/gtk.js");

pub struct Host {
    pub kind: String,
    /// The store's name, for the host's id.
    pub store: String,
    /// The arguments that name the store.
    pub source: Vec<String>,
    /// The `nsh-widgets` to run.
    pub command: String,
    pub title: String,
    pub description: String,
}

impl Host {
    pub fn read(args: &Args, what: &str) -> anyhow::Result<Host> {
        let kind = args
            .kind
            .as_deref()
            .with_context(|| format!("{what} needs a widget kind: nsh-widgets {what} KIND --group NAME"))?;
        if !crate::config::valid_kind(kind) {
            bail!("{kind:?} is not a widget kind");
        }
        let (store, source) = match (&args.group, &args.source.dir) {
            (Some(g), _) => (g.clone(), vec!["--group".into(), g.clone()]),
            (None, Some(d)) => ("local".into(), vec!["--dir".into(), d.to_string_lossy().into_owned()]),
            _ => bail!("{what} needs --group NAME (as the app's Widgets.configure(NAME)) or --dir ROOT"),
        };
        let command = match &args.exe {
            Some(c) => c.clone(),
            None => std::env::current_exe()
                .context("the nsh-widgets path")?
                .to_string_lossy()
                .into_owned(),
        };
        let widgets = crate::config::Config::read(&args.app.join("widgets.json")).ok();
        let w = widgets.as_ref().and_then(|c| c.widgets.iter().find(|w| w.kind == kind));
        let title = w.map(|w| w.name.clone()).unwrap_or_else(|| kind.to_owned());
        let description = w
            .map(|w| w.description.clone())
            .filter(|d| !d.is_empty())
            .unwrap_or_else(|| title.clone());
        Ok(Host {
            kind: kind.to_owned(),
            store,
            source,
            command,
            title,
            description,
        })
    }

    /// The id of the host's package: letters, digits, dots and dashes before the `@`.
    pub fn uuid(&self) -> String {
        let safe = self
            .store
            .replace(|c: char| !c.is_ascii_alphanumeric() && c != '.', "-");
        format!("{safe}.{}@widgets.supernsh.dev", self.kind.to_lowercase())
    }

    /// What the host runs: a program and arguments, never a shell line.
    pub fn config(&self) -> Value {
        json!({ "kind": self.kind, "command": self.command, "source": self.source })
    }

    /// Writes the package to `--out`, or to `.nsh-widgets/<host>/<id>`.
    pub fn write(&self, args: &Args, host: &str, files: &[(&str, String)]) -> anyhow::Result<PathBuf> {
        let out = match &args.out {
            Some(o) => o.clone(),
            None => Path::new(".nsh-widgets").join(host).join(self.uuid()),
        };
        std::fs::create_dir_all(&out)?;
        for (name, text) in files {
            std::fs::write(out.join(name), text)?;
        }
        println!("wrote {}", out.display());
        Ok(out)
    }
}

pub fn pretty(v: &Value) -> String {
    format!("{v:#}\n")
}

/// A script with the shared code and `renderer` in it.
pub fn with_renderer(script: &str, renderer: &str) -> String {
    // Without the line's end: a Windows checkout ends it with CRLF.
    script.replace("//@RENDERER@", &format!("{COMMON}\n{renderer}"))
}
