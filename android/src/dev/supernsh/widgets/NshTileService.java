package dev.supernsh.widgets;

import android.content.ComponentName;
import android.content.pm.PackageManager;
import android.os.Bundle;
import android.service.quicksettings.Tile;
import android.service.quicksettings.TileService;

/**
 * A Quick Settings tile: Android's counterpart of an iOS control (widgets.json "controls"). Android
 * names a tile by its service class, so the library ships numbered slots and the app's manifest gives
 * each control one, with its data kind, key or action as meta-data.
 */
public class NshTileService extends TileService {
    static final Class<?>[] SLOTS = {Slot1.class, Slot2.class, Slot3.class, Slot4.class, Slot5.class, Slot6.class, Slot7.class, Slot8.class};

    private Bundle meta() {
        try {
            Bundle b = getPackageManager().getServiceInfo(new ComponentName(this, getClass()), PackageManager.GET_META_DATA).metaData;
            return b == null ? new Bundle() : b;
        } catch (Exception e) {
            return new Bundle();
        }
    }

    private void refresh() {
        Tile tile = getQsTile();
        if (tile == null) return;
        Bundle m = meta();
        String kind = m.getString("dev.supernsh.widgets.data"), key = m.getString("dev.supernsh.widgets.key");
        if (kind != null && key != null) {
            tile.setState(NshWidgets.data(this, kind).optBoolean(key, false) ? Tile.STATE_ACTIVE : Tile.STATE_INACTIVE);
        } else {
            tile.setState(Tile.STATE_INACTIVE);
        }
        tile.updateTile();
    }

    @Override
    public void onStartListening() {
        refresh();
    }

    @Override
    public void onClick() {
        Bundle m = meta();
        String kind = m.getString("dev.supernsh.widgets.data");
        String key = m.getString("dev.supernsh.widgets.key"), action = m.getString("dev.supernsh.widgets.action");
        if (kind == null) return;
        NshWidgets.apply(this, kind, key != null ? "toggle:" + key : action == null ? "" : action);
        NshWidgets.update(this, kind);
        refresh();
    }

    public static class Slot1 extends NshTileService {}
    public static class Slot2 extends NshTileService {}
    public static class Slot3 extends NshTileService {}
    public static class Slot4 extends NshTileService {}
    public static class Slot5 extends NshTileService {}
    public static class Slot6 extends NshTileService {}
    public static class Slot7 extends NshTileService {}
    public static class Slot8 extends NshTileService {}
}
