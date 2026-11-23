# Getting started

This adds a home-screen widget to an NSH TSX app. The steps match [`examples/gallery`](../examples/gallery), which is the working reference.

Before you start, check [Requirements](../README.md#requirements): the NSH 0.6.0 and 0.6.1 releases cannot package widgets; you need NSH main at or after `5aa56b19`, or the nightly.

## 1. Add the library

In the app's directory:

```sh
nsh add github:SuperNSH/widgets@^0.1
```

`nsh.json` then lists it, and `nsh.lock` pins the version:

```json
"dependencies": {
  "nsh-widgets": { "github": "SuperNSH/widgets", "version": "^0.1" }
}
```

To work on the library itself, point at a checkout instead: `{ "path": "../widgets" }` (the gallery uses `"../.."`).

The repository root is the nshm. It brings the TSX modules `nsh-widgets`, `nsh-widgets/base` and `nsh-widgets/kit` (its `nsh.json` `exports`), the native module they call (`nsh-widgets/native`), the Swift bridge for Apple targets, and the Android AAR (renderer, layouts, icons, provider slots, action receiver).

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

`init` writes a starter `widgets.json` beside `nsh.json` with one kind, `Counter`. The gallery's:

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
| `Widgets.configure(group)` | Uses another App Group on iOS and macOS. On Linux and Windows it names the store `nsh-widgets show --group` reads. Android ignores it. |

Calls throw `WidgetError` with a stable `code`: `invalid_kind`, `invalid_action`, `invalid_json`, `no_app_group`, `no_group_configured`, `io`, `unsupported`.

The Rust API is the same set: `configure`, `set_ui`, `set_data`, `update_data`, `data`, `take_actions`, `reload`, `next_change` (async), and `store` for direct access.

## 6. Build and package

```sh
nsh-widgets build                       # every platform this host can build
nsh-widgets build --only android        # repeatable: --only ios --only macos
nsh-widgets check                       # widgets.json is valid and nsh.json is up to date
nsh package --target ios-simulator      # or ios, android, macos
```

`build` reads `widgets.json` and `nsh.json`, then:

- On macOS, builds `.nsh-widgets/iphonesimulator/NshWidgets.appex`, `.nsh-widgets/iphoneos/NshWidgets.appex` and `.nsh-widgets/macosx/NshWidgets.appex` with `swiftc` (no Xcode project), including App Intents metadata.
- Writes `.nsh-widgets/android/nsh-widgets-app.aar`: one provider slot per kind, its `appwidget-provider` XML and description string.
- Updates `nsh.json`, keeping your own keys:
  - `ios.entitlements` gets the App Group, `ios.infoPlist.NshWidgetsAppGroup` is set, and `ios.extensions` gets the extension (`path`, `simulatorPath`, its entitlements).
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

## Linux and Windows

No system widget host draws these widgets there. The app keeps a store anyway once it names it:

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

With `--format waybar` the bar text comes from the `accessory` tree (the lock-screen tree), and the tooltip holds the full widget.

## Platform notes

- **iOS simulator**: if the widget gallery still shows old kinds, restart the simulator. The simulator sometimes redraws one press late.
- **Android**: force-stopping the app greys out its widgets until the app is opened again.
- **macOS**: needs a real signing identity (a free Personal Team works) and a team-prefixed App Group in `macosAppGroup`. Run the app from a place LaunchServices indexes, such as `~/Applications`, or App Intents fail with "There is no metadata". New kinds need a `build` bump in `nsh.json` (the extension's `CFBundleVersion`).
