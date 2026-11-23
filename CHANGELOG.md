# Changelog

## 0.1.0 (2026-11-24)

First version.

- Widget Tree IR v1: `col`, `row`, `text`, `button`, `spacer`, `progress`, `bars`, `icon`, `timer`, `clock`, `toggle`, `rotate`, with light and dark colours, `{key}` templates filled from data, and the button actions `inc:key`, `dec:key`, `toggle:key` and `open:<url>`.
- `nsh-widgets` crate: the shared store under `<root>/nsh/widgets/`, the Rust API (`configure`, `set_ui`, `set_data`, `update_data`, `data`, `take_actions`, `reload`, `next_change`, `store`), the TSX module `nsh-widgets/native`, the Swift bridge for WidgetKit and App Groups, and the Android AAR with the RemoteViews renderer, 16 provider slots and the action receiver.
- `nsh-widgets` command: `init`, `build`, `check`, `preview`, `show`, `doctor`. `preview` draws every kind as HTML from what the app published (the booted iOS simulator, a store directory, or a snapshot file); `show` prints a widget as text or Waybar JSON, once or `--follow`. `doctor --app DIR` reports signing identities and the provisioning profiles device packages need. `build` compiles the WidgetKit extension for the iOS simulator, iOS devices and macOS with swiftc and the App Intents metadata processor, writes the app's Android widget library, and updates `nsh.json`.
- `nsh-widgets` TypeScript package: base primitives with a Tailwind-subset class resolver and theme tokens, kit components (`Card`, `Header`, `Stat`, `ActionButton`, `LinkButton`, `ButtonRow`, `Gap`), and the `Widgets` API with `WidgetError` codes.
- Installable as an nshm from git: `nsh add github:SuperNSH/widgets@^0.1` brings the Rust module, the AAR and the TSX modules `nsh-widgets`, `nsh-widgets/base`, `nsh-widgets/kit`.
- 21 shared icons; real gradients on Android (bitmaps behind the content); the root's padding applied the same on every platform; iOS lock-screen families (`circular`, `rectangular`, `inline`) with an `accessory` tree; a desktop store on Linux and Windows named by `configure`.
- `examples/gallery`: Counter, Dashboard, Focus timer, Tasks, Steps and Icons.
- Verified on the iOS 27 simulator, an Android 17 (API 37) emulator and a Mac desktop. Lock-screen rendering and physical devices are unverified.
