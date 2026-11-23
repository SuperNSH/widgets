import { mount, Text, View } from "nsh";
import { snapshot } from "./src/base";
import { ActionButton, ButtonRow, Card, Header, Stat } from "./src/kit";

/** Shows the IR a kit widget compiles to; it never calls the native module. */
const tree = Card(Header("NSH WIDGETS"), Stat("{count}", "taps"), ButtonRow([ActionButton("+1", "inc:count")]));

function App() {
  return (
    <View class="flex-1 gap-3 p-4">
      <Text testID="title" class="text-foreground">nsh-widgets</Text>
      <Text testID="ir" class="text-foreground">{JSON.stringify(snapshot(tree))}</Text>
    </View>
  );
}

mount(App);
