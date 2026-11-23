import { effect, lifecycle, mount, signal, Text, UrlScheme, View } from "nsh";
import { platform } from "os";
import { Widgets } from "nsh-widgets";
import { counterWidget } from "./widgets/counter";
import { dashboardWidget, iconsWidget, liveWidget, stepsAccessory, stepsWidget, tasksWidget } from "./widgets/demos";

const NAMES: Record<string, string> = { macos: "macOS", ios: "iOS", android: "Android" };
const here = NAMES[platform()] ?? platform();

function randomDashboard() {
  const steps = 2000 + Math.round(Math.random() * 9000);
  const goal = 10000;
  const old = Widgets.data("Dashboard");
  Widgets.update("Dashboard", {
    steps,
    goal,
    percent: Math.round((steps / goal) * 100),
    progress: Math.min(steps / goal, 1),
    week: Array.from({ length: 7 }, () => 2 + Math.round(Math.random() * 10)),
    streak: typeof old.streak === "number" ? old.streak : 12,
    kcal: Math.round(steps * 0.045),
  });
  Widgets.update("Steps", { steps, goal, percent: Math.round((steps / goal) * 100), progress: Math.min(steps / goal, 1) });
}

function restartFocus() {
  const now = Date.now();
  Widgets.update("Live", { focusStart: now, breakAt: now + 25 * 60 * 1000 });
}

/** Publishes every widget's tree, and seeds data the first time. */
function publish() {
  Widgets.setUi("Counter", counterWidget(here), { url: "nshwidgets://widget/Counter" });
  Widgets.setUi("Dashboard", dashboardWidget());
  Widgets.setUi("Live", liveWidget());
  Widgets.setUi("Tasks", tasksWidget());
  Widgets.setUi("Steps", stepsWidget(), { accessory: stepsAccessory() });
  Widgets.setUi("Icons", iconsWidget());
  if (Widgets.data("Counter").count === undefined) Widgets.update("Counter", { count: 0, label: "tap +1 in the widget" });
  if (Widgets.data("Dashboard").steps === undefined || Widgets.data("Steps").steps === undefined) randomDashboard();
  if (Widgets.data("Live").focusStart === undefined) restartFocus();
  if (Widgets.data("Tasks").t1 === undefined) Widgets.update("Tasks", { t1: true, t2: false, t3: false });
}

function App() {
  const count = signal(0);
  const status = signal("starting");
  const tasks = signal("tasks: not read yet");
  const presses = signal("no widget presses yet");
  const link = signal("no link yet");
  let total = 0;

  const showTasks = (d: Record<string, unknown>) =>
    tasks.set(`tasks: ${["t1", "t2", "t3"].map((k) => (d[k] ? "☑" : "☐")).join(" ")}`);
  const refresh = () => {
    const c = Widgets.data("Counter").count;
    if (typeof c === "number") count.set(c);
    showTasks(Widgets.data("Tasks"));
    const taken = Widgets.takeActions();
    if (taken.length > 0) {
      total += taken.length;
      presses.set(`${total} widget presses, last: ${taken.map((a) => `${a.kind} ${a.action}`).join(", ")}`);
    }
  };
  const bump = (by: number) => {
    const c = Widgets.data("Counter").count;
    const n = (typeof c === "number" ? c : 0) + by;
    Widgets.update("Counter", { count: n, label: `set in the ${here} app` });
    count.set(n);
  };
  const guarded = (what: string, f: () => void) => () => {
    try {
      f();
      status.set(what);
    } catch (e) {
      status.set(`error: ${String(e)}`);
    }
  };

  guarded("widgets published", publish)();
  UrlScheme.on("open", ({ url }) => link.set(`opened by link: ${url}`));
  Widgets.onChange("Counter", (d) => {
    if (typeof d.count === "number") count.set(d.count);
    refresh();
  });
  Widgets.onChange("Tasks", (d) => {
    showTasks(d);
    refresh();
  });
  effect(() => {
    if (lifecycle() === "active") refresh();
  });

  const button = (label: string, color: string, onPress: () => void) => (
    <View onPress={onPress} role="button" class={`rounded-lg ${color} px-5 py-3`}>
      <Text class="text-white">{label}</Text>
    </View>
  );

  return (
    <View class="gap-4 p-6 pt-24 bg-neutral-950 flex-1">
      <Text class="text-xl font-semibold text-white">nsh-widgets gallery</Text>
      <Text class="text-5xl font-bold text-white">Count: {count()}</Text>
      <View class="flex-row gap-3">
        {button("+1 from app", "bg-blue-600", guarded("count changed", () => bump(1)))}
        {button("−1 from app", "bg-neutral-700", guarded("count changed", () => bump(-1)))}
      </View>
      <View class="flex-row gap-3">
        {button("New dashboard data", "bg-violet-700", guarded("dashboard updated", randomDashboard))}
        {button("Restart focus timer", "bg-amber-700", guarded("focus timer restarted", restartFocus))}
      </View>
      {button("Publish widget UI", "bg-green-700", guarded("widgets published", publish))}
      <Text class="text-base text-neutral-100">{tasks()}</Text>
      <Text class="text-base text-neutral-100">{status()}</Text>
      <Text class="text-base font-semibold text-emerald-300">{presses()}</Text>
      <Text class="text-base font-semibold text-emerald-300">{link()}</Text>
    </View>
  );
}

mount(App);
