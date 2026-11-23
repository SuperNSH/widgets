package dev.supernsh.widgets;

import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;

/** Not exported: runs a widget button's action (inc:key, dec:key, toggle:key) on data.json, in the app's process. */
public class NshWidgetsActionReceiver extends BroadcastReceiver {
    static final String ACTION = "dev.supernsh.widgets.ACTION";

    @Override
    public void onReceive(Context c, Intent intent) {
        if (!ACTION.equals(intent.getAction())) return;
        String kind = intent.getStringExtra("kind"), action = intent.getStringExtra("action");
        if (kind == null || action == null) return;
        NshWidgets.apply(c, kind, action);
        NshWidgets.update(c, kind);
    }
}
