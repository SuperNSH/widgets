# nsh-widgets

Home-screen widgets for Native Shell (NSH) apps.

The app describes each widget once, as a tree of primitives (Widget Tree IR v1), and feeds it data. You write the tree in TSX with the `nsh-widgets` TypeScript package, or as JSON from Rust. A generic renderer draws it natively:

- iOS and macOS: SwiftUI in a WidgetKit extension. The renderer is one Swift file, [`crates/nsh-widgets-cli/renderer/NshWidgets.swift`](crates/nsh-widgets-cli/renderer/NshWidgets.swift), compiled into the app's extension by `nsh-widgets build`.
- Android: RemoteViews, from Java in the crate's AAR ([`crates/nsh-widgets/android/`](crates/nsh-widgets/android/)).

Widget buttons change the data without the app. On Apple platforms a button runs an App Intent in the extension; on Android it sends a broadcast to a non-exported receiver in the app's process. The app reads the presses later, or hears them live.

Part of the SuperNSH set of optional NSH libraries. Repository: https://github.com/SuperNSH/widgets

## Status

Version 0.1.0, unreleased. Verified on 23 November 2026; details and screenshots in [docs/status.md](docs/status.md).

- **iOS 27 simulator**: the gallery example, packaged by NSH main with the extension, shows its kinds in the widget gallery. Two `+1` presses in a placed Counter widget wrote `count: 2` and two events, and the app showed both.
- **Android 17 (API 37) emulator**: the launcher lists all four kinds. Widget presses changed the counter with the app idle, the app saw them, and an app-side change redrew the widget. A Tasks switch toggled in the widget showed up in the app.
- **macOS**: the extension builds (swiftc and App Intents metadata). The library build has not been placed on a Mac yet. Unverified.
- **Physical iPhone and Android devices**: not tested.
- **Linux, Windows, web**: no widget host. The store works there only through the `NSH_WIDGETS_DIR` environment variable, and nothing redraws.

Tests: `cargo test --workspace` runs 8 store tests and 7 CLI tests; `nsh test` in `packages/nsh-widgets` runs 12 tests of the TS layers. Output is saved in `docs/evidence/tests/`. Each suite was seen to fail once against deliberately broken code. CI (`.github/workflows/ci.yml`) runs format, clippy and tests on Linux, macOS and Windows, cross-checks iOS, Android and web, rebuilds the Android AAR, builds the gallery's widget extension on macOS, and runs the TS checks.

Not done yet:

- Linux widget hosts (KDE Plasma QML, GNOME and others)
- Live Activities
- Control Center controls
- Lock-screen accessory families
- Configurable widgets (`AppIntentConfiguration`)
- A widget preview command
- Interactive data lists
- Device signing and provisioning help in `nsh-widgets doctor`
- Publishing the TS package and the crates

## Requirements

- NSH main at or after commit `5aa56b19` (2026-11-23). It needs `ios.extensions` and `macos.extensions` in `nsh.json` (longbridge/nsh#2879), `android.libraries`, and the AAR manifest merge. The NSH 0.6.0 release has none of these.
- For Apple targets: macOS with Xcode 26 or later, including `appintentsmetadataprocessor`. Tested with Xcode 27.0 (27A266a). `nsh-widgets doctor` checks the tools.
- Android API 26 or later. Switches, light and dark colours, corner radius and icon sizing need API 31.

## Layout

- `crates/nsh-widgets/`: the nshm Rust crate. The shared store, the Rust API, the TSX native module `nsh-widgets/native`, the Swift bridge (`swift/Bridge.swift`, compiled by `build.rs`), JNI for Android, and the Android AAR.
- `crates/nsh-widgets-cli/`: the `nsh-widgets` command (`init`, `build`, `check`, `doctor`) and the Swift renderer it compiles into each app's extension.
- `packages/nsh-widgets/`: the TypeScript package. `base` (primitives and the class resolver), `kit` (styled components built from base), and the `Widgets` API.
- `examples/gallery/`: an NSH TSX app with four kinds: Counter, Dashboard, Live ("Focus timer") and Tasks.
- `scripts/build-aar.sh`: rebuilds `nsh-widgets.aar` from the Java sources and resources.
- `docs/`: [getting-started.md](docs/getting-started.md), [ir.md](docs/ir.md), [capabilities.md](docs/capabilities.md), [decisions.md](docs/decisions.md), [status.md](docs/status.md), and `evidence/`.
- `site/`: the static project page.

## Quick look

A widget in TSX, from [`examples/gallery/widgets/counter.ts`](examples/gallery/widgets/counter.ts):

```ts
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

[docs/getting-started.md](docs/getting-started.md) walks through adding widgets to an app.

## Licence

MIT. See [LICENSE](LICENSE).

CI: run https://github.com/SuperNSH/widgets/actions/runs/45179814280 (commit `ca211e9`) passed all 11 jobs: format; rust on ubuntu, macos and windows; clippy for aarch64-apple-ios, aarch64-apple-ios-sim, aarch64-linux-android and wasm32-unknown-unknown; the Android library rebuild; the gallery's extension build on macOS; and the TypeScript checks.
