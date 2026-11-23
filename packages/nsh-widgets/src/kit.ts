// nsh-widgets/kit: styled widget components, built only from nsh-widgets/base primitives and the theme's
// semantic tokens (as GPUI Kit is built on gpui-base). Nothing here reaches a renderer directly: every
// component returns base nodes, so the kit can grow and restyle without changing the SwiftUI or
// RemoteViews renderers. Apps that want their own look skip the kit and compose base primitives.
import { Button, Col, Link, Row, Spacer, Text, type Child, type Node } from "./base";

export type Variant = "primary" | "secondary" | "accent" | "ghost";
export type Size = "sm" | "md";

const variants: Record<Variant, string> = {
  primary: "bg-primary text-on-primary",
  secondary: "bg-surface-2 text-text",
  accent: "bg-accent text-on-accent",
  ghost: "text-primary",
};
const sizes: Record<Size, string> = {
  sm: "rounded-full px-3 py-1 text-xs font-semibold",
  md: "rounded-full px-4 py-1.5 text-sm font-semibold",
};

/** The widget's frame: theme surface, standard padding, fills the widget. */
export const Card = (...children: Child[]): Node => Col("bg-surface p-4 gap-1 grow", ...children);

/** A small caps-style title, with an optional trailing note that drops out of small widgets. */
export const Header = (title: string, trailing?: string): Node =>
  Row("gap-2 items-center", Text("text-xs font-semibold text-muted", title), Spacer(), trailing ? Text("text-xs text-muted small:hidden", trailing) : null);

/** A headline number or value with a caption under it. Both may hold {key} templates. */
export const Stat = (value: string, caption?: string): Node =>
  Col("gap-1", Text("text-5xl font-bold text-primary", value), caption ? Text("text-sm text-text", caption) : null);

/** A pressable that runs a base action (inc:, dec:, toggle:) in the widget process. */
export const ActionButton = (label: string, action: string, o: { variant?: Variant; size?: Size; hide?: string } = {}): Node =>
  Button(`${variants[o.variant ?? "primary"]} ${sizes[o.size ?? "md"]} ${o.hide ?? ""}`, action, label);

/** A pressable that opens the app at a URL. */
export const LinkButton = (label: string, url: string, o: { variant?: Variant; size?: Size; hide?: string } = {}): Node =>
  Link(`${variants[o.variant ?? "accent"]} ${sizes[o.size ?? "md"]} ${o.hide ?? ""}`, url, label);

/** Buttons in a row; `trailing` sits at the far end. */
export const ButtonRow = (buttons: Child[], trailing?: Child): Node => Row("gap-2 items-center", ...buttons, trailing ? Spacer() : null, trailing);

/** Flexible space between kit pieces. */
export const Gap = () => Spacer();

/** The kit's font, when the app bundles it (widgets.json "fonts"). Android widgets use the system font. */
export const kitFont = "Onest";

/** The kit's theme tokens on top of base's: accent for links and secondary emphasis. */
export const kitTheme = {
  accent: ["#fef3c7", "#78350f"] as [string, string],
  "on-accent": ["#92400e", "#fde68a"] as [string, string],
};
