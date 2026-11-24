// A GNOME Shell top-bar widget for one nsh-widgets kind, written by `nsh-widgets gnome`: the kind's
// `accessory` tree in the bar and its `root` tree in the popup. The renderer is generic, as the others
// are. It reads through `nsh-widgets show --format ir` and presses through `nsh-widgets apply`.
import Clutter from 'gi://Clutter';
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import St from 'gi://St';
import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PanelMenu from 'resource:///org/gnome/shell/ui/panelMenu.js';
import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';

//@RENDERER@

export default class NshWidgetExtension extends Extension {
    enable() {
        const [, bytes] = GLib.file_get_contents(`${this.path}/config.json`);
        this._config = JSON.parse(new TextDecoder().decode(bytes));
        this._cancel = new Gio.Cancellable();
        this._settings = new Gio.Settings({schema_id: 'org.gnome.desktop.interface'});
        this._renderers = [];
        this._button = new PanelMenu.Button(0.5, this.metadata.name);
        this._bar = new St.Bin({y_align: Clutter.ActorAlign.CENTER});
        this._button.add_child(this._bar);
        this._item = new PopupMenu.PopupBaseMenuItem({reactive: false, can_focus: false});
        this._popup = new St.Bin({x_expand: true, style: 'width: 320px;'});
        this._item.add_child(this._popup);
        this._button.menu.addMenuItem(this._item);
        Main.panel.addToStatusArea(this.uuid, this._button);
        this._themeId = this._settings.connect('changed::color-scheme', () => this._render());
        let n = 0;
        this._timer = GLib.timeout_add_seconds(GLib.PRIORITY_DEFAULT, 1, () => {
            for (const r of this._renderers) for (const f of r.ticks) f(Date.now());
            if (++n % 2 === 0) this._read();
            return GLib.SOURCE_CONTINUE;
        });
        this._read();
    }

    disable() {
        GLib.source_remove(this._timer);
        this._cancel.cancel();
        this._settings.disconnect(this._themeId);
        this._button.destroy();
        this._button = this._bar = this._item = this._popup = this._settings = this._last = this._state = null;
        this._renderers = [];
    }

    _read() {
        spawn(this._config.command, this._cancel, ['show', this._config.kind, ...this._config.source, '--format', 'ir'], out => {
            let s;
            try { s = JSON.parse(out); } catch { s = null; }
            // Rebuild only for a real change, so a scrolled list keeps its place.
            const key = s ? JSON.stringify([s.snapshot, s.data]) : '';
            if (key === this._last) return;
            this._last = key;
            this._state = s;
            this._render();
        });
    }

    _run(action) {
        if (action.startsWith('open:')) {
            this._button.menu.close();
            openUri(action.slice(5));
            return;
        }
        spawn(this._config.command, this._cancel, ['apply', this._config.kind, action, ...this._config.source], () => this._read());
    }

    _render() {
        this._bar.child?.destroy();
        this._popup.child?.destroy();
        this._renderers = [];
        const s = this._state;
        if (!s?.snapshot?.root) {
            this._bar.set_child(new St.Label({text: this.metadata.name}));
            this._popup.set_child(new St.Label({text: 'Open the app to set up this widget'}));
            return;
        }
        const {root, accessory} = s.snapshot, run = a => this._run(a);
        // The top bar is dark in both colour schemes.
        const bar = new Renderer(s.data, true, 'inline', run);
        this._bar.set_child(accessory ? bar.build(accessory) : new St.Label({text: this.metadata.name}));
        const dark = this._settings.get_string('color-scheme') === 'prefer-dark';
        const full = new Renderer(s.data, dark, 'medium', run);
        this._popup.set_child(full.build(framed(root)));
        this._renderers = [bar, full];
    }
}
