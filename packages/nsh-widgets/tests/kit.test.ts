import { expect, test } from "nsh/test";
import { setTheme, type Node } from "../src/base";
import { ActionButton, ButtonRow, Card, Header, LinkButton, Stat, kitTheme } from "../src/kit";

const types = (n: Node): string[] => [n.t, ...(n.children ?? []).flatMap(types)];

test("kit components compile to base nodes only", () => {
  setTheme(kitTheme);
  const card = Card(Header("TITLE", "note"), Stat("{count}", "{label}"), ButtonRow([ActionButton("+1", "inc:count")], LinkButton("Open", "app://x")));
  const known = new Set(["col", "row", "text", "button", "spacer"]);
  expect(types(card).every((t) => known.has(t))).toBe(true);
  setTheme({});
});

test("variants resolve through the theme", () => {
  setTheme(kitTheme);
  expect(ActionButton("+1", "inc:count").style?.bg).toEqual(["#2563eb", "#60a5fa"]);
  expect(ActionButton("-1", "dec:count", { variant: "secondary" }).style?.bg).toEqual(["#f1f5f9", "#262626"]);
  expect(LinkButton("Open", "app://x").style?.bg).toEqual(kitTheme.accent);
  setTheme({});
});

test("the header's note drops out of small widgets", () => {
  const header = Header("TITLE", "note");
  expect(header.children?.[2]?.style?.hide).toEqual(["small"]);
  expect(Header("TITLE").children?.length).toBe(2);
});
