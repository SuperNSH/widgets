# Widget Tree IR v1

The app writes one snapshot per kind. The renderers draw whatever tree it holds and fill it from the kind's data. Sources of truth: the types in [`packages/nsh-widgets/src/base.ts`](../packages/nsh-widgets/src/base.ts), the check in [`crates/nsh-widgets/src/store.rs`](../crates/nsh-widgets/src/store.rs) (`check_snapshot`), the Swift renderer [`NshWidgets.swift`](../crates/nsh-widgets-cli/renderer/NshWidgets.swift) and the Java renderer [`NshWidgets.java`](../crates/nsh-widgets/android/src/dev/supernsh/widgets/NshWidgets.java).

## Files

Under `<root>/nsh/widgets/`:

| File | Written by | Content |
|---|---|---|
| `<kind>/snapshot.json` | the app | The snapshot below. |
| `<kind>/data.json` | the app and widget buttons | `{"schema":1,"data":{...}}`. `data` is a JSON object. |
| `actions.jsonl` | widget buttons (appended) | One event per press: `{"id","kind","action","at"}`, `at` in epoch milliseconds. The app drains it with `take_actions`. |

`<root>` is the App Group container on iOS and macOS, the app's files directory on Android, and `NSH_WIDGETS_DIR` elsewhere. Every write of `snapshot.json` and `data.json` goes to a `.tmp` file first and is renamed into place.

`take_actions` renames `actions.jsonl` to `actions.jsonl.taking`, reads it and deletes it, so a press that lands meanwhile starts a new inbox. Widget events carry a UUID `id`.

## Snapshot

```json
{ "v": 1, "font": "Onest", "url": "myapp://widget/Counter", "root": { "t": "col", "children": [] } }
```

| Key | Required | Meaning |
|---|---|---|
| `v` | yes | Must be `1`. |
| `root` | yes | The root node. |
| `font` | no | A font family bundled through `widgets.json` `fonts`. iOS and macOS only. |
| `url` | no | Opened when the widget itself is tapped. |
| `accessory` | no | The tree for the iOS lock-screen families (`circular`, `rectangular`, `inline`); `root` when absent. Checked like `root`. |

`set_ui` refuses a snapshot whose `v` is not 1, that has no `root`, that has a node without `t` or with an unknown `t`, or whose button action is not `open:...` and not in the action vocabulary.

The root node's `bg` or `gradient` becomes the widget's container background, and its `pad` the widget's padding (16 points when it has none), on every platform. iOS and macOS turn WidgetKit's own content margins off so the padding matches Android. On the lock screen the system draws the background and the padding is 0.

## Nodes

Every node has `t` and optional `style`. Other keys by type:

| `t` | Keys | TS | Draws |
|---|---|---|---|
| `col` | `children` | `Col(cls, ...children)` | A vertical stack. |
| `row` | `children` | `Row(cls, ...children)` | A horizontal stack. |
| `text` | `text` | `Text(cls, text)` | One line of text, with `{key}` templates. |
| `button` | `text`, `action` | `Button(cls, action, label)`, `Link(cls, url, label)` | A pressable label. `Button` takes only `inc:`, `dec:`, `toggle:`; `Link` writes `open:<url>`. |
| `spacer` | | `Spacer(cls?)` | Flexible space. |
| `progress` | `key` | `Progress(cls, key)` | A bar filled to `data[key]`, clamped to 0..1. `fg` is the fill, `bg` the track, `height` the thickness. |
| `bars` | `key` | `Bars(cls, key)` | A bar chart of the numbers in `data[key]`, bottom-aligned, scaled to the largest. `height` is the chart height (default 48), `gap` the spacing (default 4). |
| `icon` | `name` | `Icon(cls, name)` | One of the shared set (`ICONS`): `star`, `bolt`, `check`, `play`, `pause`, `flame`, `plus`, `minus`, `arrow-up`, `arrow-down`, `heart`, `home`, `person`, `bell`, `clock`, `calendar`, `cloud`, `sun`, `moon`, `drop`, `music`. Size from `height`, else `size`, else 18. |
| `timer` | `key`, `mode` | `Timer(cls, key, mode)` | Time since (`up`) or until (`down`) the epoch milliseconds in `data[key]`, ticked by the system. |
| `clock` | | `Clock(cls)` | The current time, kept live by the system. |
| `toggle` | `key`, `text`, `action` | `Toggle(cls, key, label)` | A switch bound to the boolean `data[key]`. The action is `toggle:<key>`. |
| `rotate` | `interval`, `children` | `Rotate(cls, interval, ...children)` | One child at a time, changing every `interval` seconds where the platform can. |

A renderer draws an unknown type as nothing, but `set_ui` refuses one first.

## Style

| Key | Type | Meaning |
|---|---|---|
| `fg` | `[light, dark]` | Text, icon, fill colour. `#RRGGBB`. |
| `bg` | `[light, dark]` | Background colour. On `progress` it is the track. |
| `size` | number | Text size in points (sp on Android). Default 14. |
| `weight` | `regular`, `medium`, `semibold`, `bold` | Text weight. |
| `pad` | `[top, right, bottom, left]` | Padding in points. |
| `gap` | number | Space between children. |
| `radius` | number | Corner radius, capped at 200. |
| `align` | `start`, `center`, `end` | Cross-axis alignment: horizontal in a `col`, vertical in a `row`. |
| `grow` | boolean | A `col` fills the remaining height. |
| `hide` | list of families | Leaves the node out of `small`, `medium` or `large` widgets, or of the lock-screen `circular`, `rectangular` or `inline` ones. |
| `gradient` | `{from, to, dir}` | A linear gradient background. `from` and `to` are pairs; `dir` is `b` (down), `r` (right) or `br` (diagonal). |
| `animate` | `"number"` | Animate changes of the text's value where the platform can. |
| `height` | number | Bar chart height, progress thickness, icon size. |

## Templates

`text` on `text`, `button` and `toggle` nodes may hold `{key}`. Each top-level key of the data replaces `{key}`; a whole number prints without decimals. Keys that are not in the data stay as written.

## Actions

| Action | Effect on `data[key]` |
|---|---|
| `inc:key` | integer + 1 (missing counts as 0) |
| `dec:key` | integer - 1 (missing counts as 0) |
| `toggle:key` | boolean flipped (missing counts as false) |
| `open:<url>` | none; opens the URL |

`key` starts with a letter or `_` and holds letters, digits, `_` or `.`. A dot does not mean nesting: `a.b` is one top-level key. The widget runs `inc`, `dec` and `toggle` itself, rewrites `data.json`, appends an event to `actions.jsonl` and redraws. On Apple platforms it also posts the Darwin notification `<appGroup>.nsh.widgets.action`, which a running app's `next_change` / `onChange` listens for.

## Classes

The TS primitives take a class string first. `tw()` resolves this Tailwind subset:

| Class | Sets |
|---|---|
| `text-xs` ... `text-5xl` | `size`: 12, 14, 16, 18, 20, 24, 30, 36, 48 |
| `font-normal`, `font-medium`, `font-semibold`, `font-bold` | `weight` |
| `text-<colour>`, `bg-<colour>` | `fg`, `bg` |
| `dark:text-<colour>`, `dark:bg-<colour>` | the dark half of `fg`, `bg` |
| `p-N`, `px-N`, `py-N`, `pt-N`, `pr-N`, `pb-N`, `pl-N` | `pad`, N x 4 points; halves like `1.5` work |
| `gap-N` | `gap`, N x 4 |
| `rounded`, `rounded-none|sm|md|lg|xl|2xl|3xl|full` | `radius`: 4, 0, 2, 6, 8, 12, 16, 24, 999 |
| `items-start|center|end` | `align` |
| `grow`, `flex-1` | `grow` |
| `bg-gradient-to-b|r|br`, `from-<colour>`, `to-<colour>`, with `dark:` | `gradient`; needs a direction and both ends |
| `animate-number` | `animate: "number"` |
| `h-N` | `height`, N x 4 |
| `small:hidden`, `medium:hidden`, `large:hidden`, `circular:hidden`, `rectangular:hidden`, `inline:hidden`, `hidden` | `hide` |

Colours are `white`, `black`, the palettes `slate`, `neutral`, `blue`, `emerald`, `amber`, `rose`, `violet` at shades 50 to 950, and theme tokens. Tokens carry both a light and a dark value. `defaultTheme` has `surface`, `surface-2`, `text`, `muted`, `primary`, `on-primary`; `setTheme(theme)` adds to or overrides them. `setFont(family)` sets the snapshot's `font`.

Classes `tw()` does not know are pushed to `unknownClasses`, never dropped silently.

## Kit

[`kit.ts`](../packages/nsh-widgets/src/kit.ts) builds styled components from base primitives only:

| Component | Is |
|---|---|
| `Card(...children)` | `Col("bg-surface p-4 gap-1 grow")` |
| `Header(title, trailing?)` | A small muted title row; `trailing` is hidden in small widgets. |
| `Stat(value, caption?)` | A large primary number with a caption. |
| `ActionButton(label, action, { variant, size, hide })` | A `Button`. Variants `primary` (default), `secondary`, `accent`, `ghost`; sizes `sm`, `md` (default). |
| `LinkButton(label, url, { variant, size, hide })` | A `Link`, variant `accent` by default. |
| `ButtonRow(buttons, trailing?)` | Buttons in a row, `trailing` at the far end. |
| `Gap()` | A `Spacer`. |

`kitTheme` adds the `accent` and `on-accent` tokens; pass it to `setTheme`. `kitFont` is `"Onest"`, for apps that bundle it.
