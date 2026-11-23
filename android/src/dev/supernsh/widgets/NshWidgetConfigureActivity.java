package dev.supernsh.widgets;

import android.app.Activity;
import android.appwidget.AppWidgetManager;
import android.appwidget.AppWidgetProviderInfo;
import android.content.Intent;
import android.os.Bundle;
import android.widget.ArrayAdapter;
import android.widget.ListView;

/**
 * The configure screen of a kind with options: lists the values the app's widgets.json gave the kind
 * (the slot's meta-data) and keeps the choice per placed widget.
 */
public class NshWidgetConfigureActivity extends Activity {
    @Override
    protected void onCreate(Bundle state) {
        super.onCreate(state);
        Bundle extras = getIntent().getExtras();
        final int id = extras == null ? AppWidgetManager.INVALID_APPWIDGET_ID
            : extras.getInt(AppWidgetManager.EXTRA_APPWIDGET_ID, AppWidgetManager.INVALID_APPWIDGET_ID);
        setResult(RESULT_CANCELED, new Intent().putExtra(AppWidgetManager.EXTRA_APPWIDGET_ID, id));
        AppWidgetManager m = AppWidgetManager.getInstance(this);
        AppWidgetProviderInfo info = id == AppWidgetManager.INVALID_APPWIDGET_ID ? null : m.getAppWidgetInfo(id);
        if (info == null) {
            finish();
            return;
        }
        final String kind = NshWidgets.kindOf(this, info.provider);
        final String[] values = NshWidgets.optionsOf(this, info.provider);
        setTitle(NshWidgets.optionsTitleOf(this, info.provider));
        ListView list = new ListView(this);
        list.setAdapter(new ArrayAdapter<>(this, android.R.layout.simple_list_item_single_choice, values));
        list.setChoiceMode(ListView.CHOICE_MODE_SINGLE);
        String current = NshWidgets.optionFor(this, id, values);
        for (int i = 0; i < values.length; i++) if (values[i].equals(current)) list.setItemChecked(i, true);
        list.setOnItemClickListener((parent, view, position, rowId) -> {
            NshWidgets.setOption(this, id, values[position]);
            NshWidgets.draw(this, m, id, kind);
            setResult(RESULT_OK, new Intent().putExtra(AppWidgetManager.EXTRA_APPWIDGET_ID, id));
            finish();
        });
        setContentView(list);
    }
}
