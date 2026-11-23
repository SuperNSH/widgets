// nsh-widgets for TSX apps: describe a widget once with base primitives or kit components, publish it with
// Widgets.setUi, and feed it data. The renderers in the widget extension (iOS, macOS) and the Android
// provider draw it natively; widget buttons change the data without the app.
import { NativeWidgets } from "nsh-widgets/native";
import { snapshot, type Node, type SnapshotOptions } from "./base";

export * from "./base";

/**
 * A failed widget call. `code` is stable: invalid_kind, invalid_action, invalid_json, no_app_group,
 * no_group_configured, io, unsupported, activities_disabled, no_activity, activity_refused.
 */
export class WidgetError extends Error {
  constructor(
    readonly code: string,
    message: string,
  ) {
    super(message);
    this.name = "WidgetError";
  }
}

function call<T>(f: () => string): T {
  try {
    return JSON.parse(f()) as T;
  } catch (e) {
    throw asError(e);
  }
}

function asError(e: unknown): WidgetError {
  if (e instanceof WidgetError) return e;
  const text = e instanceof Error ? e.message : String(e);
  const m = /^([a-z_]+): ([\s\S]*)$/.exec(text);
  return m ? new WidgetError(m[1], m[2]) : new WidgetError("io", text);
}

export type WidgetData = Record<string, unknown>;
/** A widget button press the app has not seen yet. */
export type WidgetAction = { id: string; kind: string; action: string; at: number };

export const Widgets = {
  /** Uses `group` as the App Group on iOS and macOS instead of the one `nsh-widgets build` wrote. */
  configure(group: string): void {
    NativeWidgets.configure(group);
  },
  /** Publishes `kind`'s UI tree and redraws it. `url` opens when the widget is tapped; `accessory` is the lock-screen tree. */
  setUi(kind: string, root: Node, options: SnapshotOptions = {}): void {
    call(() => NativeWidgets.setUi(kind, JSON.stringify(snapshot(root, options))));
  },
  /** Replaces `kind`'s data and redraws it. */
  setData(kind: string, data: WidgetData): void {
    call(() => NativeWidgets.setData(kind, JSON.stringify(data)));
  },
  /** Merges `patch` into `kind`'s data, keeping what widget buttons changed, and returns the result. */
  update(kind: string, patch: WidgetData): WidgetData {
    return call<WidgetData>(() => NativeWidgets.updateData(kind, JSON.stringify(patch)));
  },
  /** `kind`'s data as the widget last left it. */
  data(kind: string): WidgetData {
    return call<WidgetData>(() => NativeWidgets.data(kind));
  },
  /** The button presses since the last call, oldest first. Each is returned once. */
  takeActions(): WidgetAction[] {
    return call<WidgetAction[]>(() => NativeWidgets.takeActions());
  },
  /**
   * Starts `kind` as an iOS Live Activity (it must be listed under "activities" in widgets.json): the
   * lock screen and the expanded Dynamic Island draw the kind's tree, the compact one its `accessory`
   * tree, with `data` over the kind's stored data. Publish the tree with `setUi` first. A running
   * activity of the same kind is replaced. Throws `unsupported` off iOS.
   */
  startActivity(kind: string, data: WidgetData = {}): void {
    call(() => NativeWidgets.startActivity(kind, JSON.stringify(data)));
  },
  /** Replaces the running `kind` activity's data. */
  updateActivity(kind: string, data: WidgetData): void {
    call(() => NativeWidgets.updateActivity(kind, JSON.stringify(data)));
  },
  /** Ends the running `kind` activity, if any. */
  endActivity(kind: string): void {
    call(() => NativeWidgets.endActivity(kind));
  },
  /** Redraws `kind`'s placed widgets. */
  reload(kind: string): void {
    call(() => NativeWidgets.reload(kind));
  },
  /** Calls `listener` with `kind`'s data each time a widget button changes it. Returns a stop function. Where there is no store, it never calls. */
  onChange(kind: string, listener: (data: WidgetData) => void): () => void {
    let stopped = false;
    (async () => {
      while (!stopped) {
        let data: WidgetData;
        try {
          data = JSON.parse(await NativeWidgets.nextChange(kind)) as WidgetData;
        } catch (e) {
          // No store here (web, or a desktop app that never called configure): stop rather than spin.
          const code = asError(e).code;
          if (code === "unsupported" || code === "no_group_configured") return;
          throw asError(e);
        }
        if (!stopped) listener(data);
      }
    })();
    return () => {
      stopped = true;
    };
  },
};
