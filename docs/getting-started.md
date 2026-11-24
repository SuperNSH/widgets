# Getting started

This adds a home-screen widget to an NSH TSX app. The steps match [`examples/gallery`](../examples/gallery), which is the working reference.

Before you start, check [Requirements](../README.md#requirements): the NSH 0.6.0 and 0.6.1 releases cannot package widgets; you need NSH main at or after `5aa56b19`, or the nightly.

## 1. Add the library

In the app's directory:

```sh
nsh add github:SuperNSH/widgets@^0.3
```

`nsh.json` then lists it, and `nsh.lock` pins the version:

```json
"dependencies": {
  "nsh-widgets": { "github": "SuperNSH/widgets", "version": "^0.2" }
}
```

To work on the library itself, point at a checkout instead: `{ "path": "../widgets" }` (the gallery uses `"../.."`).

The repository root is the nshm. It brings the TSX modules `nsh-widgets`, `nsh-widgets/base` and `nsh-widgets/kit` (its `nsh.json` `exports`), the native module they call (`nsh-widgets/native`), the Swift bridge for Apple targets, and the Android AAR (renderer, layouts, icons, provider slots, action receiver, configure screen, Quick Settings tile slots).

## 2. Install the CLI

Download `nsh-widgets` for your machine from the [releases](https://github.com/SuperNSH/widgets/releases), or build it:

```sh
cargo install --git https://github.com/SuperNSH/widgets nsh-widgets-cli
nsh-widgets doctor
```

`doctor` checks Xcode, swiftc, `appintentsmetadataprocessor`, the iOS simulator, iOS and macOS SDKs, and `nsh`. With `--app DIR` it also reports your code signing identities and whether a provisioning profile covers the app and its widget extension (`<id>.widgets`) with the App Group, which iOS device packages need; the simulator needs none. On a host other than macOS it checks only `nsh`; the Apple extensions build only on macOS.

## 3. Write widgets.json

```sh
nsh-widgets init                 # App Group defaults to group.<nsh.json id>
nsh-widgets init --group group.com.example.app
```

`init` writes a starter `widgets.json` beside `nsh.json` with one kind, `Counter`. The start of the gallery's ([full file](../examples/gallery/widgets.json)):

```json
{
  "appGroup": "group.dev.supernsh.widgets.gallery",
  "widgets": [
    { "kind": "Counter", "name": "Counter", "description": "A number the app sets and the widget's buttons change.", "families": ["small", "medium"] },
    { "kind": "Dashboard", "name": "Dashboard", "description": "Progress, a bar chart and icons.", "families": ["large"], "android": { "cells": [4, 3] } },
    { "kind": "Live", "name": "Focus timer", "description": "A stopwatch, a countdown, a clock and rotating tips that move without the app.", "families": ["medium", "small"] },
    { "kind": "Tasks", "name": "Tasks", "description": "Switches that work without opening the app.", "families": ["medium", "large"], "android": { "cells": [4, 3] } }
  ]
}
```

Fields (from [`config.rs`](../crates/nsh-widgets-cli/src/config.rs); unknown fields are an error):

| Field | Required | Meaning |
|---|---|---|
| `appGroup` | yes | The App Group the app and its extension share. Must start with `group.`. |
| `macosAppGroup` | no | The macOS App Group when it differs. A Mac app signed with a Personal Team needs a group that starts with the team ID. Defaults to `appGroup`. |
| `fonts` | no | `.ttf` or `.otf` files, relative to the app directory, bundled into the extension. A snapshot's `font` names one by family. Android ignores them. |
| `deploymentTarget` | no | `{ "ios": "17.0", "macos": "14.0" }` are the defaults. |
| `widgets` | yes | 1 to 16 kinds. |
| `widgets[].kind` | yes | A letter, then letters, digits, `_` or `-`; at most 64 characters; unique. It names the store directory, the WidgetKit kind and the Android slot. |
| `widgets[].name` | yes | The name in the widget gallery or picker. |
| `widgets[].description` | no | The gallery description. Android falls back to `name`. |
| `widgets[].families` | no | `small`, `medium`, `large`, `extraLarge` (iPad and Mac), and for the iOS lock screen `circular`, `rectangular`, `inline`. Default `["small", "medium"]`. macOS and Android leave the lock-screen ones out. |
| `widgets[].android.cells` | no | `[columns, rows]`, 1 to 5 each. Default from the first home-screen family: small 2x2, medium 4x2, large 4x4. |
| `widgets[].options` | no | `{ "title", "values" }`: a choice the user makes per placed widget. 1 to 20 values, unique, not empty, without `\|`. See [Options](#options). |
| `activities` | no | Kinds that can run as iOS Live Activities. See [Live Activities](#live-activities). |
| `controls` | no | At most 8 Control Center controls (iOS 18) and Quick Settings tiles (Android). See [Controls](#controls). |

`widgets.json` sits beside `nsh.json` instead of inside it because NSH warns about keys it does not know.

## 4. Write the widget

A widget is a function that returns a tree. The gallery's counter, [`widgets/counter.ts`](../examples/gallery/widgets/counter.ts):

```ts
// The counter, written with kit components. No platform code here.
import { setTheme } from "nsh-widgets/base";
import { ActionButton, ButtonRow, Card, Gap, Header, LinkButton, Stat, kitTheme } from "nsh-widgets/kit";

export function counterWidget(platform: string) {
  setTheme(kitTheme);
  return Card(
    Header("NSH WIDGETS", platform),
    Gap(),
    Stat("{count}", "{label}"),
    Gap(),
    ButtonRow(
      [ActionButton("+1", "inc:count"), ActionButton("−1", "dec:count", { variant: "secondary" })],
      LinkButton("Open", "nshwidgets://widget/Counter", { hide: "small:hidden" }),
    ),
  );
}
```

For the iOS lock screen, pass a smaller tree as `accessory` (the gallery's Steps widget):

```ts
Widgets.setUi("Steps", stepsWidget(), {
  accessory: Row("gap-1 items-center", Icon("h-3", "bolt"), Text("text-sm font-semibold", "{steps}"), Text("text-xs circular:hidden", "steps")),
});
```

`{count}` and `{label}` are filled from the kind's data when the widget draws. `inc:count` runs in the widget process, so the button works with the app closed. [ir.md](ir.md) lists every primitive, class and action; [capabilities.md](capabilities.md) says how each one looks on each platform.

## 5. Publish the tree and feed data

From the gallery's [`App.tsx`](../examples/gallery/App.tsx):

```ts
import { Widgets } from "nsh-widgets";

function publish() {
  Widgets.setUi("Counter", counterWidget(here), { url: "nshwidgets://widget/Counter" });
  // ...the other kinds
  if (Widgets.data("Counter").count === undefined) Widgets.update("Counter", { count: 0, label: "tap +1 in the widget" });
}
```

A button in the app:

```ts
const bump = (by: number) => {
  const c = Widgets.data("Counter").count;
  const n = (typeof c === "number" ? c : 0) + by;
  Widgets.update("Counter", { count: n, label: `set in the ${here} app` });
  count.set(n);
};
```

Hearing widget presses, live and on return to the foreground:

```ts
Widgets.onChange("Counter", (d) => {
  if (typeof d.count === "number") count.set(d.count);
  refresh();
});
effect(() => {
  if (lifecycle() === "active") refresh();
});
```

where `refresh` calls `Widgets.data("Counter")` and `Widgets.takeActions()`.

The calls:

| Call | Does |
|---|---|
| `Widgets.setUi(kind, root, { url?, accessory? })` | Writes the kind's snapshot and redraws. `url` opens when the widget itself is tapped; `accessory` is the iOS lock-screen tree. |
| `Widgets.setData(kind, data)` | Replaces the data (a JSON object) and redraws. |
| `Widgets.update(kind, patch)` | Merges top-level keys into the data, keeps keys the widget changed that the patch does not name, redraws, returns the data. |
| `Widgets.data(kind)` | The data as the widget last left it, `{}` before anything wrote it. |
| `Widgets.takeActions()` | Button presses since the last call, oldest first, each returned once: `{ id, kind, action, at }`. |
| `Widgets.reload(kind)` | Redraws placed widgets of the kind. |
| `Widgets.onChange(kind, listener)` | Calls `listener` with the data each time a widget button changes it. Returns a stop function. Stops by itself where there is no widget host. |
| `Widgets.configure(group)` | Uses another App Group on iOS and macOS. On Linux and Windows it names the store `nsh-widgets show --group` and the KDE plasmoid read. Android ignores it. |
| `Widgets.startActivity(kind, data?)` | Starts `kind` as an iOS Live Activity, replacing a running one of the same kind. |
| `Widgets.updateActivity(kind, data)` | Replaces the running activity's data. |
| `Widgets.endActivity(kind)` | Ends the running activity, if any. |

Calls throw `WidgetError` with a stable `code`: `invalid_kind`, `invalid_action`, `invalid_json`, `no_app_group`, `no_group_configured`, `io`, `unsupported`, and for Live Activities `activities_disabled`, `no_activity`, `activity_refused`.

The Rust API is the same set: `configure`, `set_ui`, `set_data`, `update_data`, `data`, `take_actions`, `reload`, `start_activity`, `update_activity`, `end_activity`, `next_change` (async), and `store` for direct access.

## 6. Build and package

```sh
nsh-widgets build                       # every platform this host can build
nsh-widgets build --only android        # repeatable: --only ios --only macos
nsh-widgets check                       # widgets.json is valid and nsh.json is up to date
nsh package --target ios-simulator      # or ios, android, macos
```

`build` reads `widgets.json` and `nsh.json`, then:

- On macOS, builds `.nsh-widgets/iphonesimulator/NshWidgets.appex`, `.nsh-widgets/iphoneos/NshWidgets.appex` and `.nsh-widgets/macosx/NshWidgets.appex` with `swiftc` (no Xcode project), including App Intents metadata.
- Writes `.nsh-widgets/android/nsh-widgets-app.aar`: one provider slot per kind, its `appwidget-provider` XML and description string, the option values of kinds that have them, and one tile service per control.
- Updates `nsh.json`, keeping your own keys:
  - `ios.entitlements` gets the App Group, `ios.infoPlist.NshWidgetsAppGroup` is set, `ios.infoPlist.NSSupportsLiveActivities` is set when `activities` is not empty (and removed when it is), and `ios.extensions` gets the extension (`path`, `simulatorPath`, its entitlements).
  - `macos` gets the same with the macOS group; the extension's entitlements add `com.apple.security.app-sandbox`.
  - `android.libraries` gets the app's AAR.

Run `build` again after editing `widgets.json`. Changing the widget tree or data needs no rebuild; the app writes those at run time.

`.nsh-widgets/` is build output; keep it out of version control.

## 7. Check a layout without a device

```sh
nsh-widgets preview --simulator                      # what the app last published in the booted iOS simulator
nsh-widgets preview --snapshot s.json --data d.json  # one snapshot, anywhere
nsh-widgets show Counter --simulator                 # as text
```

`preview` writes `.nsh-widgets/preview.html`: every kind in each of its sizes, light and dark, drawn from the IR with the app's data ([example](evidence/preview/gallery-preview.png)). The native renderers stay the reference; Android's differences are in [capabilities.md](capabilities.md).

## Live Activities

iOS 16.2 and later. List the kinds in `widgets.json` and run `build`:

```json
"activities": ["Order"]
```

An activity kind needs no entry in `widgets`; it needs a snapshot. The gallery's Order ([`widgets/demos.ts`](../examples/gallery/widgets/demos.ts), [`App.tsx`](../examples/gallery/App.tsx)):

```ts
Widgets.setUi("Order", orderActivity(), { accessory: orderAccessory() });

Widgets.startActivity("Order", { order: "#1042", status: "Confirmed", progress: 0.25, eta: Date.now() + 20 * 60 * 1000 });
Widgets.updateActivity("Order", { status: "On the way", progress: 0.75, eta: Date.now() + 10 * 60 * 1000 });
Widgets.endActivity("Order");
```

- The lock-screen banner and the expanded Dynamic Island draw the `root` tree. The banner uses the root's background or gradient and padding; the expanded island draws on black.
- The compact and minimal Dynamic Island draw the `accessory` tree (or `root` without one). When it is a `row` with two or more children, the first child goes on the leading side and the rest on the trailing side; minimal shows the leading child.
- The data drawn is the activity's data laid over the kind's stored data.
- One activity per kind: `startActivity` ends a running one of the same kind first.
- iOS asks the user once whether to allow Live Activities from the app. If they are turned off, `startActivity` throws `activities_disabled`. `updateActivity` with none running throws `no_activity`. A kind missing from `activities`, or a build without them, makes ActivityKit refuse: `activity_refused`. On every other platform the calls throw `unsupported`.
- ActivityKit is weak-linked into the app, so an app that runs on iOS 15 still launches there.

## Options

A kind with `options` lets the user pick one of a fixed set of values per placed widget:

```json
{ "kind": "Weather", "name": "Weather", "families": ["small", "medium"],
  "options": { "title": "City", "values": ["Karachi", "London", "Tokyo"] } }
```

The data holds one object per value; the chosen value's object is laid over the data and `{option}` holds the value:

```ts
Widgets.update("Weather", {
  Karachi: { temp: 31, cond: "Sunny and humid" },
  London: { temp: 9, cond: "Light rain" },
  Tokyo: { temp: 16, cond: "Clear" },
});
// in the tree: Text("...", "{option}"), Text("...", "{temp}°"), Text("...", "{cond}")
```

- iOS and macOS: `build` generates an `AppEnum` of the values and a `WidgetConfigurationIntent` for the kind. The widget's edit sheet shows the title and the values; the first value is the default.
- Android: the kind's provider names `NshWidgetConfigureActivity` as its configure screen, with `widgetFeatures="reconfigurable|configuration_optional"`. On Android 12+ the widget is placed with the first value, and the launcher's "Settings" on the widget opens the list. The choice is stored per placed widget.

## Lists

```ts
List("gap-2 grow", "events", Row("gap-2", Text("text-xs", "{item.time}"), Text("text-sm", "{item.title}")), 12)
```

draws the item tree once per object in `data.events`, filling `{item.field}` from that object (the gallery's Agenda). The last argument is `max`, 8 by default. Android 12+ draws a real scrolling list of every item; one list per widget scrolls, and later lists in the same widget are drawn static. Older Android, iOS and macOS draw the first `max` items, since WidgetKit cannot scroll. KDE scrolls through the first `max`. Items are for reading: buttons inside an item are not supported. [ir.md](ir.md#list) has the details.

## Controls

Controls are small buttons and switches outside the widget: iOS 18 Control Center and lock-screen controls, and Android Quick Settings tiles. Each reads and changes one kind's data. The gallery's:

```json
"controls": [
  { "kind": "FocusToggle", "name": "Ship it", "type": "toggle", "data": "Tasks", "key": "t1", "icon": "check" },
  { "kind": "CountUp", "name": "Count +1", "type": "button", "data": "Counter", "action": "inc:count", "icon": "plus" }
]
```

| Field | Meaning |
|---|---|
| `kind` | The control's own ID: a letter, then letters, digits, `_` or `-`; unique among controls. |
| `name` | The label in the controls gallery and on the tile. |
| `type` | `toggle` or `button`. |
| `data` | The widget kind whose data it reads and changes. |
| `key` | `toggle` only: the boolean key it shows and flips. |
| `action` | `button` only: `inc:key`, `dec:key` or `toggle:key`. |
| `icon` | A name from the shared icon set; `star` on Android when absent. |

A press writes `data.json`, appends an event to `actions.jsonl` like a widget button, and redraws the kind's widgets. On iOS the app's `setData` and `update` redraw the controls too; Android tiles read the data each time the Quick Settings panel opens.

- iOS 18: a `toggle` is a `ControlWidgetToggle` run by a `SetValueIntent`, a `button` a `ControlWidgetButton` run by the widgets' action intent. They appear under the app in the Control Center gallery.
- Android: each control gets a tile slot, `NshTileService$Slot1` to `$Slot8`, with its data kind, key or action and icon in the app's manifest. Users add the tiles from the Quick Settings editor.
- macOS: no controls.

## Linux and Windows

The app keeps a store on the desktop once it names it. KDE Plasma can draw it with a generated plasmoid ([below](#kde-plasma)) and GNOME Shell with a generated extension ([below](#gnome-shell)); Windows has no widget host for it, and `nsh-widgets show` prints it instead.

Naming the store:

```ts
if (platform() === "linux") Widgets.configure("dev.example.app");
```

It lives in `$XDG_DATA_HOME/nsh-widgets/<name>` (or `~/.local/share/...`; `%LOCALAPPDATA%` on Windows), or in `NSH_WIDGETS_DIR` when that is set. `nsh-widgets show` prints it:

```sh
nsh-widgets show Steps --group dev.example.app --follow                  # redraws once a second in a terminal
nsh-widgets show Steps --group dev.example.app --follow --format waybar  # one JSON object per change
```

In Waybar's config:

```json
"custom/steps": {
  "exec": "nsh-widgets show Steps --group dev.example.app --follow --format waybar",
  "return-type": "json"
}
```

With `--format waybar` the bar text comes from the `accessory` tree (the lock-screen tree), and the tooltip holds the full widget. `--format ir` prints `{ "snapshot", "data", "now" }` as JSON, for other hosts to draw.

`nsh-widgets apply KIND ACTION --group NAME` runs `inc:key`, `dec:key` or `toggle:key` on the store and queues the event for the app, as a widget button does.

## KDE Plasma

`nsh-widgets plasmoid` writes a Plasma applet package for one kind:

```sh
nsh-widgets plasmoid Dashboard --group dev.example.app               # Plasma 6
nsh-widgets plasmoid Dashboard --group dev.example.app --plasma 5
kpackagetool6 -t Plasma/Applet -i .nsh-widgets/plasma/dev.supernsh.widgets.dev.example.app.dashboard
```

Then add it from the desktop's or panel's widget list. Options:

| Option | Meaning |
|---|---|
| `--group NAME` | The store, as the app's `Widgets.configure(NAME)` named it. `--dir ROOT` instead names a store root. |
| `--plasma 5\|6` | The Plasma version. Default 6. |
| `--out DIR` | Where to write the package. Default `.nsh-widgets/plasma/<id>`, where the id is `dev.supernsh.widgets.<group>.<kind in lower case>`. |
| `--command PATH` | The `nsh-widgets` the plasmoid runs. Default: the path of the one that wrote it. |
| `--app DIR` | Where `widgets.json` is, for the widget's name and description. Optional. |

The package holds `metadata.json`, `contents/ui/main.qml` and `contents/ui/NshNode.qml`, a generic QML renderer of the IR. Every two seconds it runs `nsh-widgets show KIND --group NAME --format ir` through Plasma's executable data engine and redraws when something changed; a button press runs `nsh-widgets apply`. On the desktop it draws the `root` tree; in a panel it draws the `accessory` tree and opens the full widget when clicked. Light or dark follows the Plasma theme.

The app must call `Widgets.configure(NAME)` on Linux so its store is `$XDG_DATA_HOME/nsh-widgets/NAME`, and the machine needs the `nsh-widgets` binary.

## GNOME Shell

GNOME has no desktop widgets, so a kind becomes a top-bar button with a popup. `nsh-widgets gnome` writes a GNOME Shell extension for one kind:

```sh
nsh-widgets gnome Dashboard --group dev.example.app --out dashboard
```

It prints the two steps that follow: copying the directory to `~/.local/share/gnome-shell/extensions/<id>`, and, after logging out and in, `gnome-extensions enable <id>`. Options:

| Option | Meaning |
|---|---|
| `--group NAME` | The store, as the app's `Widgets.configure(NAME)` named it. `--dir ROOT` instead names a store root. |
| `--out DIR` | Where to write the extension. Default `.nsh-widgets/gnome/<id>`, where the id is `<group>.<kind in lower case>@widgets.supernsh.dev`. |
| `--command PATH` | The `nsh-widgets` the extension runs. Default: the path of the one that wrote it. |
| `--app DIR` | Where `widgets.json` is, for the widget's name and description. Optional. |

The bar draws the kind's `accessory` tree, or its name when it has none; the popup draws the `root` tree. The extension runs `nsh-widgets show KIND --group NAME --format ir` every two seconds and redraws when something changed; a press runs `nsh-widgets apply`. Both run as a program with arguments, with no shell. Light or dark follows GNOME's Dark Style.

The metadata lists GNOME 45 to 51, and each was run ([status.md](status.md)). GNOME turns an extension off on a version its metadata does not list, so a later GNOME needs a newer `nsh-widgets`. Icons such as the flame are emoji and need an emoji font on the machine.

## Platform notes

- **iOS simulator**: if the widget gallery still shows old kinds, restart the simulator. The simulator sometimes redraws one press late.
- **Android**: force-stopping the app greys out its widgets until the app is opened again.
- **macOS**: needs a real signing identity (a free Personal Team works) and a team-prefixed App Group in `macosAppGroup`. Run the app from a place LaunchServices indexes, such as `~/Applications`, or App Intents fail with "There is no metadata". New kinds need a `build` bump in `nsh.json` (the extension's `CFBundleVersion`).
