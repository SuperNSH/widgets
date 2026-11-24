# Status

Verified on 23 and 24 November 2026 with the gallery example ([`examples/gallery`](../examples/gallery)). Screenshots are under [`evidence/`](evidence/). The current version is 0.4.0; its own section is [below](#24-november-2026-040).

## Summary

| Platform | State |
|---|---|
| iOS simulator (iOS 27, Xcode 27.0) | Verified: gallery listing, placement, widget presses, app sees presses, gradients, the 21 icons. |
| iOS lock screen | Verified in 0.2.0: Steps placed as a rectangular and a circular widget and drawn. |
| iOS Live Activities | Verified in 0.2.0: compact and expanded Dynamic Island, lock-screen banner, an update. |
| iOS options, lists, controls | Verified in 0.2.0: the Weather edit sheet, the Agenda list, both controls in the Control Center gallery, Count +1 in Control Center. |
| Android emulator (Android 17, API 37) | Verified: picker listing, widget presses with the app idle, app-side redraw, toggles, bitmap gradients on the root and on an inner row, the 21 icons. In 0.2.0: the scrolling list, the configure screen, Quick Settings tiles. |
| macOS | Packaged and signed by NSH main (Apple Development identity, team App Group via `macosAppGroup`), installed to `~/Applications`. On the desktop the library's Tasks (checklist rows), Dashboard (gradient banner, progress, bar chart, icons, 9200 steps from the app) and Focus timer (stopwatch, countdown, clock, rotating tip) render with the app's data ([tasks-dashboard.png](evidence/macos/tasks-dashboard.png), [focus-timer.png](evidence/macos/focus-timer.png)). Pressing a Tasks row on the desktop toggled it without the app (reported by the user on 23 November 2026; no screenshot). |
| macOS, 0.2.0 features | The extension builds with options. Options and lists have not been seen on a Mac: unverified. |
| KDE Plasma 5.27 | Verified in 0.4.0 in a full session, desktop and panel forms; in 0.2.0 in `plasmoidviewer` on Ubuntu 24.04: Dashboard, a button press, Tasks toggled, the Agenda list scrolled. |
| KDE Plasma 6.3 | Verified in 0.2.0 in `plasmoidviewer` on Debian 13: Dashboard and a button press (streak 13 to 14, [plasma6-dashboard-after-button.jpg](evidence/kde/plasma6-dashboard-after-button.jpg)), a task toggled ([plasma6-tasks-toggled.jpg](evidence/kde/plasma6-tasks-toggled.jpg)), and the Agenda list scrolled and still in place seven seconds later ([plasma6-agenda-scrolled.jpg](evidence/kde/plasma6-agenda-scrolled.jpg)). This run found two bugs, fixed before release and rechecked on Plasma 5.27: a list grew the widget instead of scrolling, and each two-second read reset its scroll. |
| Physical iPhone | Not tested. |
| Physical Android device | Not tested. |
| GNOME Shell 45 to 51 | Verified in 0.3.0: 48.7 nested with a pointer, all seven headless through a probe: drawing, a button press, a toggle, the Agenda list scrolled, light and dark, timers, disable and enable. A full session is unverified. |
| Cinnamon 6.0, 6.4, 6.6 | Verified in 0.4.0: drawing, a press, a toggle, a scrolled list, dark, removed and added back. |
| X11 (Xfce 4.18), layer-shell (sway 1.9) | Verified in 0.4.0: the same; the window stays under others. Other compositors are unverified. |
| Windows, other Wayland compositors | No widget host. `nsh-widgets show` draws a widget as text or Waybar JSON; checked on macOS against the simulator's store ([show.txt](evidence/tests/show.txt)). The Linux and Windows store paths are covered by code review only. |
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

## 24 November 2026: 0.2.0

The gallery gained Agenda (a list of 12 events), Weather (options Karachi, London, Tokyo), the Order Live Activity, and two controls: "Ship it" (a toggle on Tasks' `t1`) and "Count +1" (`inc:count` on Counter).

### iOS 27 simulator

- Lock screen: the Steps widget gallery showed the circular ("⚡ 9039") and rectangular ("⚡ 9039 steps") widgets, both placed ([lock-screen-steps.png](evidence/ios/lock-screen-steps.png)), and they drew on the lock screen ([lock-screen-placed.png](evidence/ios/lock-screen-placed.png)). This closes 0.1.0's unverified lock-screen rendering.
- Live Activity: "Start order" in the app started Order. The compact Dynamic Island showed the home icon and the countdown ([activity-island-compact.png](evidence/ios/activity-island-compact.png)). The expanded island showed "Order #1042", "Confirmed", the progress bar and "Arrives in 19:50" ([activity-island-expanded.png](evidence/ios/activity-island-expanded.png)); the island's curved edge clips the "A" of "Arrives". The lock-screen banner drew the green gradient, with iOS's "Allow Live Activities from Widget Gallery?" prompt below it ([activity-lock-screen.png](evidence/ios/activity-lock-screen.png)). "Advance" updated it to "On the way", about 75% and "Arrives in 9:59" ([activity-updated.png](evidence/ios/activity-updated.png)).
- Options: Weather's edit sheet showed "City" with Karachi, London and Tokyo ([weather-options.png](evidence/ios/weather-options.png)). With London chosen the widget drew "London", "9°", "Light rain" ([weather-london.png](evidence/ios/weather-london.png)).
- List: Agenda in the widget gallery drew all 12 events, 08:00 Standup to 19:00 Sleep, with "12 events" in the header ([gallery-agenda-list.png](evidence/ios/gallery-agenda-list.png)). Its `max` is 12.
- Controls: the Control Center gallery listed "Ship it" and "Count +1" under the app ([controls-gallery.png](evidence/ios/controls-gallery.png)). Count +1 was placed in Control Center ([controls-control-center.png](evidence/ios/controls-control-center.png)). Presses moved Counter's `count` from 2 to 3 to 4, read from `data.json` in the App Group container (not saved under `evidence/`). The Counter widget did not redraw until the action intent was changed to reload the kind's widgets; after that fix a press drew 5 on the home screen ([controls-counter-updated.png](evidence/ios/controls-counter-updated.png)).

### Android 17 (API 37) emulator

- List: Agenda drew all 12 events ([agenda-list.png](evidence/android/agenda-list.png)). Resized smaller, it showed 08:00 to 13:00 ([agenda-list-top.png](evidence/android/agenda-list-top.png)) and scrolled to 14:00 to 19:00 ([agenda-list-scrolled.png](evidence/android/agenda-list-scrolled.png)).
- Options: Weather was placed with Karachi, the first value. The launcher's "Settings" on the widget opened the "City" list ([weather-configure.png](evidence/android/weather-configure.png)); choosing Tokyo drew "Tokyo", "16°", "Clear" ([weather-tokyo.png](evidence/android/weather-tokyo.png)).
- Tiles: both tiles in Quick Settings, Ship it active ([tiles.png](evidence/android/tiles.png)). Two Count +1 clicks moved Count from 5 to 7, and the app received both presses. A Ship it click turned `t1` off and the tile inactive ([tiles-after-toggle.png](evidence/android/tiles-after-toggle.png)); the app showed "Count: 7", tasks ☐ ☑ ☑ and "3 widget presses, last: Tasks toggle:t1" ([tiles-app-synced.png](evidence/android/tiles-app-synced.png)). Taps sent with `adb shell input tap` on the tiles did not register on this emulator; the clicks went through `adb shell cmd statusbar click-tile`, which runs the same `TileService.onClick`.

### KDE Plasma 5.27 (Ubuntu 24.04, `plasmoidviewer`)

Plasma 5 plasmoids made with `nsh-widgets plasmoid KIND --group NAME --plasma 5`, reading a store with the gallery's data.

- Dashboard drew the gradient banner ("Today", "9039 steps"), "Goal 10000", 90%, the progress bar, the bar chart, the star and flame glyphs, "12 day streak", "407 kcal" and the two buttons ([dashboard.jpg](evidence/kde/dashboard.jpg)).
- "+1 day" raised the streak to 13 in the store and queued an event in `actions.jsonl` ([dashboard-after-button.jpg](evidence/kde/dashboard-after-button.jpg)).
- Tasks after toggling from the plasmoid: two rows on (☑, struck through), one off ([tasks-toggled.jpg](evidence/kde/tasks-toggled.jpg)).
- Agenda: the list at 08:00 ([agenda-top.webp](evidence/kde/agenda-top.webp)) and scrolled to 14:00 to 19:00 ([agenda-scrolled.jpg](evidence/kde/agenda-scrolled.jpg)).

The Ubuntu machine's clock in these screenshots reads 23 November, 21:22 to 21:23.

### Not verified in 0.2.0

- macOS rendering of options and lists. The Mac extension builds with the generated option intents; nothing was placed on a Mac.
- A full Plasma session: both KDE forms were seen only in `plasmoidviewer`. The panel form was broken in 0.2.0; in 0.2.1, with a hand-written Steps tree, it drew the `accessory` tree and followed the data on Plasma 5.27 and 6.3, and on 5.27 a click opened the full widget ([panel-opened.png](evidence/kde/panel-opened.png); the viewer paints no panel behind it, so the text is dark on black).
- GNOME: no host in 0.2; 0.3.0 adds one.
- Physical iPhone and Android devices: none was connected, and `nsh-widgets doctor --app` on the gallery reported no provisioning profiles for the app or its extension.

## 24 November 2026: 0.3.0

### GNOME Shell 48.7 (Debian 13, nested)

`gnome-shell --nested --wayland --no-x11` in a chroot on an Ubuntu 24.04 cloud machine, software rendering, with extensions written by `nsh-widgets gnome` and a store with the gallery's data. Steps and Kinds (timers, a clock, `rotate`, an `open:` button, an unknown node) were hand-written trees.

- Dashboard's popup drew the gradient banner, the progress bar, the bars, the glyphs and the buttons. "+1 day" raised the streak from 13 to 14 in the store and the popup, and queued the event in `actions.jsonl` ([dashboard-after-press.png](evidence/gnome/dashboard-after-press.png)). The bar drew Steps' `accessory` tree.
- Tasks: "Update the plan" turned on in the popup and the store ([tasks-toggled.png](evidence/gnome/tasks-toggled.png)).
- Agenda scrolled to 14:00 to 19:00 and was still there six seconds (about three polls) later ([agenda-scrolled.png](evidence/gnome/agenda-scrolled.png)).
- Setting `color-scheme` to `prefer-dark` while the popup was open redrew it dark ([dashboard-dark-live.png](evidence/gnome/dashboard-dark-live.png)). That run used GSettings' keyfile backend: with dconf in this chroot, the shell itself did not see changes made while it ran.
- Both timers counted, the clock drew, `rotate` alternated between its children, and the unknown node was skipped ([timers-clock-rotate.png](evidence/gnome/timers-clock-rotate.png)).
- A kind with no store draws "Open the app to set up this widget" ([empty-store.png](evidence/gnome/empty-store.png)).
- `open:` with a URL nothing handles closed the popup and logged a warning. A URL that has a handler was not tried.
- All six extensions went through disable and enable with no JavaScript error in the shell's log.
- Found and fixed here: a timer inside `rotate` went on writing to its destroyed label.

### GNOME Shell 45 to 51 (headless)

[`scripts/gnome-probe`](../scripts/gnome-probe), an extension that drives the others, ran inside GNOME Shell 45.0, 46.0, 47.0, 49.0, 50.1 and 51.0 (Ubuntu 23.10 to 26.10) and 48.7 (Debian 13), each `gnome-shell --headless` in a chroot. Presses there are the buttons' `clicked` signals, not a pointer. Results: [versions.txt](evidence/gnome/versions.txt); pictures: [45 to 48](evidence/gnome/versions-45-48.png), [49 to 51](evidence/gnome/versions-49-51.png). Where no emoji font was installed the flame drew as a red square.

- On every version: all six extensions active, the bar drew the `accessory` tree, "+1 day" raised the streak in the store and the popup, a task toggled, Agenda kept its place for 6.5 seconds, `prefer-dark` redrew the open popup dark, the timer counted, the empty kind said so, and disable and enable worked. No JavaScript error in any log.
- Found and fixed here: on 51, where `St.BoxLayout` has no `vertical`, nothing drew; on 45, redrawing a list aborted the shell.

### Not verified in 0.3.0

- A full GNOME session: every run was nested or headless, and only 48.7 was pressed with a pointer.
- The Counter, Focus timer, Icons and Weather kinds on GNOME: the store did not hold them.

## 24 November 2026: 0.4.0

The store and the hand-written Kinds tree are 0.3.0's. Presses here are a pointer's (`xdotool`), except GNOME's.

### Cinnamon

- 6.0.4 (Ubuntu 24.04): five desklets drew; "+1 day" moved the streak from 15 to 16 in the store and the desklet; a task toggled; Agenda scrolled and kept its place for 7 seconds; `prefer-dark` redrew them ([pressed-and-dark.png](evidence/cinnamon/pressed-and-dark.png)); `open:` with no handler logged a line; the desklets went away when removed and drew again when added back.
- 6.4.10 (Debian 13), 6.4.13 (Ubuntu 26.04) and 6.6.9 (Ubuntu 26.10), top to bottom in [versions-6.4-6.6.png](evidence/cinnamon/versions-6.4-6.6.png): the same, with no desklet error in the log. On 6.4.10 the click fell on the row above the one aimed at, and that row toggled.
- Cinnamon ran on a virtual X display, not from a login screen.

### X11 and layer-shell

- X11, Xfce 4.18 on the cloud machine's own desktop: five windows drew; the press, the toggle, the scroll and dark as above ([xfce-pressed-and-dark.png](evidence/window/xfce-pressed-and-dark.png)); a terminal raised over one covered it ([xfce-under-a-window.png](evidence/window/xfce-under-a-window.png)).
- Layer-shell, sway 1.9 nested on X11: four surfaces, the same checks ([sway-pressed-and-dark.png](evidence/window/sway-pressed-and-dark.png)).
- weston, which has no layer-shell: the script said so and exited with 1.

### KDE Plasma 5.27, full session

`startplasma-x11` on a virtual X display. The desktop form (Dashboard, Tasks, Agenda) and the panel form (Steps) drew; the press, the toggle and the scroll as above ([session-pressed.png](evidence/kde/session-pressed.png)); clicking the panel opened the full widget ([session-panel-opened.png](evidence/kde/session-panel-opened.png)); Breeze Dark redrew them ([session-dark.png](evidence/kde/session-dark.png)); removing them left the shell running.

### GNOME

The renderer was split into shared files, so the probe ran again on all seven versions, with the same results.

### Not verified in 0.4.0

- Hyprland, COSMIC, KWin and other layer-shell compositors; X11 desktops other than Xfce.
- Plasma 6 in a full session (6.3 only in `plasmoidviewer`), and a full GNOME session.
- Several monitors, and fractional scaling.

## Tests

| Command | Result |
|---|---|
| `cargo test --workspace` | 10 store tests ([`tests/store.rs`](../tests/store.rs)) and 19 CLI tests ([`tests.rs`](../crates/nsh-widgets-cli/src/tests.rs)) pass. |
| `nsh test` in `packages/nsh-widgets` | 15 tests pass: 12 in [`base.test.ts`](../packages/nsh-widgets/tests/base.test.ts), 3 in [`kit.test.ts`](../packages/nsh-widgets/tests/kit.test.ts). |
| `scripts/check-aar.sh` | The committed AAR matches its sources; a planted extra drawable made it fail, and so did an edit to a Java file (it compares a hash of the sources). |

Output: [cargo-test.txt](evidence/tests/cargo-test.txt), [nsh-test.txt](evidence/tests/nsh-test.txt). Each suite was seen to fail once against deliberately broken code: in 0.1.0 the action inbox, the `nsh.json` writer, the action check in `base.ts` and the toggle in the text renderer; in 0.2.0 removing `list` from the store's node types, dropping the tile entries from the generated Android manifest, and one break for each other new test ([fail-once-0.2.0.txt](evidence/tests/fail-once-0.2.0.txt)). The 0.4.0 test was seen to fail five ways ([fail-once-0.4.0.txt](evidence/tests/fail-once-0.4.0.txt)) and the 0.3.0 test five ([fail-once-0.3.0.txt](evidence/tests/fail-once-0.3.0.txt)). The Swift, Java, QML, St and GTK renderers are covered only by the runs above. CI is defined in `.github/workflows/ci.yml`.

## Not done

- A Windows widget host and a COSMIC applet; `show` covers terminals and Waybar
- Tests on physical devices
- A full Plasma 6 session and a full GNOME session
- macOS runs of options and lists
- A crates.io and npm release (NSH installs from the git tags)

## Release

- Tag `v0.3.0` on `d1a4d28`; CI run https://github.com/SuperNSH/widgets/actions/runs/45334884052 passed all 12 jobs. In a blank app, `nsh add github:SuperNSH/widgets@^0.3` resolved to 0.3.0 at that commit and `nsh check` passed. The released `nsh-widgets` for arm64 macOS and x86_64 Linux (SHA-256 checked) each wrote a GNOME extension.
- Tag `v0.2.0` on `1372cce`; CI run https://github.com/SuperNSH/widgets/actions/runs/45327392050 passed all 12 jobs. In a blank app, `nsh add github:SuperNSH/widgets@^0.2` resolved to 0.2.0 at that commit, `nsh check` passed, and the released `nsh-widgets` (aarch64 macOS, SHA-256 checked) built a kind with options, a list, a Live Activity and a control; `nsh package` built the iOS simulator app and the Android APK.
.1.0

- Tag `v0.1.0` on `fe8cb17`. In a blank app, `nsh add github:SuperNSH/widgets@^0.1` resolved to 0.1.0 at that commit, `nsh check` type-checked imports of `nsh-widgets` and `nsh-widgets/kit`, `nsh-widgets init` and `build` ran, and `nsh package` built both the Android APK (54 `nsh_widgets` entries) and the iOS simulator app (with `NshWidgets.appex`). On the emulator the installed APK registered `NshWidgetProvider$Slot1` as a widget provider.
- [Release v0.1.0](https://github.com/SuperNSH/widgets/releases/tag/v0.1.0) carries `nsh-widgets` for aarch64 and x86_64 macOS, x86_64 Linux and x86_64 Windows, each with a SHA-256 file. The aarch64 macOS archive was downloaded, checked against its SHA-256 and run (`help`, `show`).
- CI on `fe8cb17`: run https://github.com/SuperNSH/widgets/actions/runs/45248139253 passed all 13 jobs: format; rust on ubuntu, macos and windows; clippy for aarch64-apple-ios, aarch64-apple-ios-sim, aarch64-linux-android and wasm32-unknown-unknown; the AAR check; the extension build on macOS; the gallery type-check on NSH nightly; and the TypeScript checks.
