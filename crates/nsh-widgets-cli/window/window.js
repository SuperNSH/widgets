#!/usr/bin/env -S gjs
// A desktop window for one nsh-widgets kind, written by `nsh-widgets window`: a layer-shell surface under the
// windows on Wayland compositors that have the protocol (wlroots, KWin, COSMIC), a window kept below the
// others on X11. It reads through `nsh-widgets show --format ir` and presses through `nsh-widgets apply`.
imports.gi.versions.Gtk = '3.0';
imports.gi.versions.Gdk = '3.0';
const {Gdk, Gio, GLib, Gtk} = imports.gi;
const System = imports.system;
let Layer = null;
try { Layer = imports.gi.GtkLayerShell; } catch (e) { /* X11 does not need it */ }

//@RENDERER@

const here = Gio.File.new_for_path(System.programInvocationName).get_parent().get_path();
const config = JSON.parse(imports.byteArray.toString(GLib.file_get_contents(`${here}/config.json`)[1]));

Gtk.init(null);
const win = new Gtk.Window({title: `nsh-widgets ${config.kind}`, decorated: false, resizable: false, app_paintable: true, skip_taskbar_hint: true, skip_pager_hint: true, accept_focus: false});
const visual = win.get_screen().get_rgba_visual();
if (visual) win.set_visual(visual); // for the rounded corners
style(win, 'background: transparent;');
const [vertical, horizontal] = config.anchor.split('-');
const wayland = win.get_display().constructor.name.includes('Wayland');
if (wayland) {
    if (!Layer || !Layer.is_supported()) {
        printerr('nsh-widgets: this Wayland compositor has no layer-shell, or gtk-layer-shell is not installed (GNOME: use `nsh-widgets gnome`)');
        System.exit(1);
    }
    Layer.init_for_window(win);
    Layer.set_namespace(win, 'nsh-widgets');
    Layer.set_layer(win, Layer.Layer.BOTTOM);
    for (const [edge, on] of [[Layer.Edge.TOP, vertical === 'top'], [Layer.Edge.BOTTOM, vertical === 'bottom'], [Layer.Edge.LEFT, horizontal === 'left'], [Layer.Edge.RIGHT, horizontal === 'right']]) {
        Layer.set_anchor(win, edge, on);
        Layer.set_margin(win, edge, config.margin[edge === Layer.Edge.TOP || edge === Layer.Edge.BOTTOM ? 1 : 0]);
    }
} else {
    win.set_type_hint(Gdk.WindowTypeHint.UTILITY);
    win.set_keep_below(true);
    win.stick();
    const place = () => {
        const d = win.get_display(), area = (d.get_primary_monitor() ?? d.get_monitor(0)).get_workarea(), [w, h] = win.get_size();
        win.move(horizontal === 'left' ? area.x + config.margin[0] : area.x + area.width - w - config.margin[0],
            vertical === 'top' ? area.y + config.margin[1] : area.y + area.height - h - config.margin[1]);
    };
    win.connect('size-allocate', place);
}

const settings = Gtk.Settings.get_default();
const schema = Gio.SettingsSchemaSource.get_default()?.lookup('org.gnome.desktop.interface', true);
const look = schema?.has_key('color-scheme') ? new Gio.Settings({settings_schema: schema}) : null;
const isDark = () => look?.get_string('color-scheme') === 'prefer-dark' || settings.gtk_application_prefer_dark_theme || /-dark$/i.test(settings.gtk_theme_name ?? '');

const cancel = new Gio.Cancellable();
let state = null, last = null, renderer = null;

function draw() {
    win.get_child()?.destroy();
    renderer = new Renderer(state?.data ?? {}, isDark(), 'medium', press);
    const tree = renderer.build(framed(state?.snapshot?.root ?? EMPTY));
    tree.set_size_request(config.width, -1);
    win.add(tree);
    win.show_all();
}

function read() {
    spawn(config.command, cancel, ['show', config.kind, ...config.source, '--format', 'ir'], out => {
        let s;
        try { s = JSON.parse(out); } catch (e) { s = null; }
        // Rebuild only for a real change, so a scrolled list keeps its place.
        const key = s ? JSON.stringify([s.snapshot, s.data]) : '';
        if (key === last) return;
        last = key;
        state = s;
        draw();
    });
}

function press(action) {
    if (action.startsWith('open:')) openUri(action.slice(5));
    else spawn(config.command, cancel, ['apply', config.kind, action, ...config.source], read);
}

look?.connect('changed::color-scheme', draw);
settings.connect('notify::gtk-theme-name', draw);
settings.connect('notify::gtk-application-prefer-dark-theme', draw);
let n = 0;
GLib.timeout_add_seconds(GLib.PRIORITY_DEFAULT, 1, () => {
    for (const f of renderer?.ticks ?? []) f(Date.now());
    if (++n % 2 === 0) read();
    return GLib.SOURCE_CONTINUE;
});
win.connect('destroy', () => Gtk.main_quit());
draw();
read();
Gtk.main();
