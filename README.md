# nsh-widgets

Home-screen widgets for Native Shell (NSH) apps.

The app describes each widget once, as a tree of primitives (Widget Tree IR v1), and feeds it data. You write the tree in TSX with `nsh-widgets/base` and `nsh-widgets/kit`, or as JSON from Rust. A generic renderer draws it natively:

- iOS and macOS: SwiftUI in a WidgetKit extension. The renderer is one Swift file, [`crates/nsh-widgets-cli/renderer/NshWidgets.swift`](crates/nsh-widgets-cli/renderer/NshWidgets.swift), compiled into the app's extension by `nsh-widgets build`. iOS lock-screen widgets (circular, rectangular, inline) take a smaller tree of their own.
- Android: RemoteViews, from Java in the crate's AAR ([`android/`](android/)).
- Terminals and Linux status bars: `nsh-widgets show` prints a widget as text or Waybar JSON.

Widget buttons change the data without the app. On Apple platforms a button runs an App Intent in the extension; on Android it sends a broadcast to a non-exported receiver in the app's process. The app reads the presses later, or hears them live.

Part of the SuperNSH set of optional NSH libraries. Repository: https://github.com/SuperNSH/widgets

## Install

In the app's directory:

```sh
nsh add github:SuperNSH/widgets@^0.1
```

That lists it in `nsh.json` and pins it in `nsh.lock`:

```json
"dependencies": {
  "nsh-widgets": { "github": "SuperNSH/widgets", "version": "^0.1" }
}
```

The repository root is the nshm: a Rust crate with a `#[native_module]` and an `nsh.json` whose `exports` give TSX the `nsh-widgets`, `nsh-widgets/base` and `nsh-widgets/kit` modules.

The `nsh-widgets` command builds the extensions. Download it from the [releases](https://github.com/SuperNSH/widgets/releases), or:

```sh
cargo install --git https://github.com/SuperNSH/widgets nsh-widgets-cli
```

[docs/getting-started.md](docs/getting-started.md) walks through the rest.

## Status

Version 0.1.0 ([release](https://github.com/SuperNSH/widgets/releases/tag/v0.1.0)). Verified on 23 and 24 November 2026, including a fresh app installing it with `nsh add github:SuperNSH/widgets@^0.1`; details and screenshots in [docs/status.md](docs/status.md).

- **iOS 27 simulator**: the gallery example shows all six kinds in the widget gallery, including real gradients and the 21 shared icons. Two `+1` presses in a placed Counter wrote `count: 2` and two events, and the app showed both.
- **Android 17 (API 37) emulator**: the launcher lists every kind. Widget presses changed the counter with the app idle, the app saw them, and an app-side change redrew the widget. A Tasks switch toggled in the widget showed up in the app. Gradients draw as bitmaps behind the content, including on an inner rounded row.
- **macOS**: packaged, signed and placed on the desktop; Tasks, Dashboard and Focus timer render with the app's data ([screenshot](docs/evidence/macos/desktop-widgets.png)), and a Tasks row toggled without the app (reported by the user).
- **iOS lock screen**: the families build and register; the simulator's lock-screen editor would not open, so their rendering is unverified.
- **Linux, Windows**: no system widget host. The app's store lives under the user's data directory; `nsh-widgets show --group NAME --follow` prints a widget for a terminal or, with `--format waybar`, for Waybar.
- **Physical iPhone and Android devices**: not tested.

Tests: `cargo test --workspace` runs 9 store tests and 13 CLI tests; `nsh test` in `packages/nsh-widgets` runs 14 TS tests. Output is in `docs/evidence/tests/`. Each suite was seen to fail once against deliberately broken code. CI runs format, clippy and tests on Linux, macOS and Windows, cross-checks iOS, Android and web, checks the committed Android AAR against its sources, builds the gallery's widget extensions on macOS, type-checks the gallery with NSH nightly, and runs the TS checks.

Not in this release:

- Live Activities and Control Center controls
- Widgets the user configures (`AppIntentConfiguration`)
- Native Linux hosts (KDE Plasma QML, GNOME Shell); `show` covers terminals and Waybar
- Lists that scroll inside a widget

## Requirements

- NSH main at or after commit `5aa56b19` (23 November 2026), or the nightly. It needs `ios.extensions` and `macos.extensions` in `nsh.json` (longbridge/nsh#2879), `android.libraries`, and the AAR manifest merge. The 0.6.0 and 0.6.1 releases have none of these.
- For Apple targets: macOS with Xcode 26 or later, including `appintentsmetadataprocessor`. Tested with Xcode 27.0 (27A266a). `nsh-widgets doctor` checks the tools.
- Android API 26 or later. Switches, light and dark colours, corner radius and icon sizing need API 31.

## Layout

- `src/`, `swift/`, `android/`, `build.rs`, `Cargo.toml`, `nsh.json` (repo root): the nshm. The shared store, the Rust API, the TSX native module `nsh-widgets/native`, the Swift bridge, JNI for Android, and the Android AAR with its sources.
- `packages/nsh-widgets/`: the TypeScript the nshm exports. `base` (primitives and the class resolver), `kit` (styled components built from base), and the `Widgets` API, with its tests.
- `crates/nsh-widgets-cli/`: the `nsh-widgets` command (`init`, `build`, `check`, `preview`, `show`, `doctor`) and the Swift renderer it compiles into each app's extension.
- `examples/gallery/`: an NSH TSX app with six kinds: Counter, Dashboard, Focus timer, Tasks, Steps and Icons.
- `scripts/`: `build-aar.sh` rebuilds the AAR; `check-aar.sh` checks the committed one matches its sources.
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
