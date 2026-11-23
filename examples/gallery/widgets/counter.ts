// The counter, written with kit components. No platform code here.
import { setTheme } from "nsh-widgets/base";
import { ActionButton, ButtonRow, Card, Gap, Header, LinkButton, Stat, kitTheme } from "nsh-widgets/kit";

export function counterWidget(platform: string) {
  setTheme(kitTheme);
  return Card(
    Header("NSH WIDGETS", platform),
    Gap(),
    Stat("{count}", "{label}"),
    Gap(),
    ButtonRow(
      [ActionButton("+1", "inc:count"), ActionButton("−1", "dec:count", { variant: "secondary" })],
      LinkButton("Open", "nshwidgets://widget/Counter", { hide: "small:hidden" }),
    ),
  );
}
