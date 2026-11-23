import { expect, test } from "nsh/test";
import { Bars, Button, Col, Icon, Link, Progress, Rotate, Row, Spacer, Text, Timer, Toggle, setFont, setTheme, snapshot, tw, defaultTheme } from "../src/base";

test("resolves sizes, weights, spacing and radius", () => {
  const unknown: string[] = [];
  expect(tw("text-sm font-semibold p-4 px-2 gap-1.5 rounded-xl items-center grow h-16", defaultTheme, unknown)).toEqual({
    size: 14,
    weight: "semibold",
    pad: [16, 8, 16, 8],
    gap: 6,
    radius: 12,
    align: "center",
    grow: true,
    height: 64,
  });
  expect(unknown).toEqual([]);
});

test("palette colours and dark variants fill light and dark slots", () => {
  expect(tw("bg-white dark:bg-neutral-900 text-blue-600").bg).toEqual(["#ffffff", "#171717"]);
  expect(tw("text-blue-600").fg).toEqual(["#2563eb", "#2563eb"]);
});

test("theme tokens carry both of their values", () => {
  expect(tw("bg-surface text-muted", defaultTheme)).toEqual({ fg: ["#64748b", "#a3a3a3"], bg: ["#ffffff", "#171717"] });
  setTheme({ primary: ["#111111", "#eeeeee"] });
  expect(tw("text-primary").fg).toEqual(["#111111", "#eeeeee"]);
  setTheme({});
});

test("gradients need a direction and both ends", () => {
  expect(tw("bg-gradient-to-r from-blue-500 to-violet-600 dark:from-blue-700").gradient).toEqual({
    from: ["#3b82f6", "#1d4ed8"],
    to: ["#7c3aed", "#7c3aed"],
    dir: "r",
  });
  expect(tw("from-blue-500 to-violet-600").gradient).toBe(undefined);
});

test("size variants hide nodes per widget family", () => {
  expect(tw("small:hidden").hide).toEqual(["small"]);
  expect(tw("hidden").hide).toEqual(["small", "medium", "large"]);
});

test("unknown classes are reported, never dropped silently", () => {
  const unknown: string[] = [];
  tw("shadow-lg text-sm blur", defaultTheme, unknown);
  expect(unknown).toEqual(["shadow-lg", "blur"]);
});

test("primitives build IR nodes and drop false children", () => {
  const node = Col("gap-2", Text("text-xs", "{count}"), false, Row("", Spacer()), null);
  expect(node.t).toBe("col");
  expect(node.children?.map((c) => c.t)).toEqual(["text", "row"]);
  expect(Progress("h-2", "p")).toEqual({ t: "progress", style: { height: 8 }, key: "p" });
  expect(Bars("", "week").key).toBe("week");
  expect(Icon("", "bolt").name).toBe("bolt");
  expect(Timer("", "at", "down").mode).toBe("down");
  expect(Toggle("", "t1", "Ship").action).toBe("toggle:t1");
  expect(Rotate("", 5, Text("", "a"), Text("", "b")).children?.length).toBe(2);
  expect(Link("", "app://x", "Open").action).toBe("open:app://x");
});

test("buttons take only the action vocabulary", () => {
  expect(Button("", "inc:count", "+1").action).toBe("inc:count");
  expect(() => Button("", "delete:everything", "x")).toThrow();
  expect(() => Button("", "inc:", "x")).toThrow();
});

test("snapshots carry the IR version, the font and the widget url", () => {
  setFont(undefined);
  expect(snapshot(Text("", "hi"))).toEqual({ v: 1, root: { t: "text", style: {}, text: "hi" } });
  setFont("Onest");
  expect(snapshot(Text("", "hi"), { url: "app://home" })).toEqual({ v: 1, font: "Onest", url: "app://home", root: { t: "text", style: {}, text: "hi" } });
  setFont(undefined);
});
