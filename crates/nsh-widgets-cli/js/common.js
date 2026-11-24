// What the JavaScript renderers share. `nsh-widgets gnome`, `cinnamon` and `window` paste this file and the
// renderer into the scripts they write, under the scripts' own imports of Gio and GLib.
const GLYPHS = {
    star: '★', bolt: '⚡', check: '✓', play: '▶', pause: '⏸', flame: '🔥', plus: '+', minus: '−', 'arrow-up': '↑',
    'arrow-down': '↓', heart: '♥', home: '⌂', person: '👤', bell: '🔔', clock: '🕒', calendar: '📅', cloud: '☁',
    sun: '☀', moon: '☾', drop: '💧', music: '♪',
};
const WEIGHTS = {medium: 500, semibold: 600, bold: 700};
const num = (v, d) => (Number.isFinite(v) ? v : d);

class Base {
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

/** The root with the frame every host gives it. */
function framed(root) {
    const st = root.style ?? {};
    const base = st.bg || st.gradient ? {} : {bg: ['#ffffff', '#1c1c1e']};
    return {...root, style: {pad: [16, 16, 16, 16], ...base, ...st, radius: 12}};
}

/** What a kind the app has not published yet draws. */
const EMPTY = {t: 'text', text: 'Open the app to set up this widget', style: {}};
