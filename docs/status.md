# Status

Verified on 23 November 2026 with the gallery example ([`examples/gallery`](../examples/gallery)). Screenshots are under [`evidence/`](evidence/).

## Summary

| Platform | State |
|---|---|
| iOS simulator (iOS 27, Xcode 27.0) | Verified: gallery listing, placement, widget presses, app sees presses. |
| Android emulator (Android 17, API 37) | Verified: picker listing, widget presses with the app idle, app-side redraw, toggles. |
| macOS | Packaged and signed by NSH main (Apple Development identity, team App Group via `macosAppGroup`), installed to `~/Applications`. On the desktop the library's Tasks (checklist rows), Dashboard (gradient banner, progress, bar chart, icons, 9200 steps from the app) and Focus timer (stopwatch, countdown, clock, rotating tip) render with the app's data ([desktop-widgets.png](evidence/macos/desktop-widgets.png); the other NSH widgets in that shot are the older spike app's). Pressing a Tasks row on the desktop toggled it without the app (reported by the user on 23 November 2026; no screenshot). |
| Physical iPhone | Not tested. |
| Physical Android device | Not tested. |
| Linux, Windows, web | No widget host. The store works through `NSH_WIDGETS_DIR` only. |

## iOS 27 simulator

The gallery was packaged by NSH main with the extension and installed on an iPhone simulator.

- The app published all four kinds: [app-published.png](evidence/ios/app-published.png) shows "widgets published" and Count 0.
- The widget gallery listed the kinds. Counter: [gallery-counter.png](evidence/ios/gallery-counter.png). Dashboard, with the gradient banner, progress bar, bar chart and icons: [gallery-dashboard.png](evidence/ios/gallery-dashboard.png).
- Focus timer (medium) and Counter (small) placed on the home screen, with the stopwatch at 1:17, the countdown at 23:43, the clock and a rotating tip: [home-live-counter.png](evidence/ios/home-live-counter.png).
- Two `+1` presses in the Counter widget wrote `count: 2` to `data.json` and two events with UUID ids to `actions.jsonl`. The app showed Count 2 and "2 widget presses: Counter inc:count, Counter inc:count": [app-after-widget-presses.png](evidence/ios/app-after-widget-presses.png). The screenshot predates a wording change in `App.tsx`, which now prints "2 widget presses, last: ...".

The `data.json` and `actions.jsonl` contents were read from the App Group container at the time; the files are not saved under `evidence/`.

## Android 17 (API 37) emulator

- The app published the kinds: [app-published.png](evidence/android/app-published.png).
- The launcher's widget picker listed "Counter, Dashboard, Tasks, Focus timer" under Widget Gallery: [picker-kinds.png](evidence/android/picker-kinds.png).
- Counter placed at 0: [home-counter-0.png](evidence/android/home-counter-0.png). Two `+1` presses with the app idle: [home-counter-after-widget-presses.png](evidence/android/home-counter-after-widget-presses.png) shows 2.
- The app then showed Count 2: [app-after-widget-presses.png](evidence/android/app-after-widget-presses.png). Its press line reads "1 widget presses: Counter inc:count", one event rather than two; the count is right, and the cause of the missing second event in that line was not traced. Unverified.
- "+1 from app" redrew the widget to 3 with "set in the Android app": [home-counter-after-app-press.png](evidence/android/home-counter-after-app-press.png).
- Tasks: toggling t2 in the widget turned its switch on ([home-tasks-toggled.png](evidence/android/home-tasks-toggled.png)); the app showed tasks ☑ ☑ ☐ and "Tasks toggle:t2" ([app-tasks-synced.png](evidence/android/app-tasks-synced.png)).
- The Focus timer's stopwatch moved from 04:03 to 04:07 with the app idle. No screenshot of this is saved.

## Tests

| Command | Result |
|---|---|
| `cargo test -p nsh-widgets` | 8 store tests pass ([`tests/store.rs`](../crates/nsh-widgets/tests/store.rs)). |
| `nsh test` in `packages/nsh-widgets` | 12 tests pass: 9 in [`base.test.ts`](../packages/nsh-widgets/tests/base.test.ts), 3 in [`kit.test.ts`](../packages/nsh-widgets/tests/kit.test.ts). |

Test output is in `evidence/tests/` (8 store tests, 7 CLI tests, 12 TS tests); each suite was seen to fail once against deliberately broken code. The Swift and Java renderers are covered only by the simulator and emulator runs above. CI is defined in `.github/workflows/ci.yml`.

## Not done

- Linux widget hosts (KDE Plasma QML, GNOME and others)
- Live Activities
- Control Center controls
- Lock-screen accessory families
- Configurable widgets (`AppIntentConfiguration`)
- A widget preview command
- Interactive data lists
- Device signing and provisioning help in `nsh-widgets doctor`
- Publishing the TS package and the crates
- macOS verification with this library build
- Tests on physical devices

CI: run https://github.com/SuperNSH/widgets/actions/runs/45179814280 (commit `ca211e9`) passed all 11 jobs: format; rust on ubuntu, macos and windows; clippy for aarch64-apple-ios, aarch64-apple-ios-sim, aarch64-linux-android and wasm32-unknown-unknown; the Android library rebuild; the gallery's extension build on macOS; and the TypeScript checks.
