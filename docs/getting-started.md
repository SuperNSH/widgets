# Getting started

This adds a home-screen widget to an NSH TSX app. The steps match [`examples/gallery`](../examples/gallery), which is the working reference.

Before you start, check [Requirements](../README.md#requirements): the NSH 0.6.0 release cannot package widgets; you need NSH main at or after `5aa56b19`.

## 1. Add the crate

In the app's `nsh.json`:

```json
"dependencies": {
  "nsh-widgets": { "path": "../widgets/crates/nsh-widgets" }
}
```

The path points at `crates/nsh-widgets` in a checkout of this repo. NSH also reads `"git"` and `"github"` dependencies, but it installs the repository root as the nshm, and this repository's root is a Cargo workspace, not the crate. A git dependency on this repo is untested; use a path until the crate is published.

The crate brings the TSX module `nsh-widgets/native`, the Swift bridge for Apple targets, and the Android AAR (renderer, layouts, icons, provider slots, action receiver) through its `package.metadata.nsh.android.libraries`.

## 2. Install the CLI

```sh
cargo install --git https://github.com/SuperNSH/widgets nsh-widgets-cli
nsh-widgets doctor
```

`doctor` checks Xcode, swiftc, `appintentsmetadataprocessor`, the iOS simulator, iOS and macOS SDKs, and `nsh`. On a host other than macOS it checks only `nsh`; the Apple extensions build only on macOS.

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
| `widgets[].families` | no | `small`, `medium`, `large`, `extraLarge` (iPad and Mac). Default `["small", "medium"]`. |
| `widgets[].android.cells` | no | `[columns, rows]`, 1 to 5 each. Default from the first family: small 2x2, medium 4x2, large 4x4. |

`widgets.json` sits beside `nsh.json` instead of inside it because NSH warns about keys it does not know.

## 4. Write the widget

A widget is a function that returns a tree. The gallery's counter, [`widgets/counter.ts`](../examples/gallery/widgets/counter.ts):

```ts
// The counter, written with kit components. No platform code here.
import { setTheme } from "../../../packages/nsh-widgets/src/base";
import { ActionButton, ButtonRow, Card, Gap, Header, LinkButton, Stat, kitTheme } from "../../../packages/nsh-widgets/src/kit";

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

The TS package is not published yet, so the gallery imports it by relative path from `packages/nsh-widgets/src`. Do the same from your app, or copy the three files in.

`{count}` and `{label}` are filled from the kind's data when the widget draws. `inc:count` runs in the widget process, so the button works with the app closed. [ir.md](ir.md) lists every primitive, class and action; [capabilities.md](capabilities.md) says how each one looks on each platform.

## 5. Publish the tree and feed data

From the gallery's [`App.tsx`](../examples/gallery/App.tsx):

```ts
import { Widgets } from "../../packages/nsh-widgets/src";

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
| `Widgets.setUi(kind, root, { url? })` | Writes the kind's snapshot and redraws. `url` opens when the widget itself is tapped. |
| `Widgets.setData(kind, data)` | Replaces the data (a JSON object) and redraws. |
| `Widgets.update(kind, patch)` | Merges top-level keys into the data, keeps keys the widget changed that the patch does not name, redraws, returns the data. |
| `Widgets.data(kind)` | The data as the widget last left it, `{}` before anything wrote it. |
| `Widgets.takeActions()` | Button presses since the last call, oldest first, each returned once: `{ id, kind, action, at }`. |
| `Widgets.reload(kind)` | Redraws placed widgets of the kind. |
| `Widgets.onChange(kind, listener)` | Calls `listener` with the data each time a widget button changes it. Returns a stop function. Stops by itself where there is no widget host. |
| `Widgets.configure(group)` | Uses another App Group on iOS and macOS. Android ignores it. |

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

## Platform notes

- **iOS simulator**: if the widget gallery still shows old kinds, restart the simulator. The simulator sometimes redraws one press late.
- **Android**: force-stopping the app greys out its widgets until the app is opened again.
- **macOS**: needs a real signing identity (a free Personal Team works) and a team-prefixed App Group in `macosAppGroup`. Run the app from a place LaunchServices indexes, such as `~/Applications`, or App Intents fail with "There is no metadata". New kinds need a `build` bump in `nsh.json` (the extension's `CFBundleVersion`). The macOS path is unverified with this library build; see [status.md](status.md).
