# nsh-widgets

Home-screen widgets for Native Shell (NSH) apps.

The app describes each widget once, as a tree of primitives (Widget Tree IR v1), and feeds it data. You write the tree in TSX with `nsh-widgets/base` and `nsh-widgets/kit`, or as JSON from Rust. A generic renderer draws it natively:

- iOS and macOS: SwiftUI in a WidgetKit extension. The renderer is one Swift file, [`crates/nsh-widgets-cli/renderer/NshWidgets.swift`](crates/nsh-widgets-cli/renderer/NshWidgets.swift), compiled into the app's extension by `nsh-widgets build`. iOS lock-screen widgets (circular, rectangular, inline) take a smaller tree of their own.
- Android: RemoteViews, from Java in the crate's AAR ([`android/`](android/)).
- KDE Plasma: a generated plasmoid whose QML renderer ([`crates/nsh-widgets-cli/plasma/NshNode.qml`](crates/nsh-widgets-cli/plasma/NshNode.qml)) draws the same tree.
- GNOME Shell: a generated extension ([`crates/nsh-widgets-cli/js/st.js`](crates/nsh-widgets-cli/js/st.js)) draws the small tree in the top bar and the full tree in its popup.
- Cinnamon: a generated desklet, on the GNOME renderer.
- Other X11 desktops, and Wayland compositors with layer-shell (sway, Hyprland and others): a generated GTK 3 window under the other windows.
- Terminals and Linux status bars: `nsh-widgets show` prints a widget as text or Waybar JSON.

The same trees drive iOS Live Activities (lock screen and Dynamic Island). Widgets can offer the user a choice of values (an edit sheet on iOS and macOS, a configure screen on Android), draw lists (scrolling on Android 12+ and KDE), and an app can add iOS 18 Control Center controls and Android Quick Settings tiles bound to a widget's data.

Widget buttons change the data without the app. On Apple platforms a button runs an App Intent in the extension; on Android it sends a broadcast to a non-exported receiver in the app's process. The app reads the presses later, or hears them live.

Part of the SuperNSH set of optional NSH libraries. Repository: https://github.com/SuperNSH/widgets

## Install

In the app's directory:

```sh
nsh add github:SuperNSH/widgets@^0.4
```

That lists it in `nsh.json` and pins it in `nsh.lock`:

```json
"dependencies": {
  "nsh-widgets": { "github": "SuperNSH/widgets", "version": "^0.2" }
}
```

The repository root is the nshm: a Rust crate with a `#[native_module]` and an `nsh.json` whose `exports` give TSX the `nsh-widgets`, `nsh-widgets/base` and `nsh-widgets/kit` modules.

The `nsh-widgets` command builds the extensions. Download it from the [releases](https://github.com/SuperNSH/widgets/releases), or:

```sh
cargo install --git https://github.com/SuperNSH/widgets nsh-widgets-cli
```

[docs/getting-started.md](docs/getting-started.md) walks through the rest.

## Status

Version 0.4.0. Verified on 23 and 24 November 2026; details and screenshots in [docs/status.md](docs/status.md).

- **iOS 27 simulator**: the gallery's kinds list in the widget gallery and draw on the home screen; widget presses reach the app. Lock-screen widgets were placed and drawn ([screenshot](docs/evidence/ios/lock-screen-placed.png)). The Order Live Activity started from the app and showed in the compact and expanded Dynamic Island and as a lock-screen banner, and an update moved it to "On the way". The Weather edit sheet offered Karachi, London and Tokyo, and London drew London's data. The Agenda list drew 12 items. Both controls listed in the Control Center gallery; Count +1 placed there moved the Counter widget to 5.
- **Android 17 (API 37) emulator**: the Agenda list drew all 12 items and scrolled when the widget was made smaller. Weather's configure screen switched it to Tokyo. The Quick Settings tiles changed Counter and toggled a task, and the app received the presses.
- **KDE Plasma 5.27 and 6.3** (5.27 in a full session, both in `plasmoidviewer`; Ubuntu 24.04 and Debian 13): Dashboard, Tasks and Agenda drew from the app's store; a button raised the streak and queued an event, a task toggled, the list scrolled and kept its place. In a panel it draws the small tree (fixed in 0.2.1).
- **macOS**: 0.1.0's Tasks, Dashboard and Focus timer render on the desktop ([screenshot](docs/evidence/macos/tasks-dashboard.png)). The 0.2.0 extension builds with options; their rendering on a Mac is unverified.
- **GNOME Shell 45 to 51** (headless, driven by a probe; 48.7 also nested, with a pointer): Dashboard, Tasks and Agenda drew from the store; a button raised the streak, a task toggled, the list scrolled and kept its place, and the popup followed the light or dark setting. A full GNOME session is unverified.
- **Cinnamon 6.0, 6.4 and 6.6**: drawing, a button, a toggle, the list scrolled, dark while running, removed and added back.
- **X11 (Xfce) and layer-shell (sway)**: the same, with the window staying under others. Hyprland, COSMIC and KWin have layer-shell but were not run.
- **Windows, and Wayland compositors without layer-shell**: no widget host. `nsh-widgets show --group NAME --follow` prints a widget for a terminal or, with `--format waybar`, for Waybar.
- **Physical iPhone and Android devices**: not tested.

Tests: `cargo test --workspace` runs 10 store tests and 21 CLI tests; `nsh test` in `packages/nsh-widgets` runs 15 TS tests. Output is in `docs/evidence/tests/`. Each suite was seen to fail once against deliberately broken code. CI runs format, clippy and tests on Linux, macOS and Windows, cross-checks iOS, Android and web, checks the committed Android AAR against its sources, builds the gallery's widget extensions on macOS, type-checks the gallery with NSH nightly, generates a Plasma 5 and a Plasma 6 plasmoid, a GNOME extension, a Cinnamon desklet and a desktop window (without loading them), and runs the TS checks.

Not in this release:

- A Windows widget host and a COSMIC applet; `show` covers terminals and Waybar
- Runs on physical devices
- A crates.io and npm release (NSH installs from the git tags)

## Requirements

- NSH main at or after commit `5aa56b19` (23 November 2026), or the nightly. It needs `ios.extensions` and `macos.extensions` in `nsh.json` (longbridge/nsh#2879), `android.libraries`, and the AAR manifest merge. The 0.6.0 and 0.6.1 releases have none of these.
- For Apple targets: macOS with Xcode 26 or later, including `appintentsmetadataprocessor`. Tested with Xcode 27.0 (27A266a). `nsh-widgets doctor` checks the tools.
- Android API 26 or later. Switches, light and dark colours, corner radius, icon sizing and scrolling lists need API 31.
- Live Activities need iOS 16.2 or later; controls need iOS 18.
- KDE Plasma 5 or 6 with `kpackagetool5` or `kpackagetool6`, and the `nsh-widgets` binary on the machine the plasmoid runs on.
- GNOME Shell 45 to 51, Cinnamon 6, or GJS with GTK 3 (and gtk-layer-shell on Wayland) for the window; each needs the `nsh-widgets` binary on the machine it runs on.

## Layout

- `src/`, `swift/`, `android/`, `build.rs`, `Cargo.toml`, `nsh.json` (repo root): the nshm. The shared store, the Rust API, the TSX native module `nsh-widgets/native`, the Swift bridge, JNI for Android, and the Android AAR with its sources.
- `packages/nsh-widgets/`: the TypeScript the nshm exports. `base` (primitives and the class resolver), `kit` (styled components built from base), and the `Widgets` API, with its tests.
- `crates/nsh-widgets-cli/`: the `nsh-widgets` command (`init`, `build`, `check`, `preview`, `show`, `apply`, `plasmoid`, `gnome`, `cinnamon`, `window`, `doctor`), the Swift renderer and Live Activity it compiles into each app's extension (`renderer/`), the KDE Plasma QML (`plasma/`) the GNOME Shell extension (`gnome/`), the Cinnamon desklet (`cinnamon/`), the desktop window (`window/`), and the JavaScript renderers those three share (`js/`).
- `examples/gallery/`: an NSH TSX app with eight kinds (Counter, Dashboard, Focus timer, Tasks, Steps, Icons, Agenda, Weather), the Order Live Activity and two controls.
- `scripts/`: `build-aar.sh` rebuilds the AAR; `check-aar.sh` checks the committed one matches its sources; `gnome-probe/` drives the GNOME extensions inside a headless GNOME Shell.
- `docs/`: [getting-started.md](docs/getting-started.md), [ir.md](docs/ir.md), [capabilities.md](docs/capabilities.md), [decisions.md](docs/decisions.md), [status.md](docs/status.md), and `evidence/`.
- `site/`: the project page, https://supernsh.github.io/widgets/

## Quick look

A widget in TSX, from [`examples/gallery/widgets/counter.ts`](examples/gallery/widgets/counter.ts):

```ts
import { ActionButton, ButtonRow, Card, Gap, Header, LinkButton, Stat } from "nsh-widgets/kit";

Card(
  Header("NSH WIDGETS", platform),
  Gap(),
  Stat("{count}", "{label}"),
  Gap(),
  ButtonRow(
    [ActionButton("+1", "inc:count"), ActionButton("−1", "dec:count", { variant: "secondary" })],
    LinkButton("Open", "nshwidgets://widget/Counter", { hide: "small:hidden" }),
  ),
);
```

Published and fed from the app ([`examples/gallery/App.tsx`](examples/gallery/App.tsx)):

```ts
import { Widgets } from "nsh-widgets";

Widgets.setUi("Counter", counterWidget(here), { url: "nshwidgets://widget/Counter" });
Widgets.update("Counter", { count: 0, label: "tap +1 in the widget" });
Widgets.onChange("Counter", (d) => {
  if (typeof d.count === "number") count.set(d.count);
});
```

A Live Activity uses the same trees (the gallery's Order, in [`widgets/demos.ts`](examples/gallery/widgets/demos.ts)):

```ts
Widgets.setUi("Order", orderActivity(), { accessory: orderAccessory() });
Widgets.startActivity("Order", { order: "#1042", status: "Confirmed", progress: 0.25, eta: Date.now() + 20 * 60 * 1000 });
```

From Rust:

```rust
nsh_widgets::update_data("Counter", &serde_json::json!({ "count": 3 }))?;
```

Checking a layout without a device:

```sh
nsh-widgets preview --simulator          # every kind, every size, light and dark, as HTML
nsh-widgets show Steps --simulator       # as text
```

## Licence

MIT. See [LICENSE](LICENSE).
