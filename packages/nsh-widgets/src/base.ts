// nsh-widgets/base: the widget primitives, with no look of their own (as nsh/base is to NSH apps and
// gpui-base to GPUI Kit). It holds what every platform's renderer understands and nothing more:
// layout (Col, Row, Spacer), Text, and the two kinds of press (Button runs an action, Link opens the
// app at a URL); the class resolver; the theme's semantic tokens; and the Widget Tree IR v1 the app
// writes as snapshot.json. Looks live in nsh-widgets/kit or in the app's own components, built from these.
//
// Text may hold {key} templates the renderer fills from data.json, so a button that changes data.json
// redraws without the app. Button actions are the vocabulary a widget runs alone: "inc:key",
// "dec:key", "toggle:key". Classes: the Tailwind subset below, "dark:" variants, theme tokens, and
// "small:hidden" / "medium:hidden" / "large:hidden" to leave a node out of a widget size.

export type Pair = [string, string]; // [light, dark] "#RRGGBB"
export type Style = {
  fg?: Pair;
  bg?: Pair;
  size?: number; // points (sp on Android)
  weight?: "regular" | "medium" | "semibold" | "bold";
  pad?: [number, number, number, number]; // top, right, bottom, left
  gap?: number;
  radius?: number;
  align?: "start" | "center" | "end";
  grow?: boolean;
  hide?: string[]; // families this node is left out of: "small", "medium", "large", and on the iOS lock screen "circular", "rectangular", "inline"
  gradient?: { from: Pair; to: Pair; dir: "b" | "r" | "br" }; // falls back to `from` where a platform has no gradients
  animate?: "number"; // animate value changes where the platform can (iOS numeric transition)
  height?: number; // points: bar chart height, progress thickness, icon size
};
export type Node = {
  t: "col" | "row" | "text" | "button" | "spacer" | "progress" | "bars" | "icon" | "timer" | "clock" | "toggle" | "rotate";
  key?: string; // data.json key a node reads: progress value, bar values, timer date, toggle state
  mode?: "up" | "down"; // timer direction
  name?: string; // icon name from the shared icon set
  interval?: number; // rotate: seconds per child
  style?: Style;
  text?: string;
  action?: string;
  children?: Node[];
};

const palette: Record<string, string[]> = {
  slate: ["f8fafc", "f1f5f9", "e2e8f0", "cbd5e1", "94a3b8", "64748b", "475569", "334155", "1e293b", "0f172a", "020617"],
  neutral: ["fafafa", "f5f5f5", "e5e5e5", "d4d4d4", "a3a3a3", "737373", "525252", "404040", "262626", "171717", "0a0a0a"],
  blue: ["eff6ff", "dbeafe", "bfdbfe", "93c5fd", "60a5fa", "3b82f6", "2563eb", "1d4ed8", "1e40af", "1e3a8a", "172554"],
  emerald: ["ecfdf5", "d1fae5", "a7f3d0", "6ee7b7", "34d399", "10b981", "059669", "047857", "065f46", "064e3b", "022c22"],
  amber: ["fffbeb", "fef3c7", "fde68a", "fcd34d", "fbbf24", "f59e0b", "d97706", "b45309", "92400e", "78350f", "451a03"],
  rose: ["fff1f2", "ffe4e6", "fecdd3", "fda4af", "fb7185", "f43f5e", "e11d48", "be123c", "9f1239", "881337", "4c0519"],
  violet: ["f5f3ff", "ede9fe", "ddd6fe", "c4b5fd", "a78bfa", "8b5cf6", "7c3aed", "6d28d9", "5b21b6", "4c1d95", "2e1065"],
};
const shades = [50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950];

// Theme tokens: one name, a light and a dark value. An app passes its own theme.
export type Theme = Record<string, Pair>;
export const defaultTheme: Theme = {
  surface: ["#ffffff", "#171717"],
  "surface-2": ["#f1f5f9", "#262626"],
  text: ["#0f172a", "#fafafa"],
  muted: ["#64748b", "#a3a3a3"],
  primary: ["#2563eb", "#60a5fa"],
  "on-primary": ["#ffffff", "#0a0a0a"],
};

let current: Theme = defaultTheme;
/** Sets the semantic tokens classes resolve against: the app's own theme, or a kit's. */
export function setTheme(theme: Theme) {
  current = { ...defaultTheme, ...theme };
}
let font: string | undefined;
/**
 * The widget's font family. Undefined means each platform's system font (a native look); a name means
 * that font file is bundled into every widget (an identical look). Kits and apps set it with the theme.
 */
export function setFont(family: string | undefined) {
  font = family;
}
/** Classes no renderer could draw, collected so the app or `nsh-widgets check` can report them. */
export const unknownClasses: string[] = [];

const sizes: Record<string, number> = { xs: 12, sm: 14, base: 16, lg: 18, xl: 20, "2xl": 24, "3xl": 30, "4xl": 36, "5xl": 48 };
const radii: Record<string, number> = { none: 0, sm: 2, "": 4, md: 6, lg: 8, xl: 12, "2xl": 16, "3xl": 24, full: 999 };

function color(name: string, theme: Theme): string | Pair | undefined {
  if (theme[name]) return theme[name];
  if (name === "white") return "#ffffff";
  if (name === "black") return "#000000";
  const m = /^([a-z]+)-(\d+)$/.exec(name);
  if (m && palette[m[1]]) {
    const i = shades.indexOf(Number(m[2]));
    if (i >= 0) return "#" + palette[m[1]][i];
  }
  return undefined;
}

// Light and dark slots fill separately: "bg-white dark:bg-neutral-900", or a theme token that has both.
type Slots = { fg: [string?, string?]; bg: [string?, string?] };
function setColor(slots: Slots, key: "fg" | "bg", value: string | Pair, dark: boolean) {
  const [l, d] = Array.isArray(value) ? value : [value, value];
  if (dark) slots[key][1] = d;
  else {
    slots[key][0] = l;
    if (slots[key][1] === undefined || Array.isArray(value)) slots[key][1] = d;
  }
}

/** Resolves the supported Tailwind subset. Unknown classes are reported, never silently dropped. */
export function tw(classes: string, theme: Theme = current, unknown: string[] = unknownClasses): Style {
  const s: Style = {};
  const slots: Slots = { fg: [], bg: [] };
  const grad: { from?: Pair; to?: Pair; dir?: "b" | "r" | "br" } = {};
  const pad = [0, 0, 0, 0] as [number, number, number, number];
  let padded = false;
  for (const raw of classes.split(/\s+/).filter(Boolean)) {
    const fam = /^(small|medium|large|circular|rectangular|inline):hidden$/.exec(raw);
    if (fam || raw === "hidden") {
      (s.hide ??= []).push(...(fam ? [fam[1]] : ["small", "medium", "large"]));
      continue;
    }
    const dark = raw.startsWith("dark:");
    const c = dark ? raw.slice(5) : raw;
    let m: RegExpExecArray | null;
    if ((m = /^text-(xs|sm|base|lg|xl|2xl|3xl|4xl|5xl)$/.exec(c))) s.size = sizes[m[1]];
    else if ((m = /^font-(normal|medium|semibold|bold)$/.exec(c))) s.weight = m[1] === "normal" ? "regular" : (m[1] as Style["weight"]);
    else if ((m = /^text-(.+)$/.exec(c)) && color(m[1], theme)) setColor(slots, "fg", color(m[1], theme)!, dark);
    else if ((m = /^bg-(.+)$/.exec(c)) && color(m[1], theme)) setColor(slots, "bg", color(m[1], theme)!, dark);
    else if ((m = /^p([xytrbl]?)-(\d+(?:\.5)?)$/.exec(c))) {
      const v = Number(m[2]) * 4;
      padded = true;
      const sides = { "": [0, 1, 2, 3], x: [1, 3], y: [0, 2], t: [0], r: [1], b: [2], l: [3] }[m[1]]!;
      for (const i of sides) pad[i] = v;
    } else if ((m = /^gap-(\d+(?:\.5)?)$/.exec(c))) s.gap = Number(m[1]) * 4;
    else if ((m = /^rounded(?:-(none|sm|md|lg|xl|2xl|3xl|full))?$/.exec(c))) s.radius = radii[m[1] ?? ""];
    else if ((m = /^items-(start|center|end)$/.exec(c))) s.align = m[1] as Style["align"];
    else if (c === "grow" || c === "flex-1") s.grow = true;
    else if ((m = /^bg-gradient-to-(b|r|br)$/.exec(c))) grad.dir = m[1] as "b" | "r" | "br";
    else if ((m = /^(from|to)-(.+)$/.exec(c)) && color(m[2], theme)) {
      const v = color(m[2], theme)!;
      const pair: Pair = Array.isArray(v) ? v : [v, v];
      const slot = m[1] as "from" | "to";
      if (dark) grad[slot] = [grad[slot]?.[0] ?? pair[1], pair[1]];
      else grad[slot] = [pair[0], grad[slot]?.[1] ?? pair[1]];
    } else if (c === "animate-number") s.animate = "number";
    else if ((m = /^h-(\d+(?:\.5)?)$/.exec(c))) s.height = Number(m[1]) * 4;
    else unknown.push(raw);
  }
  if (padded) s.pad = pad;
  if (grad.dir && grad.from && grad.to) s.gradient = { from: grad.from, to: grad.to, dir: grad.dir };
  for (const k of ["fg", "bg"] as const) {
    const [l, d] = slots[k];
    if (l || d) s[k] = [l ?? d!, d ?? l!];
  }
  return s;
}

export type Child = Node | false | null | undefined;
const kids = (c: Child[]) => c.filter(Boolean) as Node[];
const ACTION = /^(inc|dec|toggle):[A-Za-z_][\w.]*$/;

/** Primitives. Each takes classes first; none adds a look the classes do not ask for. */
export const Col = (cls: string, ...children: Child[]): Node => ({ t: "col", style: tw(cls), children: kids(children) });
export const Row = (cls: string, ...children: Child[]): Node => ({ t: "row", style: tw(cls), children: kids(children) });
export const Text = (cls: string, text: string): Node => ({ t: "text", style: tw(cls), text });
export const Spacer = (cls = ""): Node => ({ t: "spacer", style: tw(cls) });
/** Runs a vocabulary action in the widget process, without the app. */
export function Button(cls: string, action: string, label: string): Node {
  if (!ACTION.test(action)) throw new Error(`nsh-widgets: unknown button action "${action}" (use inc:, dec: or toggle:)`);
  return { t: "button", style: tw(cls), action, text: label };
}
/** A horizontal bar filled to data[key] (0 to 1). `text-*` colours the fill, `bg-*` the track, `h-*` its thickness. */
export const Progress = (cls: string, key: string): Node => ({ t: "progress", style: tw(cls), key });
/** A bar chart of the numbers in data[key], bottom-aligned, scaled to the largest. `h-*` is the chart height. */
export const Bars = (cls: string, key: string): Node => ({ t: "bars", style: tw(cls), key });
/** The shared icon set: SF Symbols on Apple, bundled vector drawables on Android. */
export const ICONS = [
  "star", "bolt", "check", "play", "pause", "flame", "plus", "minus", "arrow-up", "arrow-down", "heart",
  "home", "person", "bell", "clock", "calendar", "cloud", "sun", "moon", "drop", "music",
] as const;
export type IconName = (typeof ICONS)[number];
/** An icon from the shared set. `text-*` colours it, `h-*` sizes it. */
export const Icon = (cls: string, name: IconName): Node => ({ t: "icon", style: tw(cls), name });
/** Text the system ticks every second: time since data[key] ("up") or until it ("down"), an epoch in ms. */
export const Timer = (cls: string, key: string, mode: "up" | "down" = "up"): Node => ({ t: "timer", style: tw(cls), key, mode });
/** The current time, kept live by the system. */
export const Clock = (cls: string): Node => ({ t: "clock", style: tw(cls) });
/** A switch bound to the boolean data[key]; pressing it runs toggle:key in the widget. */
export const Toggle = (cls: string, key: string, label: string): Node => ({ t: "toggle", style: tw(cls), key, text: label, action: `toggle:${key}` });
/** Shows its children one at a time, changing every `interval` seconds where the platform can animate it. */
export const Rotate = (cls: string, interval: number, ...children: Child[]): Node => ({ t: "rotate", style: tw(cls), interval, children: kids(children) });

/** Opens the app at `url` (its URL scheme or a universal link). */
export const Link = (cls: string, url: string, label: string): Node => ({ t: "button", style: tw(cls), action: `open:${url}`, text: label });

/** Widget Tree IR v1, the content of snapshot.json. */
export type Snapshot = { v: 1; font?: string; url?: string; root: Node; accessory?: Node };
export type SnapshotOptions = {
  /** Opened when the widget itself is tapped (the app's URL scheme or a universal link). */
  url?: string;
  /** A smaller tree for the iOS lock screen (circular, rectangular, inline); `root` when absent. */
  accessory?: Node;
};

/** The snapshot of `root`, with the font set by `setFont`. */
export function snapshot(root: Node, options: SnapshotOptions = {}): Snapshot {
  const out: Snapshot = { v: 1, root };
  if (font) out.font = font;
  if (options.url) out.url = options.url;
  if (options.accessory) out.accessory = options.accessory;
  return out;
}
