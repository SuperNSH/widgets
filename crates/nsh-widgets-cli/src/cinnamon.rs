//! `nsh-widgets cinnamon`: a Cinnamon desklet for one kind, drawing its `root` tree on the desktop.

use serde_json::json;

use crate::{Args, host};

pub fn write(args: &Args) -> anyhow::Result<()> {
    let h = host::Host::read(args, "cinnamon")?;
    let id = h.uuid();
    let metadata = json!({
        "uuid": id,
        "name": h.title,
        "description": h.description,
        // The card draws its own frame.
        "prevent-decorations": true,
    });
    let js = host::with_renderer(include_str!("../cinnamon/desklet.js"));
    let out = h.write(args, "cinnamon", metadata, ("desklet.js", &js))?;
    let dir = format!("~/.local/share/cinnamon/desklets/{id}");
    println!(
        "install: mkdir -p {dir} && cp -r {}/. {dir}",
        crate::plasma::shell_word(&out.to_string_lossy())
    );
    println!("then add it from System Settings > Desklets");
    Ok(())
}
