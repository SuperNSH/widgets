import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Shell from 'gi://Shell';
import St from 'gi://St';
import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
const sleep = ms => new Promise(r => GLib.timeout_add(GLib.PRIORITY_DEFAULT, ms, () => { r(); return GLib.SOURCE_REMOVE; }));
const id = k => `dev.supernsh.widgets.gallery.${k}@widgets.supernsh.dev`;
const all = (a, f, out = []) => { if (f(a)) out.push(a); for (const c of a.get_children?.() ?? []) all(c, f, out); return out; };
const texts = a => all(a, x => x instanceof St.Label).map(l => l.text);
const data = k => JSON.parse(new TextDecoder().decode(GLib.file_get_contents(`${GLib.get_user_data_dir()}/nsh-widgets/dev.supernsh.widgets.gallery/nsh/widgets/${k}/data.json`)[1])).data;
const out = {};
const shot = async n => {
    try {
        const s = new Shell.Screenshot(), f = Gio.File.new_for_path(`/tmp/probe/${n}.png`);
        const st = f.replace(null, false, Gio.FileCreateFlags.NONE, null);
        await new Promise((res, rej) => s.screenshot(false, st, (o, r) => { try { s.screenshot_finish(r); st.close(null); res(); } catch (e) { rej(e); } }));
    } catch (e) { out[`shot_${n}`] = String(e); }
};
const press = (root, match) => {
    const b = all(root, x => x instanceof St.Button).find(x => texts(x).some(t => t === match));
    if (!b) return false;
    b.emit('clicked', 1);
    return true;
};
async function run() {
    const area = k => Main.panel.statusArea[id(k)], EM = Main.extensionManager;
    out.states = Object.fromEntries(['dashboard', 'tasks', 'agenda', 'steps', 'kinds', 'ghost'].map(k => [k, EM.lookup(id(k))?.state + (EM.lookup(id(k))?.error ? ` ${EM.lookup(id(k)).error}` : '')]));
    Main.overview.hide();
    await sleep(2500);
    out.bar = texts(area('steps'));
    area('dashboard').menu.open();
    await sleep(1500);
    out.dash = texts(area('dashboard').menu.actor);
    const s0 = data('Dashboard').streak;
    out.pressed = press(area('dashboard').menu.actor, '+1 day');
    await sleep(3500);
    out.streak = [s0, data('Dashboard').streak, texts(area('dashboard').menu.actor).find(t => t.includes('streak'))];
    await shot('dashboard');
    new Gio.Settings({schema_id: 'org.gnome.desktop.interface'}).set_string('color-scheme', 'prefer-dark');
    await sleep(1500);
    const card = area('dashboard').menu.actor;
    out.dark = all(card, x => x.style?.includes('border-radius: 12px')).map(x => x.style);
    await shot('dashboard-dark');
    area('dashboard').menu.close();
    await sleep(600);
    area('tasks').menu.open();
    await sleep(1000);
    const t0 = data('Tasks').t3;
    out.toggled = press(area('tasks').menu.actor, 'Update the plan');
    await sleep(3500);
    out.t3 = [t0, data('Tasks').t3];
    await shot('tasks');
    area('tasks').menu.close();
    await sleep(600);
    area('agenda').menu.open();
    await sleep(1000);
    const view = all(area('agenda').menu.actor, x => x instanceof St.ScrollView)[0];
    const adj = view.vadjustment ?? view.vscroll.adjustment;
    adj.value = adj.upper;
    const v0 = adj.value;
    await sleep(6500);
    const view2 = all(area('agenda').menu.actor, x => x instanceof St.ScrollView)[0];
    out.scroll = [v0, (view2.vadjustment ?? view2.vscroll.adjustment).value, view === view2];
    await shot('agenda');
    area('agenda').menu.close();
    area('kinds').menu.open();
    await sleep(1000);
    const a = texts(area('kinds').menu.actor);
    await sleep(3000);
    out.kinds = [a, texts(area('kinds').menu.actor)];
    await shot('kinds');
    out.open = press(area('kinds').menu.actor, 'Open');
    out.ghost = texts(area('ghost').menu.actor);
    out.cycle = {};
    for (const k of ['dashboard', 'tasks', 'agenda', 'steps', 'kinds', 'ghost']) {
        EM.disableExtension(id(k));
        await sleep(700);
        const off = area(k) === undefined;
        EM.enableExtension(id(k));
        await sleep(1200);
        out.cycle[k] = [off, EM.lookup(id(k)).state, area(k) !== undefined && area(k) !== null];
    }
    await sleep(3000);
}
export default class P extends Extension {
    enable() {
        GLib.timeout_add(GLib.PRIORITY_DEFAULT, 8000, () => {
            run().catch(e => { out.error = `${e} ${e.stack}`; }).finally(() => GLib.file_set_contents('/tmp/probe/out.json', JSON.stringify(out, null, 1)));
            return GLib.SOURCE_REMOVE;
        });
    }
    disable() {}
}
