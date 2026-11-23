# Status

Verified on 23 and 24 November 2026 with the gallery example ([`examples/gallery`](../examples/gallery)). Screenshots are under [`evidence/`](evidence/).

## Summary

| Platform | State |
|---|---|
| iOS simulator (iOS 27, Xcode 27.0) | Verified: gallery listing, placement, widget presses, app sees presses, gradients, the 21 icons. |
| iOS lock screen | The families build into the extension and register. The simulator's lock-screen editor would not open, so their rendering is unverified. |
| Android emulator (Android 17, API 37) | Verified: picker listing, widget presses with the app idle, app-side redraw, toggles, bitmap gradients on the root and on an inner row, the 21 icons. |
| macOS | Packaged and signed by NSH main (Apple Development identity, team App Group via `macosAppGroup`), installed to `~/Applications`. On the desktop the library's Tasks (checklist rows), Dashboard (gradient banner, progress, bar chart, icons, 9200 steps from the app) and Focus timer (stopwatch, countdown, clock, rotating tip) render with the app's data ([desktop-widgets.png](evidence/macos/desktop-widgets.png); the other NSH widgets in that shot are the older spike app's). Pressing a Tasks row on the desktop toggled it without the app (reported by the user on 23 November 2026; no screenshot). |
| Physical iPhone | Not tested. |
| Physical Android device | Not tested. |
| Linux, Windows | No system widget host. `nsh-widgets show` draws a widget as text or Waybar JSON; checked on macOS against the simulator's store ([show.txt](evidence/tests/show.txt)). The Linux and Windows store paths are covered by code review only. |
| Web | No store. |

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
- The app then showed Count 2: [app-after-widget-presses.png](evidence/android/app-after-widget-presses.png). Its press line reads "1 widget presses: Counter inc:count": the app was running, so `onChange` drained the first press live and that line showed only the latest batch. The gallery now prints a running total.
- "+1 from app" redrew the widget to 3 with "set in the Android app": [home-counter-after-app-press.png](evidence/android/home-counter-after-app-press.png).
- Tasks: toggling t2 in the widget turned its switch on ([home-tasks-toggled.png](evidence/android/home-tasks-toggled.png)); the app showed tasks ☑ ☑ ☐ and "Tasks toggle:t2" ([app-tasks-synced.png](evidence/android/app-tasks-synced.png)).
- The Focus timer's stopwatch moved from 04:03 to 04:07 with the app idle. No screenshot of this is saved.

## 24 November 2026: 0.1.0

The library moved to the repository root and the gallery now imports `nsh-widgets`, `nsh-widgets/base` and `nsh-widgets/kit` by name; `nsh check` type-checks it (a planted type error in `widgets/counter.ts` failed the check).

- iOS: the widget gallery listed six kinds. Steps draws its diagonal gradient, progress and number: [gallery-steps-gradient.png](evidence/ios/gallery-steps-gradient.png). Icons draws all 21 icons on a horizontal gradient: [gallery-icons.png](evidence/ios/gallery-icons.png). The placed Counter and Focus timer kept drawing with the new padding rules.
- Android: Icons placed on the home screen, with the gradient drawn as a bitmap and all 21 vector icons: [home-icons-gradient.png](evidence/android/home-icons-gradient.png). Dashboard placed, its banner row drawing a rounded blue-to-violet gradient: [home-dashboard-gradient.png](evidence/android/home-dashboard-gradient.png).
- `nsh-widgets preview --simulator` drew the six kinds the app had published, light and dark: [gallery-preview.png](evidence/preview/gallery-preview.png).
- `nsh-widgets show` printed each kind and Waybar JSON from the same store: [show.txt](evidence/tests/show.txt).

## Tests

| Command | Result |
|---|---|
| `cargo test --workspace` | 9 store tests ([`tests/store.rs`](../tests/store.rs)) and 13 CLI tests ([`tests.rs`](../crates/nsh-widgets-cli/src/tests.rs)) pass. |
| `nsh test` in `packages/nsh-widgets` | 14 tests pass: 11 in [`base.test.ts`](../packages/nsh-widgets/tests/base.test.ts), 3 in [`kit.test.ts`](../packages/nsh-widgets/tests/kit.test.ts). |
| `scripts/check-aar.sh` | The committed AAR matches its sources; a planted extra drawable made it fail. |

Test output is in `evidence/tests/`. Each suite was seen to fail once against deliberately broken code (the action inbox, the `nsh.json` writer, the action check in `base.ts`, and the toggle in the text renderer). The Swift and Java renderers are covered only by the simulator and emulator runs above. CI is defined in `.github/workflows/ci.yml`.

## Not done

- Live Activities and Control Center controls
- Widgets the user configures (`AppIntentConfiguration`)
- Native Linux widget hosts (KDE Plasma QML, GNOME Shell); `show` covers terminals and Waybar
- Lists that scroll inside a widget
- A crates.io and npm release (NSH installs from the git tags)
- Lock-screen rendering, checked on a device
- Tests on physical devices

CI: run https://github.com/SuperNSH/widgets/actions/runs/45179814280 (commit `ca211e9`) passed all 11 jobs: format; rust on ubuntu, macos and windows; clippy for aarch64-apple-ios, aarch64-apple-ios-sim, aarch64-linux-android and wasm32-unknown-unknown; the Android library rebuild; the gallery's extension build on macOS; and the TypeScript checks.
