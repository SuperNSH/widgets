// A Cinnamon desklet for one nsh-widgets kind, written by `nsh-widgets cinnamon`: the kind's `root` tree on
// the desktop. It reads through `nsh-widgets show --format ir` and presses through `nsh-widgets apply`.
const {Clutter, Gio, GLib, St} = imports.gi;
const ByteArray = imports.byteArray;
const Desklet = imports.ui.desklet;

//@RENDERER@

class NshWidgetDesklet extends Desklet.Desklet {
    constructor(metadata, id) {
        super(metadata, id);
        const [, bytes] = GLib.file_get_contents(`${metadata.path}/config.json`);
        this._nsh = JSON.parse(ByteArray.toString(bytes));
        this._cancel = new Gio.Cancellable();
        this._interface = new Gio.Settings({schema_id: 'org.x.apps.portal'});
        this._themeId = this._interface.connect('changed::color-scheme', () => this._draw());
        this._card = new St.Bin({style: 'width: 320px;'});
        this.setHeader(metadata.name);
        this.setContent(this._card);
        let n = 0;
        this._timer = GLib.timeout_add_seconds(GLib.PRIORITY_DEFAULT, 1, () => {
            for (const f of this._renderer?.ticks ?? []) f(Date.now());
            if (++n % 2 === 0) this._read();
            return GLib.SOURCE_CONTINUE;
        });
        this._draw();
        this._read();
    }

    on_desklet_removed() {
        GLib.source_remove(this._timer);
        this._cancel.cancel();
        this._interface.disconnect(this._themeId);
        this._renderer = null;
    }

    _read() {
        spawn(this._nsh.command, this._cancel, ['show', this._nsh.kind, ...this._nsh.source, '--format', 'ir'], out => {
            let s;
            try { s = JSON.parse(out); } catch (e) { s = null; }
            // Rebuild only for a real change, so a scrolled list keeps its place.
            const key = s ? JSON.stringify([s.snapshot, s.data]) : '';
            if (key === this._last) return;
            this._last = key;
            this._state = s;
            this._draw();
        });
    }

    _press(action) {
        if (action.startsWith('open:')) openUri(action.slice(5));
        else spawn(this._nsh.command, this._cancel, ['apply', this._nsh.kind, action, ...this._nsh.source], () => this._read());
    }

    _draw() {
        this._card.get_child()?.destroy();
        this._renderer = null;
        const root = this._state?.snapshot?.root;
        const dark = this._interface.get_string('color-scheme') === 'prefer-dark';
        this._renderer = new Renderer(this._state?.data ?? {}, dark, 'medium', a => this._press(a));
        this._card.set_child(card(this._renderer, root ?? {t: 'text', text: 'Open the app to set up this widget', style: {}}));
    }
}

function main(metadata, id) {
    return new NshWidgetDesklet(metadata, id);
}
