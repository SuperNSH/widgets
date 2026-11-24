// Draws a widget tree with St. `nsh-widgets gnome` and `nsh-widgets cinnamon` paste this file into the
// extension and the desklet they write, under their own imports of Clutter, Gio, GLib and St.
const GLYPHS = {
    star: '★', bolt: '⚡', check: '✓', play: '▶', pause: '⏸', flame: '🔥', plus: '+', minus: '−', 'arrow-up': '↑',
    'arrow-down': '↓', heart: '♥', home: '⌂', person: '👤', bell: '🔔', clock: '🕒', calendar: '📅', cloud: '☁',
    sun: '☀', moon: '☾', drop: '💧', music: '♪',
};
const WEIGHTS = {medium: 500, semibold: 600, bold: 700};
/** A row or column: St.BoxLayout's own property for it differs between GNOME 45 and 51, its layout manager's does not. */
function box(vertical, props) {
    const b = new St.BoxLayout(props);
    b.layout_manager.orientation = vertical ? Clutter.Orientation.VERTICAL : Clutter.Orientation.HORIZONTAL;
    return b;
}
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
            const b = box(!row, {style: this.css(node, false), x_expand: !inRow, y_expand: s.grow === true});
            for (const c of node.children ?? []) {
                const a = this.build(c, data, row);
                if (a) b.add_child(a);
            }
            return b;
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
            const bars = new St.BoxLayout({x_expand: true, height: h, style: `spacing: ${num(s.gap, 4)}px;`});
            for (const v of values) {
                bars.add_child(new St.Widget({x_expand: true, y_align: Clutter.ActorAlign.END, y_expand: true,
                    height: Math.max(2, Math.round(h * v / top)),
                    style: `border-radius: 3px; background-color: ${this.color(s.fg) ?? '#2563eb'};`}));
            }
            return bars;
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
            const rows = box(true, {x_expand: true, style: `spacing: ${num(s.gap, 0)}px;`});
            const items = Array.isArray(data[node.key]) ? data[node.key] : [];
            for (const item of items.slice(0, num(node.max, 8))) {
                const d = {...data};
                for (const k in item) d[`item.${k}`] = item[k];
                const a = this.build(node.children?.[0], d);
                if (a) rows.add_child(a);
            }
            const view = new St.ScrollView({x_expand: true, style: 'max-height: 160px;', hscrollbar_policy: St.PolicyType.NEVER});
            // GNOME 45 has a set_child here too, St.Bin's, and using it aborts the shell when the view goes.
            if (view.add_actor) view.add_actor(rows);
            else view.set_child(rows);
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

/** Runs nsh-widgets with an argument list (never a shell) and hands over its output. */
function spawn(command, cancel, args, done) {
    try {
        const p = Gio.Subprocess.new([command, ...args], Gio.SubprocessFlags.STDOUT_PIPE | Gio.SubprocessFlags.STDERR_SILENCE);
        p.communicate_utf8_async(null, cancel, (_, res) => {
            try {
                const [, out] = p.communicate_utf8_finish(res);
                if (!cancel.is_cancelled()) done(out);
            } catch (e) {
                if (!cancel.is_cancelled()) logError(e, 'nsh-widgets');
            }
        });
    } catch (e) {
        logError(e, 'nsh-widgets');
    }
}

function openUri(uri) {
    try {
        Gio.AppInfo.launch_default_for_uri(uri, null);
    } catch (e) {
        log(`nsh-widgets: ${e.message}`);
    }
}

/** The actor for a whole widget: the root on its rounded card. */
function card(renderer, root) {
    const st = root.style ?? {};
    const base = st.bg || st.gradient ? {} : {bg: ['#ffffff', '#1c1c1e']};
    return renderer.build({...root, style: {pad: [16, 16, 16, 16], ...base, ...st, radius: 12}});
}
