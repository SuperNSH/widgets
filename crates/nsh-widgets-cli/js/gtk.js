// Draws a widget tree with GTK 3, for the X11 and layer-shell window.
function style(widget, css) {
    const provider = new Gtk.CssProvider();
    provider.load_from_data(`* { ${css} }`);
    widget.get_style_context().add_provider(provider, Gtk.STYLE_PROVIDER_PRIORITY_USER);
    return widget;
}

class Renderer extends Base {
    css(node, isText) {
        const s = node.style ?? {}, out = [];
        const pad = Array.isArray(s.pad) ? s.pad.map(p => num(p, 0)) : null;
        if (pad) out.push(`padding: ${pad.map(p => `${p}px`).join(' ')}`);
        const from = this.color(s.gradient?.from), to = this.color(s.gradient?.to), bg = this.color(s.bg);
        if (from && to) out.push(`background-image: linear-gradient(to ${{r: 'right', br: 'bottom right'}[s.gradient.dir] ?? 'bottom'}, ${from}, ${to})`);
        else if (bg) out.push(`background-color: ${bg}`);
        if (num(s.radius, 0) > 0) out.push(`border-radius: ${Math.min(s.radius, 99)}px`);
        if (isText) {
            out.push(`font-size: ${num(s.size, 14)}px`, `font-weight: ${WEIGHTS[s.weight] ?? 400}`,
                `color: ${this.color(s.fg) ?? (this.dark ? '#ffffff' : '#111111')}`);
        }
        return `${out.join('; ')};`;
    }

    label(node, text) {
        return style(new Gtk.Label({label: text, xalign: 0}), this.css(node, true));
    }

    pressable(child, node, action) {
        const b = new Gtk.Button({halign: Gtk.Align.START, valign: Gtk.Align.CENTER, relief: Gtk.ReliefStyle.NONE, can_focus: false});
        b.add(child);
        style(b, `border: none; box-shadow: none; text-shadow: none; background-image: none; background-color: transparent; min-width: 0; min-height: 0; padding: 0; ${this.css(node, true)}`);
        b.connect('clicked', () => this.run(action));
        return b;
    }

    /** A drawing area `h` pixels high, drawn by `paint(cairo, width, height)`. */
    canvas(h, paint) {
        const a = new Gtk.DrawingArea({hexpand: true, height_request: h});
        a.connect('draw', (w, cr) => {
            paint(cr, w.get_allocated_width(), w.get_allocated_height());
            cr.$dispose();
            return false;
        });
        return a;
    }

    /** The widget for `node`, or null when it is hidden or unknown. */
    build(node, data = this.data, inRow = false) {
        const s = node?.style ?? {};
        if (!node || s.hide?.includes(this.family)) return null;
        switch (node.t) {
        case 'col':
        case 'row': {
            const row = node.t === 'row';
            const b = new Gtk.Box({orientation: row ? Gtk.Orientation.HORIZONTAL : Gtk.Orientation.VERTICAL, spacing: num(s.gap, 0)});
            if (row) b.valign = Gtk.Align.CENTER;
            if (inRow && s.grow !== true) b.halign = Gtk.Align.START;
            for (const c of node.children ?? []) {
                const a = this.build(c, data, row);
                if (!a) continue;
                if (row) a.valign = Gtk.Align.CENTER;
                const grow = c.t === 'spacer' || (row && ['progress', 'bars'].includes(c.t)) || c.style?.grow === true;
                b.pack_start(a, grow, true, 0);
            }
            return style(b, this.css(node, false));
        }
        case 'text': return this.label(node, this.fill(node.text, data));
        case 'spacer': return new Gtk.Box();
        case 'button': return this.pressable(new Gtk.Label({label: this.fill(node.text, data)}), node, node.action ?? '');
        case 'icon': return this.label({style: {...s, size: s.height ?? s.size ?? 18}}, GLYPHS[node.name] ?? '?');
        case 'progress': {
            const v = Math.max(0, Math.min(1, num(data[node.key], 0))), track = this.color(s.bg), bar = this.color(s.fg) ?? '#2563eb';
            return this.canvas(num(s.height, 8), (cr, w, h) => {
                rounded(cr, 0, 0, w, h, h / 2, track, 0.25);
                if (v > 0) rounded(cr, 0, 0, Math.max(h, w * v), h, h / 2, bar, 1);
            });
        }
        case 'bars': {
            const values = (Array.isArray(data[node.key]) ? data[node.key] : []).filter(Number.isFinite);
            const top = Math.max(1, ...values), gap = num(s.gap, 4), color = this.color(s.fg) ?? '#2563eb';
            return this.canvas(num(s.height, 48), (cr, w, h) => {
                const each = (w - gap * (values.length - 1)) / values.length;
                values.forEach((v, i) => {
                    const high = Math.max(2, h * v / top);
                    rounded(cr, i * (each + gap), h - high, each, high, Math.min(3, high / 2), color, 1);
                });
            });
        }
        case 'timer':
        case 'clock': {
            const l = this.label(node, '');
            this.tick(now => {
                const at = num(data[node.key], now);
                l.label = node.t === 'clock' ? GLib.DateTime.new_now_local().format('%H:%M')
                    : duration(node.mode === 'down' ? at - now : now - at);
            });
            return l;
        }
        case 'toggle': {
            const on = data[node.key] === true, size = num(s.size, 14);
            const row = new Gtk.Box({spacing: 8});
            row.add(this.label({style: {...s, size: size + 4}}, on ? '☑' : '☐'));
            const l = this.label(node, this.fill(node.text, data));
            if (on) style(l, 'text-decoration-line: line-through;');
            l.opacity = on ? 0.55 : 1;
            row.add(l);
            return this.pressable(row, {}, node.action ?? `toggle:${node.key}`);
        }
        case 'rotate': {
            const kids = node.children ?? [], bin = new Gtk.Box({halign: Gtk.Align.START});
            let shown = -1, own = []; // the shown child's ticks, dropped with it
            if (kids.length) {
                this.tick(now => {
                    const i = Math.floor(now / (1000 * Math.max(1, num(node.interval, 60)))) % kids.length;
                    if (i !== shown) {
                        shown = i;
                        for (const c of bin.get_children()) c.destroy();
                        const outer = this.ticks;
                        this.ticks = [];
                        const kid = this.build(kids[i], data);
                        if (kid) bin.add(kid);
                        bin.show_all();
                        own = this.ticks;
                        this.ticks = outer;
                    }
                    for (const f of own) f(now);
                });
            }
            return bin;
        }
        case 'list': {
            const rows = new Gtk.Box({orientation: Gtk.Orientation.VERTICAL, spacing: num(s.gap, 0)});
            const items = Array.isArray(data[node.key]) ? data[node.key] : [];
            for (const item of items.slice(0, num(node.max, 8))) {
                const d = {...data};
                for (const k in item) d[`item.${k}`] = item[k];
                const a = this.build(node.children?.[0], d);
                if (a) rows.add(a);
            }
            const view = new Gtk.ScrolledWindow({hscrollbar_policy: Gtk.PolicyType.NEVER, propagate_natural_height: true, max_content_height: 160});
            view.add(rows);
            return view;
        }
        default: return null;
        }
    }
}

/** A filled rounded rectangle; grey at `alpha` without a colour. */
function rounded(cr, x, y, w, h, r, color, alpha) {
    const c = new Gdk.RGBA();
    c.parse(color ?? '#7f7f7f');
    cr.setSourceRGBA(c.red, c.green, c.blue, color ? 1 : alpha);
    r = Math.min(r, w / 2, h / 2);
    cr.newSubPath();
    cr.arc(x + w - r, y + r, r, -Math.PI / 2, 0);
    cr.arc(x + w - r, y + h - r, r, 0, Math.PI / 2);
    cr.arc(x + r, y + h - r, r, Math.PI / 2, Math.PI);
    cr.arc(x + r, y + r, r, Math.PI, 1.5 * Math.PI);
    cr.closePath();
    cr.fill();
}
