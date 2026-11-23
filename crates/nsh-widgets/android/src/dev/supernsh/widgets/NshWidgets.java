package dev.supernsh.widgets;

import android.app.PendingIntent;
import android.appwidget.AppWidgetManager;
import android.content.ComponentName;
import android.content.Context;
import android.content.Intent;
import android.content.res.Configuration;
import android.graphics.Color;
import android.net.Uri;
import android.os.Build;
import android.util.TypedValue;
import android.widget.RemoteViews;
import java.io.File;
import java.io.FileOutputStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.util.Iterator;
import org.json.JSONArray;
import org.json.JSONObject;

/**
 * The nsh-widgets renderer for Android: builds RemoteViews from whatever Widget Tree IR v1 tree the app
 * wrote to snapshot.json, fills {key} templates from data.json, and redraws placed widgets. Each kind is
 * an {@link NshWidgetProvider} slot that the app's generated manifest names, with its kind as meta-data.
 */
public final class NshWidgets {
    private NshWidgets() {}

    /** The receiver meta-data that names a slot's kind. */
    static final String KIND_META = "dev.supernsh.widgets.kind";

    static File dir(Context c, String kind) { return new File(c.getFilesDir(), "nsh/widgets/" + kind); }

    static JSONObject readJson(File f) {
        try {
            return new JSONObject(new String(Files.readAllBytes(f.toPath()), StandardCharsets.UTF_8));
        } catch (Exception e) {
            return null;
        }
    }

    static JSONObject data(Context c, String kind) {
        JSONObject env = readJson(new File(dir(c, kind), "data.json"));
        JSONObject d = env == null ? null : env.optJSONObject("data");
        return d == null ? new JSONObject() : d;
    }

    static void writeData(Context c, String kind, JSONObject body, String action) throws Exception {
        File d = dir(c, kind);
        d.mkdirs();
        File f = new File(d, "data.json");
        File tmp = new File(d, "data.json.tmp");
        try (FileOutputStream o = new FileOutputStream(tmp)) {
            o.write(new JSONObject().put("schema", 1).put("data", body).toString().getBytes(StandardCharsets.UTF_8));
        }
        if (!tmp.renameTo(f)) throw new Exception("rename failed");
        try (FileOutputStream o = new FileOutputStream(new File(d.getParentFile(), "actions.jsonl"), true)) {
            o.write(new JSONObject().put("id", java.util.UUID.randomUUID().toString()).put("kind", kind).put("action", action).put("at", System.currentTimeMillis()).toString()
                .concat("\n").getBytes(StandardCharsets.UTF_8));
        }
    }

    static int res(Context c, String type, String name) {
        return c.getResources().getIdentifier(name, type, c.getPackageName());
    }

    static int px(Context c, double dp) {
        return (int) Math.round(dp * c.getResources().getDisplayMetrics().density);
    }

    static int color(String hex) {
        return Color.parseColor(hex.length() == 7 ? hex : "#000000");
    }

    static String fill(String text, JSONObject data) {
        String out = text == null ? "" : text;
        for (Iterator<String> it = data.keys(); it.hasNext(); ) {
            String k = it.next();
            Object v = data.opt(k);
            String s = v instanceof Number && ((Number) v).doubleValue() == Math.rint(((Number) v).doubleValue())
                ? String.valueOf(((Number) v).longValue()) : String.valueOf(v);
            out = out.replace("{" + k + "}", s);
        }
        return out;
    }

    /** Sets a colour that follows light and dark: both on Android 12+, the current one before. */
    static void pairColor(Context c, RemoteViews v, int id, String method, JSONArray pair) {
        if (pair == null || pair.length() != 2) return;
        int light = color(pair.optString(0)), dark = color(pair.optString(1));
        if (Build.VERSION.SDK_INT >= 31) {
            v.setColorInt(id, method, light, dark);
        } else {
            boolean night = (c.getResources().getConfiguration().uiMode & Configuration.UI_MODE_NIGHT_MASK) == Configuration.UI_MODE_NIGHT_YES;
            v.setInt(id, method, night ? dark : light);
        }
    }

    static void box(Context c, RemoteViews v, int id, JSONObject style) {
        if (style == null) return;
        JSONArray pad = style.optJSONArray("pad");
        if (pad != null && pad.length() == 4) {
            v.setViewPadding(id, px(c, pad.optDouble(3)), px(c, pad.optDouble(0)), px(c, pad.optDouble(1)), px(c, pad.optDouble(2)));
        }
        JSONArray bg = style.optJSONArray("bg");
        JSONObject grad = style.optJSONObject("gradient");
        // RemoteViews cannot draw a gradient from code: use the gradient's first colour.
        if (bg == null && grad != null) bg = grad.optJSONArray("from");
        pairColor(c, v, id, "setBackgroundColor", bg);
        double r = style.optDouble("radius", 0);
        if (r > 0 && Build.VERSION.SDK_INT >= 31) {
            v.setViewOutlinePreferredRadius(id, (float) Math.min(r, 200), TypedValue.COMPLEX_UNIT_DIP);
        }
    }

    static RemoteViews gap(Context c, double size) {
        RemoteViews g = new RemoteViews(c.getPackageName(), res(c, "layout", "nsh_widgets_gap"));
        int p = px(c, size);
        g.setViewPadding(res(c, "id", "nsh_widgets_gap"), p, p, 0, 0);
        return g;
    }

    static String weightOf(JSONObject style) {
        String w = style == null ? "" : style.optString("weight", "regular");
        return w.equals("medium") || w.equals("semibold") || w.equals("bold") ? w : "regular";
    }

    static void textStyle(Context c, RemoteViews v, int id, JSONObject style) {
        if (style == null) return;
        if (style.has("size")) v.setTextViewTextSize(id, TypedValue.COMPLEX_UNIT_SP, (float) style.optDouble("size"));
        pairColor(c, v, id, "setTextColor", style.optJSONArray("fg"));
    }

    static PendingIntent action(Context c, String kind, String action, int[] req) {
        Intent run = new Intent(c, NshWidgetsActionReceiver.class).setAction(NshWidgetsActionReceiver.ACTION)
            .setData(Uri.parse("nshui://" + kind + "/" + Uri.encode(action)))
            .putExtra("kind", kind).putExtra("action", action);
        return PendingIntent.getBroadcast(c, req[0]++, run, PendingIntent.FLAG_IMMUTABLE | PendingIntent.FLAG_UPDATE_CURRENT);
    }

    static final String[] ICONS = {"star", "bolt", "check", "play", "flame"};

    static RemoteViews build(Context c, JSONObject node, JSONObject data, String kind, boolean inRow, int[] req, String family) {
        String t = node.optString("t");
        JSONObject style = node.optJSONObject("style");
        JSONArray hide = style == null ? null : style.optJSONArray("hide");
        if (hide != null) for (int i = 0; i < hide.length(); i++) if (family.equals(hide.optString(i))) return gap(c, 0);
        String pkg = c.getPackageName();
        switch (t) {
            case "col":
            case "row": {
                boolean row = t.equals("row");
                String align = style == null ? "" : style.optString("align");
                String layout = row ? (align.equals("start") ? "nsh_widgets_row_top" : align.equals("end") ? "nsh_widgets_row_bottom" : "nsh_widgets_row")
                    : (style != null && style.optBoolean("grow") ? "nsh_widgets_col_fill" : "nsh_widgets_col");
                RemoteViews v = new RemoteViews(pkg, res(c, "layout", layout));
                int id = res(c, "id", "nsh_widgets_box");
                box(c, v, id, style);
                JSONArray kids = node.optJSONArray("children");
                double g = style == null ? 0 : style.optDouble("gap", 0);
                if (kids != null) {
                    for (int i = 0; i < kids.length(); i++) {
                        if (i > 0 && g > 0) v.addView(id, gap(c, g));
                        v.addView(id, build(c, kids.optJSONObject(i), data, kind, row, req, family));
                    }
                }
                return v;
            }
            case "spacer":
                return new RemoteViews(pkg, res(c, "layout", inRow ? "nsh_widgets_hspacer" : "nsh_widgets_vspacer"));
            case "text":
            case "button": {
                // One layout per weight: RemoteViews cannot set a typeface, but a layout can name the bundled font and weight.
                String w = style == null ? "" : style.optString("weight", "regular");
                if (!w.equals("medium") && !w.equals("semibold") && !w.equals("bold")) w = "regular";
                String layout = (t.equals("button") ? "nsh_widgets_button_" : "nsh_widgets_text_") + w;
                RemoteViews v = new RemoteViews(pkg, res(c, "layout", layout));
                int id = res(c, "id", "nsh_widgets_text");
                v.setTextViewText(id, fill(node.optString("text"), data));
                if (style != null) {
                    if (style.has("size")) v.setTextViewTextSize(id, TypedValue.COMPLEX_UNIT_SP, (float) style.optDouble("size"));
                    pairColor(c, v, id, "setTextColor", style.optJSONArray("fg"));
                }
                box(c, v, id, style);
                if (t.equals("button")) {
                    String action = node.optString("action");
                    PendingIntent pi;
                    if (action.startsWith("open:")) {
                        Intent open = new Intent(Intent.ACTION_VIEW, Uri.parse(action.substring(5))).setPackage(pkg);
                        pi = PendingIntent.getActivity(c, req[0]++, open, PendingIntent.FLAG_IMMUTABLE | PendingIntent.FLAG_UPDATE_CURRENT);
                    } else {
                        Intent run = new Intent(c, NshWidgetsActionReceiver.class).setAction(NshWidgetsActionReceiver.ACTION)
                            .setData(Uri.parse("nshui://" + kind + "/" + Uri.encode(action)))
                            .putExtra("kind", kind).putExtra("action", action);
                        pi = PendingIntent.getBroadcast(c, req[0]++, run, PendingIntent.FLAG_IMMUTABLE | PendingIntent.FLAG_UPDATE_CURRENT);
                    }
                    v.setOnClickPendingIntent(id, pi);
                }
                return v;
            }
            case "progress": {
                RemoteViews v = new RemoteViews(pkg, res(c, "layout", "nsh_widgets_progress"));
                int id = res(c, "id", "nsh_widgets_progress");
                double value = Math.max(0, Math.min(1, data.optDouble(node.optString("key"), 0)));
                v.setProgressBar(id, 1000, (int) Math.round(value * 1000), false);
                if (style != null && Build.VERSION.SDK_INT >= 31) {
                    JSONArray fg = style.optJSONArray("fg"), bg = style.optJSONArray("bg");
                    if (fg != null) v.setColorStateList(id, "setProgressTintList", android.content.res.ColorStateList.valueOf(color(fg.optString(0))), android.content.res.ColorStateList.valueOf(color(fg.optString(1))));
                    if (bg != null) v.setColorStateList(id, "setProgressBackgroundTintList", android.content.res.ColorStateList.valueOf(color(bg.optString(0))), android.content.res.ColorStateList.valueOf(color(bg.optString(1))));
                    if (style.has("height")) v.setViewLayoutHeight(id, (float) style.optDouble("height"), TypedValue.COMPLEX_UNIT_DIP);
                }
                return v;
            }
            case "bars": {
                // Each bar is a FrameLayout whose padding sets its size, which works on every Android version.
                RemoteViews row = new RemoteViews(pkg, res(c, "layout", "nsh_widgets_row_bottom"));
                int box = res(c, "id", "nsh_widgets_box");
                JSONArray values = data.optJSONArray(node.optString("key"));
                double h = style == null ? 48 : style.optDouble("height", 48), g = style == null ? 4 : style.optDouble("gap", 4);
                double top = 1;
                if (values != null) for (int i = 0; i < values.length(); i++) top = Math.max(top, values.optDouble(i, 0));
                int n = values == null ? 0 : values.length();
                for (int i = 0; i < n; i++) {
                    if (i > 0) row.addView(box, gap(c, g));
                    RemoteViews bar = new RemoteViews(pkg, res(c, "layout", "nsh_widgets_bar"));
                    int bid = res(c, "id", "nsh_widgets_bar");
                    bar.setViewPadding(bid, 0, px(c, Math.max(2, h * values.optDouble(i, 0) / top)), 0, 0);
                    pairColor(c, bar, bid, "setBackgroundColor", style == null ? null : style.optJSONArray("fg"));
                    if (Build.VERSION.SDK_INT >= 31) bar.setViewOutlinePreferredRadius(bid, 3, TypedValue.COMPLEX_UNIT_DIP);
                    row.addView(box, bar);
                }
                // Hold the chart's full height even when every value is small.
                RemoteViews frame = new RemoteViews(pkg, res(c, "layout", "nsh_widgets_gap"));
                frame.setViewPadding(res(c, "id", "nsh_widgets_gap"), 0, px(c, h), 0, 0);
                row.addView(box, frame);
                return row;
            }
            case "icon": {
                RemoteViews v = new RemoteViews(pkg, res(c, "layout", "nsh_widgets_icon"));
                int id = res(c, "id", "nsh_widgets_icon");
                String name = node.optString("name");
                int drawable = res(c, "drawable", "nsh_widgets_icon_" + (java.util.Arrays.asList(ICONS).contains(name) ? name : "star"));
                v.setImageViewResource(id, drawable);
                if (style != null) {
                    pairColor(c, v, id, "setColorFilter", style.optJSONArray("fg"));
                    double size = style.optDouble("height", style.optDouble("size", 18));
                    if (Build.VERSION.SDK_INT >= 31) {
                        v.setViewLayoutWidth(id, (float) size, TypedValue.COMPLEX_UNIT_DIP);
                        v.setViewLayoutHeight(id, (float) size, TypedValue.COMPLEX_UNIT_DIP);
                    }
                }
                return v;
            }
            case "timer": {
                RemoteViews v = new RemoteViews(pkg, res(c, "layout", "nsh_widgets_timer_" + weightOf(style)));
                int id = res(c, "id", "nsh_widgets_timer");
                long at = data.optLong(node.optString("key"), System.currentTimeMillis());
                long now = System.currentTimeMillis(), boot = android.os.SystemClock.elapsedRealtime();
                boolean down = "down".equals(node.optString("mode"));
                v.setChronometer(id, boot + (at - now), null, true);
                v.setChronometerCountDown(id, down);
                textStyle(c, v, id, style);
                return v;
            }
            case "clock": {
                RemoteViews v = new RemoteViews(pkg, res(c, "layout", "nsh_widgets_clock_" + weightOf(style)));
                textStyle(c, v, res(c, "id", "nsh_widgets_clock"), style);
                return v;
            }
            case "toggle": {
                boolean on = data.optBoolean(node.optString("key"), false);
                String label = fill(node.optString("text"), data);
                if (Build.VERSION.SDK_INT >= 31) {
                    // Android 12+: a real switch the launcher animates; the press runs toggle:key in our receiver.
                    RemoteViews v = new RemoteViews(pkg, res(c, "layout", "nsh_widgets_toggle"));
                    int id = res(c, "id", "nsh_widgets_toggle");
                    v.setTextViewText(id, label);
                    v.setCompoundButtonChecked(id, on);
                    v.setOnCheckedChangeResponse(id, RemoteViews.RemoteResponse.fromPendingIntent(action(c, kind, node.optString("action"), req)));
                    textStyle(c, v, id, style);
                    return v;
                }
                // Older Android: a pressable line with a check mark.
                RemoteViews v = new RemoteViews(pkg, res(c, "layout", "nsh_widgets_text_regular"));
                int id = res(c, "id", "nsh_widgets_text");
                v.setTextViewText(id, (on ? "\u2611 " : "\u2610 ") + label);
                textStyle(c, v, id, style);
                v.setOnClickPendingIntent(id, action(c, kind, node.optString("action"), req));
                return v;
            }
            case "rotate": {
                // ViewFlipper is one of the few views RemoteViews may animate: it slides between children on its own.
                RemoteViews v = new RemoteViews(pkg, res(c, "layout", "nsh_widgets_rotate"));
                int id = res(c, "id", "nsh_widgets_rotate");
                v.setInt(id, "setFlipInterval", (int) (node.optDouble("interval", 3) * 1000));
                JSONArray kids = node.optJSONArray("children");
                if (kids != null) for (int i = 0; i < kids.length(); i++) v.addView(id, build(c, kids.optJSONObject(i), data, kind, false, req, family));
                return v;
            }
            default:
                return new RemoteViews(pkg, res(c, "layout", "nsh_widgets_gap"));
        }
    }

    static RemoteViews views(Context c, String kind, String family) {
        String pkg = c.getPackageName();
        RemoteViews root = new RemoteViews(pkg, res(c, "layout", "nsh_widgets_root"));
        int rootId = android.R.id.background;
        // Updates reapply onto the existing views: clear the root so children are not reused or stacked.
        root.removeAllViews(rootId);
        // Keep the launcher's rounded widget corners even after the tree sets the root's colour.
        if (Build.VERSION.SDK_INT >= 31) root.setViewOutlinePreferredRadiusDimen(rootId, android.R.dimen.system_app_widget_background_radius);
        JSONObject snap = readJson(new File(dir(c, kind), "snapshot.json"));
        JSONObject tree = snap == null ? null : snap.optJSONObject("root");
        if (tree == null) {
            RemoteViews msg = new RemoteViews(pkg, res(c, "layout", "nsh_widgets_text_regular"));
            msg.setTextViewText(res(c, "id", "nsh_widgets_text"), "Open the app to set up this widget");
            root.addView(rootId, msg);
            return root;
        }
        JSONObject style = tree.optJSONObject("style");
        // The tree's root background and padding belong to the widget's own frame, as on iOS.
        if (style != null) {
            JSONArray rbg = style.optJSONArray("bg");
            if (rbg == null && style.optJSONObject("gradient") != null) rbg = style.optJSONObject("gradient").optJSONArray("from");
            pairColor(c, root, rootId, "setBackgroundColor", rbg);
            JSONArray pad = style.optJSONArray("pad");
            if (pad != null && pad.length() == 4) {
                root.setViewPadding(rootId, px(c, pad.optDouble(3)), px(c, pad.optDouble(0)), px(c, pad.optDouble(1)), px(c, pad.optDouble(2)));
            }
            try {
                JSONObject inner = new JSONObject(tree.toString());
                JSONObject s = inner.optJSONObject("style");
                s.remove("bg");
                s.remove("pad");
                s.remove("radius");
                s.remove("gradient");
                tree = inner;
            } catch (Exception ignored) {
            }
        }
        root.addView(rootId, build(c, tree, data(c, kind), kind, false, new int[] {100}, family));
        String url = snap.optString("url", "");
        if (!url.isEmpty()) {
            Intent open = new Intent(Intent.ACTION_VIEW, Uri.parse(url)).setPackage(pkg);
            root.setOnClickPendingIntent(rootId, PendingIntent.getActivity(c, 1, open, PendingIntent.FLAG_IMMUTABLE | PendingIntent.FLAG_UPDATE_CURRENT));
        }
        return root;
    }

    /** The IR's size names from the widget's current width, as the launcher reports it. */
    static String family(AppWidgetManager m, int id) {
        int w = m.getAppWidgetOptions(id).getInt(AppWidgetManager.OPTION_APPWIDGET_MIN_WIDTH, 250);
        return w < 200 ? "small" : w < 450 ? "medium" : "large";
    }

    /** The kind a provider slot draws, from its receiver's meta-data; null when the manifest names none. */
    static String kindOf(Context c, ComponentName provider) {
        try {
            android.content.pm.ActivityInfo info = c.getPackageManager().getReceiverInfo(provider, android.content.pm.PackageManager.GET_META_DATA);
            return info.metaData == null ? null : info.metaData.getString(KIND_META);
        } catch (Exception e) {
            return null;
        }
    }

    /** Redraws one placed widget. */
    static void draw(Context c, AppWidgetManager m, int id, String kind) {
        if (kind != null) m.updateAppWidget(id, views(c, kind, family(m, id)));
    }

    /** Redraws every placed widget of `kind` (every kind when null). Called by the app's Rust through JNI. */
    public static void update(Context c, String kind) {
        AppWidgetManager m = AppWidgetManager.getInstance(c);
        for (Class<?> slot : NshWidgetProvider.SLOTS) {
            ComponentName provider = new ComponentName(c, slot);
            String k = kindOf(c, provider);
            if (k == null || (kind != null && !kind.equals(k))) continue;
            for (int id : m.getAppWidgetIds(provider)) draw(c, m, id, k);
        }
    }
}
