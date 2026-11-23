# Capabilities by platform

One tree, three renderers. iOS and macOS share the SwiftUI renderer ([`NshWidgets.swift`](../crates/nsh-widgets-cli/renderer/NshWidgets.swift)); Android has the RemoteViews renderer ([`NshWidgets.java`](../crates/nsh-widgets/android/src/dev/supernsh/widgets/NshWidgets.java)). This page lists where they differ and why.

The iOS and Android columns were seen on the iOS 27 simulator and an Android 17 (API 37) emulator, or read from the platform's API where noted. The macOS column follows from the shared renderer; this library build has not been placed on a Mac, so it is unverified. See [status.md](status.md).

| Feature | iOS | Android | macOS |
|---|---|---|---|
| Layout (`col`, `row`, `spacer`, `gap`, `pad`) | `VStack`, `HStack`, `Spacer` | `LinearLayout` layouts; gaps and spacers are `FrameLayout`s | as iOS |
| Text | one line, scales down to 60% before truncating | one line, ellipsized | as iOS |
| Light and dark colours | both, from the colour scheme | both on API 31+ (`setColorInt`); before that, the colour for the mode at draw time | as iOS |
| Corner radius | yes | API 31+ | as iOS |
| Gradients | real `LinearGradient` | the gradient's first colour; RemoteViews cannot draw gradients from code | as iOS |
| `animate-number` | `contentTransition(.numericText())` | none | as iOS |
| `progress` | capsule track and fill | `ProgressBar`; colours and thickness on API 31+ | as iOS |
| `bars` | rounded rectangles | `FrameLayout`s sized by padding, rounded on API 31+ | as iOS |
| Icons (`star`, `bolt`, `check`, `play`, `flame`) | SF Symbols; unknown names draw a question mark | vector drawables in the AAR; unknown names draw the star; sizing on API 31+ | as iOS |
| `timer` | `Text(timerInterval:)` and `.timer` style | `Chronometer` | as iOS |
| `clock` | `Text(date, style: .time)`, one timeline entry per minute | `TextClock` | as iOS |
| Timers and clock with the app closed | tick | tick; seen 04:03 to 04:07 with the app idle | as iOS |
| `rotate` | the timeline has ten one-minute entries, so it steps at most once a minute, whatever `interval` says | `ViewFlipper` slides on its own every `interval` seconds | as iOS |
| `toggle` | a checklist row (check circle and label, struck through when on) pressed through an App Intent | API 31+: a real `Switch` via `setCompoundButtonChecked` and `setOnCheckedChangeResponse`; older: a text row with a check mark | as iOS |
| Buttons (`inc`, `dec`, `toggle`) | `Button(intent:)` runs an App Intent in the extension | `PendingIntent` to the non-exported `NshWidgetsActionReceiver` in the app's process | as iOS |
| `open:` buttons and the widget `url` | `Link`, `widgetURL` | `ACTION_VIEW` intent limited to the app's package | as iOS |
| Fonts | bundled fonts work (`UIAppFonts`) | system `sans-serif` only, see below | bundled fonts work (`ATSApplicationFontsPath`) |
| Font weight | `Font.Weight` | `textFontWeight` (API 28+) with a bold fallback, one layout per weight | as iOS |
| Sizes | families `small`, `medium`, `large`, `extraLarge` (iPad) | cells from `widgets.json`; the family name for `hide` comes from `OPTION_APPWIDGET_MIN_WIDTH`: under 200 dp small, under 450 medium, else large | as iOS, with `extraLarge` |
| Where data lives | App Group container | the app's files directory | App Group container (`macosAppGroup`) |
| App hears presses live | Darwin notification `<group>.nsh.widgets.action`, then the data is re-read | the receiver runs in the app's process; `data.json`'s modification time changes | as iOS |

## Platform limits

**Android fonts.** Android widgets cannot load app fonts. Bundled `res/font` fonts and downloadable fonts both failed, because RemoteViews inflate in a restricted context, and RemoteViews has no `setTypeface` (checked with `javap` on the SDK 36 and 37 jars). The renderer uses `sans-serif` with `textFontWeight`. `RemoteViews.DrawInstructions` (API 35, Remote Compose) may lift this later; it is a watch item.

**Android layout classes.** Widget layouts may use only RemoteViews-whitelisted classes. That is why gaps and bars are `FrameLayout`, not `View`.

**Android force-stop.** Force-stopping the app greys out its widgets until the app is opened again.

**iOS toggles.** A widget `Toggle` draws only a tinted pill, and forcing `.switch` shows the unsupported placeholder. The checklist row reads clearly in both states.

**iOS gallery cache.** New kinds appear in the widget gallery only after restarting the simulator (SpringBoard) when the gallery cached the old kinds. The simulator sometimes redraws one press late.

**macOS.** Needs a real signing identity (a free Personal Team works) and a team-prefixed App Group in `macosAppGroup`. The app must run from a LaunchServices-indexed place such as `~/Applications`, or App Intents fail with "There is no metadata". New kinds need a `CFBundleVersion` bump (`build` in `nsh.json`). The extension is built for arm64 only.

**Linux, Windows, web.** No widget host. The store works through `NSH_WIDGETS_DIR`; `reload` does nothing, and without the variable every call fails with `unsupported`.
