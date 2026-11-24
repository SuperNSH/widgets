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

const GLYPHS = {
    star: '★', bolt: '⚡', check: '✓', play: '▶', pause: '⏸', flame: '🔥', plus: '+', minus: '−', 'arrow-up': '↑',
    'arrow-down': '↓', heart: '♥', home: '⌂', person: '👤', bell: '🔔', clock: '🕒', calendar: '📅', cloud: '☁',
    sun: '☀', moon: '☾', drop: '💧', music: '♪',
};
const WEIGHTS = {medium: 500, semibold: 600, bold: 700};
const num = (v, d) => (Number.isFinite(v) ? v : d);

class Renderer {
    constructor(data, dark, family, run) {
        Object.assign(this, {data, dark, family, run});
        this.ticks = []; // called every second with the time: timers, clocks and rotating children
    }

    color(pair) {
        const c = Array.isArray(pair) ? pair[this.dark ? 1 : 0] : null;
        return /^#[0-9a-f]{6}$/i.test(c ?? '') ? c : null;
    }

    fill(text, data) {
        return String(text ?? '').replace(/\{([A-Za-z_][\w.]*)\}/g, (m, k) => (data[k] === undefined ? m : String(data[k])));
    }

    css(node, isText) {
        const s = node.style ?? {}, out = [];
        const pad = Array.isArray(s.pad) ? s.pad.map(p => num(p, 0)) : null;
        if (pad) out.push(`padding: ${pad.map(p => `${p}px`).join(' ')}`);
        const from = this.color(s.gradient?.from), to = this.color(s.gradient?.to), bg = this.color(s.bg);
        if (from && to) {
            out.push(`background-gradient-direction: ${s.gradient.dir === 'r' ? 'horizontal' : 'vertical'}`,
                `background-gradient-start: ${from}`, `background-gradient-end: ${to}`);
        } else if (bg) {
            out.push(`background-color: ${bg}`);
        }
        if (num(s.radius, 0) > 0) out.push(`border-radius: ${Math.min(s.radius, 99)}px`);
        if (!isText && num(s.gap, 0) > 0) out.push(`spacing: ${s.gap}px`);
        if (isText) {
            out.push(`font-size: ${num(s.size, 14)}px`, `font-weight: ${WEIGHTS[s.weight] ?? 400}`,
                `color: ${this.color(s.fg) ?? (this.dark ? '#ffffff' : '#111111')}`);
        }
        return `${out.join('; ')};`;
    }

    label(node, text) {
        return new St.Label({text, style: this.css(node, true), y_align: Clutter.ActorAlign.CENTER});
    }

    pressable(child, node, action) {
        const b = new St.Button({child, style: this.css(node, true), x_align: Clutter.ActorAlign.START, can_focus: true});
        b.connect('clicked', () => this.run(action));
        return b;
    }

    /** The actor for `node`, or null when it is hidden or unknown. */
    build(node, data = this.data, inRow = false) {
        const s = node?.style ?? {};
        if (!node || s.hide?.includes(this.family)) return null;
        switch (node.t) {
        case 'col':
        case 'row': {
            const row = node.t === 'row';
            const box = new St.BoxLayout({vertical: !row, style: this.css(node, false), x_expand: !inRow, y_expand: s.grow === true});
            for (const c of node.children ?? []) {
                const a = this.build(c, data, row);
                if (a) box.add_child(a);
            }
            return box;
        }
        case 'text': return this.label(node, this.fill(node.text, data));
        case 'spacer': return new St.Widget({x_expand: inRow, y_expand: !inRow});
        case 'button': return this.pressable(new St.Label({text: this.fill(node.text, data)}), node, node.action ?? '');
        case 'icon': return this.label({style: {...s, size: s.height ?? s.size ?? 18}}, GLYPHS[node.name] ?? '?');
        case 'progress': {
            const h = num(s.height, 8), v = Math.max(0, Math.min(1, num(data[node.key], 0)));
            const track = new St.Widget({x_expand: true, height: h,
                style: `border-radius: ${h}px; background-color: ${this.color(s.bg) ?? 'rgba(127,127,127,0.25)'};`});
            const bar = new St.Widget({height: h, width: 0, style: `border-radius: ${h}px; background-color: ${this.color(s.fg) ?? '#2563eb'};`});
            track.add_child(bar);
            track.connect('notify::width', () => { bar.width = Math.round(track.width * v); });
            return track;
        }
        case 'bars': {
            const values = (Array.isArray(data[node.key]) ? data[node.key] : []).filter(Number.isFinite);
            const top = Math.max(1, ...values), h = num(s.height, 48);
            const box = new St.BoxLayout({x_expand: true, height: h, style: `spacing: ${num(s.gap, 4)}px;`});
            for (const v of values) {
                box.add_child(new St.Widget({x_expand: true, y_align: Clutter.ActorAlign.END, y_expand: true,
                    height: Math.max(2, Math.round(h * v / top)),
                    style: `border-radius: 3px; background-color: ${this.color(s.fg) ?? '#2563eb'};`}));
            }
            return box;
        }
        case 'timer':
        case 'clock': {
            const l = this.label(node, '');
            this.tick(now => {
                const at = num(data[node.key], now);
                l.text = node.t === 'clock' ? GLib.DateTime.new_now_local().format('%H:%M')
                    : duration(node.mode === 'down' ? at - now : now - at);
            });
            return l;
        }
        case 'toggle': {
            const on = data[node.key] === true, size = num(s.size, 14);
            const row = new St.BoxLayout({style: 'spacing: 8px;'});
            row.add_child(this.label({style: {...s, size: size + 4}}, on ? '☑' : '☐'));
            const l = this.label(node, this.fill(node.text, data));
            if (on) l.style += 'text-decoration: line-through;';
            l.opacity = on ? 140 : 255;
            row.add_child(l);
            return this.pressable(row, {}, node.action ?? `toggle:${node.key}`);
        }
        case 'rotate': {
            const kids = node.children ?? [], bin = new St.Bin({x_align: Clutter.ActorAlign.START});
            let shown = -1, own = []; // the shown child's ticks, dropped with it
            if (kids.length) {
                this.tick(now => {
                    const i = Math.floor(now / (1000 * Math.max(1, num(node.interval, 60)))) % kids.length;
                    if (i !== shown) {
                        shown = i;
                        bin.child?.destroy();
                        const outer = this.ticks;
                        this.ticks = [];
                        bin.set_child(this.build(kids[i], data));
                        own = this.ticks;
                        this.ticks = outer;
                    }
                    for (const f of own) f(now);
                });
            }
            return bin;
        }
        case 'list': {
            const box = new St.BoxLayout({vertical: true, x_expand: true, style: `spacing: ${num(s.gap, 0)}px;`});
            const items = Array.isArray(data[node.key]) ? data[node.key] : [];
            for (const item of items.slice(0, num(node.max, 8))) {
                const d = {...data};
                for (const k in item) d[`item.${k}`] = item[k];
                const a = this.build(node.children?.[0], d);
                if (a) box.add_child(a);
            }
            const view = new St.ScrollView({x_expand: true, style: 'max-height: 160px;', hscrollbar_policy: St.PolicyType.NEVER});
            if (view.set_child) view.set_child(box);
            else view.add_actor(box);
            return view;
        }
        default: return null;
        }
    }

    tick(f) {
        this.ticks.push(f);
        f(Date.now());
    }
}

function duration(ms) {
    const t = Math.max(0, Math.floor(ms / 1000)), h = Math.floor(t / 3600), m = Math.floor(t / 60) % 60, x = t % 60;
    const p = v => String(v).padStart(2, '0');
    return h > 0 ? `${h}:${p(m)}:${p(x)}` : `${m}:${p(x)}`;
}

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

    /** Runs nsh-widgets with an argument list (never a shell) and hands over its output. */
    _spawn(args, done) {
        const cancel = this._cancel;
        try {
            const p = Gio.Subprocess.new([this._config.command, ...args], Gio.SubprocessFlags.STDOUT_PIPE | Gio.SubprocessFlags.STDERR_SILENCE);
            p.communicate_utf8_async(null, cancel, (_, res) => {
                try {
                    const [, out] = p.communicate_utf8_finish(res);
                    if (!cancel.is_cancelled()) done(out);
                } catch (e) {
                    if (!cancel.is_cancelled()) console.warn(`nsh-widgets: ${e.message}`);
                }
            });
        } catch (e) {
            console.warn(`nsh-widgets: ${e.message}`);
        }
    }

    _read() {
        this._spawn(['show', this._config.kind, ...this._config.source, '--format', 'ir'], out => {
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
            try {
                Gio.AppInfo.launch_default_for_uri(action.slice(5), null);
            } catch (e) {
                console.warn(`nsh-widgets: ${e.message}`);
            }
            return;
        }
        this._spawn(['apply', this._config.kind, action, ...this._config.source], () => this._read());
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
        const st = root.style ?? {};
        const base = st.bg || st.gradient ? {} : {bg: ['#ffffff', '#1c1c1e']};
        const tree = full.build({...root, style: {pad: [16, 16, 16, 16], ...base, ...st, radius: 12}});
        this._popup.set_child(tree);
        this._renderers = [bar, full];
    }
}
