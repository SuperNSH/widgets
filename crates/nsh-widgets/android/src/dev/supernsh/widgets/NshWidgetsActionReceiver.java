package dev.supernsh.widgets;

import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;
import org.json.JSONObject;

/** Not exported: runs a widget button's action (inc:key, dec:key, toggle:key) on data.json, in the app's process. */
public class NshWidgetsActionReceiver extends BroadcastReceiver {
    static final String ACTION = "dev.supernsh.widgets.ACTION";

    @Override
    public void onReceive(Context c, Intent intent) {
        if (!ACTION.equals(intent.getAction())) return;
        String kind = intent.getStringExtra("kind"), action = intent.getStringExtra("action");
        if (kind == null || action == null) return;
        String[] p = action.split(":", 2);
        if (p.length != 2) return;
        try {
            JSONObject d = NshWidgets.data(c, kind);
            switch (p[0]) {
                case "inc": d.put(p[1], d.optLong(p[1], 0) + 1); break;
                case "dec": d.put(p[1], d.optLong(p[1], 0) - 1); break;
                case "toggle": d.put(p[1], !d.optBoolean(p[1], false)); break;
                default: return;
            }
            NshWidgets.writeData(c, kind, d, action);
        } catch (Exception e) {
            android.util.Log.e("nsh-widgets", "action failed", e);
        }
        NshWidgets.update(c, kind);
    }
}
