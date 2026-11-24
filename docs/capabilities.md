# Capabilities by platform

One tree, five native renderers. iOS and macOS share the SwiftUI renderer ([`NshWidgets.swift`](../crates/nsh-widgets-cli/renderer/NshWidgets.swift), with [`NshActivities.swift`](../crates/nsh-widgets-cli/renderer/NshActivities.swift) for Live Activities); Android has the RemoteViews renderer ([`NshWidgets.java`](../android/src/dev/supernsh/widgets/NshWidgets.java)); KDE Plasma has a QML renderer ([`NshNode.qml`](../crates/nsh-widgets-cli/plasma/NshNode.qml), see [below](#kde-plasma)); GNOME Shell and Cinnamon have an St renderer ([`st.js`](../crates/nsh-widgets-cli/js/st.js), see [below](#gnome-shell)); X11 and layer-shell desktops have a GTK one ([`gtk.js`](../crates/nsh-widgets-cli/js/gtk.js)). This page lists where they differ and why.

The iOS and Android columns were seen on the iOS 27 simulator and an Android 17 (API 37) emulator, or read from the platform's API where noted. The macOS column follows from the shared renderer; the 0.1.0 features were seen on a Mac desktop, the 0.2.0 ones (options, lists) were not. See [status.md](status.md).

| Feature | iOS | Android | macOS |
|---|---|---|---|
| Layout (`col`, `row`, `spacer`, `gap`, `pad`) | `VStack`, `HStack`, `Spacer` | `LinearLayout` layouts; gaps and spacers are `FrameLayout`s | as iOS |
| Text | one line, scales down to 60% before truncating | one line, ellipsized | as iOS |
| Light and dark colours | both, from the colour scheme | both on API 31+ (`setColorInt`); before that, the colour for the mode at draw time | as iOS |
| Corner radius | yes | API 31+ | as iOS |
| Gradients | real `LinearGradient` | drawn into a small bitmap that an `ImageView` stretches behind the content (RemoteViews has no gradient drawable it can set from code); follows dark mode at the next redraw; other nodes than containers fall back to the first colour | as iOS |
| `animate-number` | `contentTransition(.numericText())` | none | as iOS |
| `progress` | capsule track and fill | `ProgressBar`; colours and thickness on API 31+ | as iOS |
| `bars` | rounded rectangles | `FrameLayout`s sized by padding, rounded on API 31+ | as iOS |
| Icons (21 shared names, see [ir.md](ir.md)) | SF Symbols; unknown names draw a question mark | vector drawables in the AAR; unknown names draw the star; sizing on API 31+ | as iOS |
| `timer` | `Text(timerInterval:)` and `.timer` style | `Chronometer` | as iOS |
| `clock` | `Text(date, style: .time)`, one timeline entry per minute | `TextClock` | as iOS |
| Timers and clock with the app closed | tick | tick; seen 04:03 to 04:07 with the app idle | as iOS |
| `rotate` | the timeline has ten one-minute entries, so it steps at most once a minute, whatever `interval` says | `ViewFlipper` slides on its own every `interval` seconds | as iOS |
| `toggle` | a checklist row (check circle and label, struck through when on) pressed through an App Intent | API 31+: a real `Switch` via `setCompoundButtonChecked` and `setOnCheckedChangeResponse`; older: a text row with a check mark | as iOS |
| Buttons (`inc`, `dec`, `toggle`) | `Button(intent:)` runs an App Intent in the extension | `PendingIntent` to the non-exported `NshWidgetsActionReceiver` in the app's process | as iOS |
| `open:` buttons and the widget `url` | `Link`, `widgetURL` | `ACTION_VIEW` intent limited to the app's package | as iOS |
| Fonts | bundled fonts work (`UIAppFonts`) | system `sans-serif` only, see below | bundled fonts work (`ATSApplicationFontsPath`) |
| Font weight | `Font.Weight` | `textFontWeight` (API 28+) with a bold fallback, one layout per weight | as iOS |
| Sizes | families `small`, `medium`, `large`, `extraLarge` (iPad), and the lock-screen `circular`, `rectangular`, `inline` with their own `accessory` tree | cells from `widgets.json`; the family name for `hide` comes from `OPTION_APPWIDGET_MIN_WIDTH`: under 200 dp small, under 450 medium, else large | as iOS, with `extraLarge` |
| Lock screen | `accessory` tree; the system draws it monochrome (vibrant); `circular` is centred on `AccessoryWidgetBackground` | none | none |
| Live Activities | iOS 16.2+: lock-screen banner and expanded Dynamic Island draw `root`, compact and minimal draw `accessory`; data is the activity's over the kind's | none (`unsupported`) | none (`unsupported`) |
| Options (user-configured widgets) | edit sheet from a generated `AppEnum` and `WidgetConfigurationIntent` | configure screen (`NshWidgetConfigureActivity`), reopened from the launcher's "Settings"; placed with the first value on Android 12+ | as iOS |
| `list` | the first `max` items (WidgetKit cannot scroll) | API 31+: a scrolling list of every item (`RemoteCollectionItems`), one per widget; older: the first `max` | as iOS |
| Controls | iOS 18+: Control Center and lock-screen controls (`ControlWidgetToggle`, `ControlWidgetButton`) | Quick Settings tiles (`NshTileService$Slot1` to `$Slot8`); a toggle tile is active while its key is true | none |
| Padding | the root's `pad`, 16 by default; content margins off | the root's `pad`, 16 by default | as iOS |
| Where data lives | App Group container | the app's files directory | App Group container (`macosAppGroup`) |
| App hears presses live | Darwin notification `<group>.nsh.widgets.action`, then the data is re-read | the receiver runs in the app's process; `data.json`'s modification time changes | as iOS |

## Platform limits

**Android fonts.** Android widgets cannot load app fonts. Bundled `res/font` fonts and downloadable fonts both failed, because RemoteViews inflate in a restricted context, and RemoteViews has no `setTypeface` (checked with `javap` on the SDK 36 and 37 jars). The renderer uses `sans-serif` with `textFontWeight`. `RemoteViews.DrawInstructions` (API 35, Remote Compose) may lift this later; it is a watch item.

**Android layout classes.** Widget layouts may use only RemoteViews-whitelisted classes. That is why gaps and bars are `FrameLayout`, not `View`.

**Android force-stop.** Force-stopping the app greys out its widgets until the app is opened again.

**iOS toggles.** A widget `Toggle` draws only a tinted pill, and forcing `.switch` shows the unsupported placeholder. The checklist row reads clearly in both states.

**iOS Live Activities.** iOS asks the user once whether to allow Live Activities from the app. In the expanded Dynamic Island the tree draws on black with 8 points of side padding; the island's curved corners can still clip the first characters of a line at its edges (seen with "Arrives in" in [activity-island-expanded.png](evidence/ios/activity-island-expanded.png)). On the lock screen iOS drew the banner's countdown as "19:––" ([activity-lock-screen.png](evidence/ios/activity-lock-screen.png)); the island showed full seconds.

**Android list.** RemoteViews takes a collection adapter only on the views given to `updateAppWidget`, not on nested ones, so the renderer sets the list's `RemoteCollectionItems` on the root views. That allows one scrolling list per widget.

**Android tiles.** A tile carries the control's shared icon and name. Taps sent with `adb shell input tap` on a tile did not register on the Android 17 emulator; `adb shell cmd statusbar click-tile` did, and runs the same `TileService.onClick`. A finger tap on a device is untested.

**iOS gallery cache.** New kinds appear in the widget gallery only after restarting the simulator (SpringBoard) when the gallery cached the old kinds. The simulator sometimes redraws one press late.

**macOS.** Needs a real signing identity (a free Personal Team works) and a team-prefixed App Group in `macosAppGroup`. The app must run from a LaunchServices-indexed place such as `~/Applications`, or App Intents fail with "There is no metadata". New kinds need a `CFBundleVersion` bump (`build` in `nsh.json`). The extension is built for arm64 only.

**Linux and Windows.** KDE Plasma draws widgets through a generated plasmoid, GNOME Shell through an extension, Cinnamon through a desklet and other desktops through a window (all below); Windows has no widget host for them. Once the app calls `configure(name)`, the store lives in `$XDG_DATA_HOME/nsh-widgets/<name>` (`~/.local/share/...` without it, `%LOCALAPPDATA%` on Windows), or in `NSH_WIDGETS_DIR` when set. `reload` does nothing. `nsh-widgets show KIND --group <name> [--follow] [--format waybar]` draws a widget as text (progress as `▓░`, bars as a sparkline, toggles as `☑`/`☐`, timers counting) or as Waybar JSON, whose bar text comes from the `accessory` tree. Without a name or the variable, calls fail with `no_group_configured`.

**Web.** No store: calls fail with `no_group_configured`, or `unsupported` once a name is configured.

## KDE Plasma

`nsh-widgets plasmoid` writes a Plasma applet whose QML renderer reads the store through `nsh-widgets show --format ir` every two seconds and runs presses through `nsh-widgets apply`. Seen on Plasma 5.27 and 6.3, in full sessions and in `plasmoidviewer`; in a panel it draws the `accessory` tree (since 0.2.1).

| Feature | KDE Plasma |
|---|---|
| Layout | a `GridLayout` per `col` or `row` (one column or one row), children loaded from `NshNode.qml` itself; rows align their children to the vertical centre |
| Light and dark colours | dark when the Kirigami theme's background is dark |
| Root background and padding | the root's `bg` or gradient fills a rounded frame (radius 16) behind the content; the root's `pad`, 16 by default |
| Gradients | `r` horizontal; `b` and `br` both vertical (QML's `Gradient` has no diagonal) |
| Text | `Text`; a bundled font applies only if it is installed on the machine |
| Icons | Unicode and emoji glyphs (★ ⚡ ✓ ...), sized like the other platforms |
| `progress`, `bars` | rounded rectangles |
| `timer`, `clock` | counted in QML each second (`h:mm:ss` or `m:ss`, `hh:mm`) |
| `rotate` | changes child every `interval` seconds |
| `toggle` | a ☑ or ☐ row, struck through when on |
| Buttons | `inc`, `dec`, `toggle` run `nsh-widgets apply`, which changes the data and queues the event; `open:` opens the URL |
| `list` | a scrolling `ListView` of the first `max` items |
| `hide` | `small` under 200 pixels wide, `medium` under 450, else `large`; `inline` in a panel |
| In a panel | the `accessory` tree; clicking opens the full widget |
| Not supported | the widget `url`, options, Live Activities, controls |
| Updates | polled every two seconds, so a change in the app shows within about two seconds |

## GNOME Shell

`nsh-widgets gnome` writes a GNOME Shell extension whose St renderer ([`st.js`](../crates/nsh-widgets-cli/js/st.js)) reads the store as the plasmoid does. Run on GNOME Shell 45 to 51, headless, and on 48.7 nested in a window.

| What | On GNOME Shell |
|---|---|
| Where | a top-bar button drawing the `accessory` tree (the kind's name without one), opening a popup with the `root` tree, 320 pixels wide |
| Layout | an `St.BoxLayout` per `col` or `row`; rows align their children to the vertical centre |
| Light and dark colours | the popup follows Dark Style while open; the bar always uses the dark colours, since GNOME's bar is dark |
| Root background and padding | the root's `bg` or gradient fills a rounded frame (radius 12); the root's `pad`, 16 by default |
| Gradients | `r` horizontal; `b` and `br` both vertical |
| Text | `St.Label` in the shell's font; bundled fonts are not used |
| Icons | Unicode and emoji glyphs; the emoji ones need an emoji font |
| `progress`, `bars` | rounded rectangles |
| `timer`, `clock` | counted each second (`h:mm:ss` or `m:ss`, `hh:mm`) |
| `rotate` | changes child every `interval` seconds |
| `toggle` | a ☑ or ☐ row, struck through when on |
| Buttons | `inc`, `dec`, `toggle` run `nsh-widgets apply`; `open:` opens the URL and closes the popup. A URL nothing handles is logged |
| `list` | a scrolling list of the first `max` items, 160 pixels high at most, that keeps its place between updates |
| `hide` | `medium` in the popup, `inline` in the bar |
| Not supported | the widget `url`, options, Live Activities, controls |
| Updates | polled every two seconds |

## Cinnamon, X11 and layer-shell

The Cinnamon desklet draws with the GNOME renderer ([`st.js`](../crates/nsh-widgets-cli/js/st.js)), so the table above holds for it, except that the `root` tree is on the desktop, there is no bar form, and light or dark follows Cinnamon. Run on Cinnamon 6.0, 6.4 and 6.6.

`nsh-widgets window` draws with GTK 3 ([`gtk.js`](../crates/nsh-widgets-cli/js/gtk.js)): the same limits, except that `br` gradients are diagonal. It draws only the `root` tree. Run on Xfce (X11), sway, KWin and labwc (layer-shell).

**Preview.** `nsh-widgets preview` draws the IR as HTML with the same rules (sizes per family, root background and padding, templates, timers at the current time). It is a check, not a fourth renderer: the native renderers are the reference.
