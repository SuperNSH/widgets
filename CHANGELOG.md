# Changelog

## 0.3.0 (2026-11-24)

- GNOME Shell. `nsh-widgets gnome KIND --group NAME [--out DIR] [--command PATH]` writes a GNOME Shell extension: the kind's `accessory` tree in the top bar and its `root` tree in the popup, with buttons, toggles, scrolling lists, timers, and light and dark. It reads through `nsh-widgets show --format ir` and presses through `nsh-widgets apply`, with no shell in between. Verified on GNOME Shell 48.7, nested; 45 to 47 are listed but have not been run.

## 0.2.1 (2026-11-24)

- KDE Plasma: in a panel the widget drew its full form, and once fixed its templates stayed unfilled. It now draws the `accessory` tree, follows its data, and opens the full widget when clicked. Plasmoids written by 0.2.0 need `nsh-widgets plasmoid` run again.

## 0.2.0 (2026-11-24)

- Live Activities on iOS. `widgets.json` `activities` lists the kinds that can run as one. `Widgets.startActivity(kind, data)`, `updateActivity`, `endActivity` in TS; `start_activity`, `update_activity`, `end_activity` in Rust. The lock-screen banner and the expanded Dynamic Island draw the kind's `root` tree; the compact and minimal Dynamic Island draw its `accessory` tree, a row's first child leading and the rest trailing. The activity's data lies over the kind's stored data. One activity per kind: a new start replaces the running one. `build` sets `ios.infoPlist.NSSupportsLiveActivities`. ActivityKit is weak-linked, so apps still run on iOS 15. New error codes `activities_disabled`, `no_activity`, `activity_refused`; other platforms return `unsupported`.
- Widgets the user configures. `widgets[].options` (`title`, 1 to 20 unique `values`, no `|`) gives iOS and macOS an edit sheet (a generated `AppEnum` and `WidgetConfigurationIntent` per kind) and Android a configure screen (`NshWidgetConfigureActivity`), with the choice kept per placed widget. The chosen value's object in the kind's data lies over the data, and `{option}` holds the value.
- `list` node: `List(cls, key, item, max = 8)` draws one `item` tree per object in `data[key]`, with `{item.field}` templates. Android 12+ scrolls it (`RemoteCollectionItems`); older Android, iOS and macOS draw the first `max` items.
- Controls. `widgets.json` `controls` (at most 8): a `toggle` bound to a boolean key of a kind's data, or a `button` that runs an action on it. iOS 18+ gets Control Center and lock-screen controls (`ControlWidgetToggle`, `ControlWidgetButton`); Android gets Quick Settings tiles (`NshTileService$Slot1` to `$Slot8` in the AAR). Controls and widgets redraw together. macOS has none.
- KDE Plasma. `nsh-widgets plasmoid KIND --group NAME [--plasma 5|6] [--out DIR] [--command PATH]` writes a Plasma applet package whose generic QML renderer (`NshNode.qml`) draws the kind from the app's store. It reads through `nsh-widgets show --format ir` and presses buttons through `nsh-widgets apply`.
- New commands and formats: `show --format ir` prints the snapshot and data as JSON; `apply KIND ACTION` runs `inc`, `dec` or `toggle` on a store and queues the event for the app.
- iOS lock-screen circular widgets are centred on the system's `AccessoryWidgetBackground`.
- `examples/gallery`: Agenda (a list), Weather (options), an Order Live Activity, and two controls (Ship it, Count +1).
- Verified on the iOS 27 simulator (lock-screen widgets, the Live Activity, options, the list, controls), an Android 17 emulator (the scrolling list, options, tiles) and KDE Plasma 5.27 and 6.3. macOS rendering of the new features, and physical devices are unverified.

## 0.1.0 (2026-11-24)

First version.

- Widget Tree IR v1: `col`, `row`, `text`, `button`, `spacer`, `progress`, `bars`, `icon`, `timer`, `clock`, `toggle`, `rotate`, with light and dark colours, `{key}` templates filled from data, and the button actions `inc:key`, `dec:key`, `toggle:key` and `open:<url>`.
- `nsh-widgets` crate: the shared store under `<root>/nsh/widgets/`, the Rust API (`configure`, `set_ui`, `set_data`, `update_data`, `data`, `take_actions`, `reload`, `next_change`, `store`), the TSX module `nsh-widgets/native`, the Swift bridge for WidgetKit and App Groups, and the Android AAR with the RemoteViews renderer, 16 provider slots and the action receiver.
- `nsh-widgets` command: `init`, `build`, `check`, `preview`, `show`, `doctor`. `preview` draws every kind as HTML from what the app published (the booted iOS simulator, a store directory, or a snapshot file); `show` prints a widget as text or Waybar JSON, once or `--follow`. `doctor --app DIR` reports signing identities and the provisioning profiles device packages need. `build` compiles the WidgetKit extension for the iOS simulator, iOS devices and macOS with swiftc and the App Intents metadata processor, writes the app's Android widget library, and updates `nsh.json`.
- `nsh-widgets` TypeScript package: base primitives with a Tailwind-subset class resolver and theme tokens, kit components (`Card`, `Header`, `Stat`, `ActionButton`, `LinkButton`, `ButtonRow`, `Gap`), and the `Widgets` API with `WidgetError` codes.
- Installable as an nshm from git: `nsh add github:SuperNSH/widgets@^0.1` brings the Rust module, the AAR and the TSX modules `nsh-widgets`, `nsh-widgets/base`, `nsh-widgets/kit`.
- 21 shared icons; real gradients on Android (bitmaps behind the content); the root's padding applied the same on every platform; iOS lock-screen families (`circular`, `rectangular`, `inline`) with an `accessory` tree; a desktop store on Linux and Windows named by `configure`.
- `examples/gallery`: Counter, Dashboard, Focus timer, Tasks, Steps and Icons.
- Verified on the iOS 27 simulator, an Android 17 (API 37) emulator and a Mac desktop. Lock-screen rendering and physical devices were unverified at this release.
