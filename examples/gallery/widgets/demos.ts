// Capability demos: three widgets written once, drawn natively on iOS and Android. No platform code here.
import { Bars, Clock, Col, ICONS, Icon, type IconName, Progress, Rotate, Row, Spacer, Text, Timer, Toggle } from "nsh-widgets/base";
import { ActionButton, ButtonRow, Card, Header } from "nsh-widgets/kit";

/** Progress, a bar chart, icons, a gradient banner, an animated number and a widget-side button. */
export const dashboardWidget = () =>
  Col(
    "bg-surface p-4 gap-2 grow",
    Row(
      "gap-2 items-center rounded-xl px-3 py-2 bg-gradient-to-r from-blue-500 to-violet-600 dark:from-blue-700 dark:to-violet-800",
      Icon("text-white h-4", "bolt"),
      Text("text-sm font-semibold text-white", "Today"),
      Spacer(),
      Text("text-sm font-bold text-white animate-number", "{steps} steps"),
    ),
    Row("gap-2 items-center", Text("text-xs text-muted", "Goal {goal}"), Spacer(), Text("text-xs font-semibold text-primary", "{percent}%")),
    Progress("text-primary bg-surface-2 h-2", "progress"),
    Bars("text-primary h-16 gap-1 small:hidden", "week"),
    Row(
      "gap-2 items-center",
      Icon("text-amber-500 h-4", "star"),
      Text("text-xs text-text animate-number", "{streak} day streak"),
      Spacer(),
      Icon("text-rose-500 h-4", "flame"),
      Text("text-xs text-text", "{kcal} kcal"),
    ),
    ButtonRow([ActionButton("+1 day", "inc:streak", { size: "sm" }), ActionButton("−1", "dec:streak", { size: "sm", variant: "secondary" })]),
  );

/** Text the system keeps moving with no app and no timeline work: a stopwatch, a countdown, a clock, rotating tips. */
export const liveWidget = () =>
  Col(
    "bg-gradient-to-b from-slate-800 to-slate-950 p-4 gap-1 grow",
    Row("gap-2 items-center", Icon("text-amber-400 h-4", "play"), Text("text-xs font-semibold text-slate-300", "FOCUS SESSION"), Spacer(), Clock("text-xs text-slate-400")),
    Timer("text-4xl font-bold text-white", "focusStart", "up"),
    Row("gap-1 items-center", Text("text-xs text-slate-400", "Break in"), Timer("text-xs font-semibold text-amber-300", "breakAt", "down")),
    Spacer(),
    Rotate(
      "small:hidden",
      3,
      Text("text-sm text-slate-200", "Tip: phone face down"),
      Text("text-sm text-slate-200", "Tip: one tab at a time"),
      Text("text-sm text-slate-200", "Tip: water, then work"),
    ),
  );

/** Switches bound to data.json: each press flips a value in the widget process, and the app sees it live. */
export const tasksWidget = () =>
  Card(
    Header("TODAY'S TASKS", "tap to toggle"),
    Toggle("text-sm text-text text-primary", "t1", "Ship the widget spike"),
    Toggle("text-sm text-text text-primary", "t2", "Review the NSH PR"),
    Toggle("text-sm text-text text-primary", "t3", "Update the plan"),
    Spacer(),
  );

/** A gradient card on the home screen, and a smaller tree for the iOS lock screen. */
export const stepsWidget = () =>
  Col(
    "bg-gradient-to-br from-emerald-500 to-blue-600 dark:from-emerald-700 dark:to-blue-800 p-4 gap-1 grow",
    Row("gap-1 items-center", Icon("text-white h-4", "bolt"), Text("text-xs font-semibold text-white", "STEPS")),
    Spacer(),
    Text("text-3xl font-bold text-white animate-number", "{steps}"),
    Progress("text-white bg-emerald-900 h-1.5", "progress"),
    Text("text-xs text-emerald-50", "{percent}% of {goal}"),
  );

export const stepsAccessory = () =>
  Row("gap-1 items-center", Icon("h-3", "bolt"), Text("text-sm font-semibold", "{steps}"), Text("text-xs circular:hidden", "steps"));

/** Every icon of the shared set, on a gradient. */
export const iconsWidget = () => {
  const row = (names: IconName[]) => Row("gap-3 items-center", ...names.map((n) => Icon("text-white h-5", n)));
  return Col(
    "bg-gradient-to-r from-violet-600 to-rose-500 dark:from-violet-800 dark:to-rose-700 p-4 gap-3 grow",
    Text("text-xs font-semibold text-white", "SHARED ICONS"),
    row(ICONS.slice(0, 7) as IconName[]),
    row(ICONS.slice(7, 14) as IconName[]),
    row(ICONS.slice(14) as IconName[]),
  );
};
