package dev.supernsh.widgets;

import android.appwidget.AppWidgetManager;
import android.appwidget.AppWidgetProvider;
import android.content.ComponentName;
import android.content.Context;
import android.os.Bundle;

/**
 * One widget kind. Android names a provider by class, so the library ships numbered slots and the
 * manifest `nsh-widgets build` writes gives each kind a slot, with the kind as receiver meta-data.
 */
public class NshWidgetProvider extends AppWidgetProvider {
    static final Class<?>[] SLOTS = {
        Slot1.class, Slot2.class, Slot3.class, Slot4.class, Slot5.class, Slot6.class, Slot7.class, Slot8.class,
        Slot9.class, Slot10.class, Slot11.class, Slot12.class, Slot13.class, Slot14.class, Slot15.class, Slot16.class};

    private String kind(Context c) {
        return NshWidgets.kindOf(c, new ComponentName(c, getClass()));
    }

    @Override
    public void onUpdate(Context c, AppWidgetManager m, int[] ids) {
        String kind = kind(c);
        for (int id : ids) NshWidgets.draw(c, m, id, kind);
    }

    @Override
    public void onAppWidgetOptionsChanged(Context c, AppWidgetManager m, int id, Bundle options) {
        NshWidgets.draw(c, m, id, kind(c));
    }

    public static class Slot1 extends NshWidgetProvider {}
    public static class Slot2 extends NshWidgetProvider {}
    public static class Slot3 extends NshWidgetProvider {}
    public static class Slot4 extends NshWidgetProvider {}
    public static class Slot5 extends NshWidgetProvider {}
    public static class Slot6 extends NshWidgetProvider {}
    public static class Slot7 extends NshWidgetProvider {}
    public static class Slot8 extends NshWidgetProvider {}
    public static class Slot9 extends NshWidgetProvider {}
    public static class Slot10 extends NshWidgetProvider {}
    public static class Slot11 extends NshWidgetProvider {}
    public static class Slot12 extends NshWidgetProvider {}
    public static class Slot13 extends NshWidgetProvider {}
    public static class Slot14 extends NshWidgetProvider {}
    public static class Slot15 extends NshWidgetProvider {}
    public static class Slot16 extends NshWidgetProvider {}
}
