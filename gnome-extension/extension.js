import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Meta from 'gi://Meta';
import Shell from 'gi://Shell';
import St from 'gi://St';
import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import Clutter from 'gi://Clutter';
import {besideSidebar, nearest, stage} from './layout.js';

const INTERFACE = `<node>
  <interface name="dev.da.Workstation.Windows">
    <method name="GetLayoutState"><arg name="state" type="s" direction="out"/></method>
    <method name="TileWorkspace"><arg name="count" type="u" direction="out"/></method>
    <method name="TileRecent">
      <arg name="limit" type="u" direction="in"/>
      <arg name="count" type="u" direction="out"/>
    </method>
    <method name="RestoreLayout"><arg name="count" type="u" direction="out"/></method>
    <method name="FocusDirection">
      <arg name="direction" type="s" direction="in"/>
      <arg name="focused" type="b" direction="out"/>
    </method>
    <method name="PlaceSidebar">
      <arg name="activate" type="b" direction="in"/>
      <arg name="found" type="b" direction="out"/>
    </method>
    <method name="SetAutoStage">
      <arg name="on" type="b" direction="in"/>
      <arg name="on" type="b" direction="out"/>
    </method>
    <method name="StageNext">
      <arg name="arm" type="b" direction="in"/>
      <arg name="armed" type="b" direction="out"/>
    </method>
    <method name="PlaceSidebarAt">
      <arg name="placement" type="s" direction="in"/>
      <arg name="activate" type="b" direction="in"/>
      <arg name="found" type="b" direction="out"/>
    </method>
    <method name="ListWindows"><arg name="windows" type="s" direction="out"/></method>
    <method name="FocusWindow">
      <arg name="id" type="t" direction="in"/>
      <arg name="focused" type="b" direction="out"/>
    </method>
    <method name="FocusApp">
      <arg name="desktopId" type="s" direction="in"/>
      <arg name="requested" type="b" direction="out"/>
    </method>
    <method name="GetAppState">
      <arg name="desktopId" type="s" direction="in"/>
      <arg name="windowCount" type="u" direction="out"/>
      <arg name="focused" type="b" direction="out"/>
    </method>
  </interface>
</node>`;

const OWN_CLASS = 'dev.da.Workstation';
const SIDEBAR_CLASS = 'dev.da.Workstation.Sidebar';
const SIDEBAR_WIDTH = 380;
const BAR_HEIGHT = 300;
// Glow frames around tiled windows: bright on the focused one, faint on the rest.
// Glow: four thin bars around the window, so nothing paints over its middle or over
// windows tucked beneath it. Bright with a halo when focused, a faint line otherwise.
const GLOW_PAD = 4;
const GLOW_BAR = 3;
const GLOW_FOCUSED = 'background-color: rgba(56,189,248,1); border-radius: 2px; box-shadow: 0 0 18px 4px rgba(56,189,248,0.7);';
const GLOW_IDLE = 'background-color: rgba(56,189,248,0.45); border-radius: 2px;';
const rect = r => ({x: r.x, y: r.y, width: r.width, height: r.height});
const isOwn = w => (w.get_wm_class() ?? '').startsWith(OWN_CLASS);

export default class WorkstationWindows extends Extension {
    _windows(desktopId) {
        const app = Shell.AppSystem.get_default().lookup_app(desktopId);
        if (!app)
            throw new Error('Application is not installed');
        // Shell orders windows by workspace, visibility, and recent interaction.
        return app.get_windows();
    }

    FocusApp(desktopId) {
        const windows = this._windows(desktopId);
        if (windows.length === 0)
            return false;
        Main.activateWindow(windows[0]);
        return true;
    }

    GetAppState(desktopId) {
        const windows = this._windows(desktopId);
        return [windows.length, windows.some(window => window.has_focus())];
    }

    // Normal windows on the active workspace and current monitor, most recently used first.
    _place() {
        const workspace = global.workspace_manager.get_active_workspace();
        const monitor = global.display.get_current_monitor();
        const all = global.display.get_tab_list(Meta.TabList.NORMAL_ALL, workspace)
            .filter(w => w.get_monitor() === monitor && w.get_window_type() === Meta.WindowType.NORMAL);
        const sidebar = all.find(w => w.get_wm_class() === SIDEBAR_CLASS && !w.minimized);
        const area = besideSidebar(rect(workspace.get_work_area_for_monitor(monitor)),
            sidebar && rect(sidebar.get_frame_rect()));
        return {area, windows: all.filter(w => !isOwn(w) && !w.minimized)};
    }

    GetLayoutState() {
        const {area, windows} = this._place();
        const tiled = this._tiled;
        return JSON.stringify({area, glows: this._glows?.size ?? 0,
            stack: global.display.sort_windows_by_stacking(windows).map(w => w.get_id()),
            big: tiled?.big.get_id() ?? null,
            autoStage: this._autoStage,
            cards: tiled ? tiled.windows.filter(w => w !== tiled.big)
                .map((w, i) => ({id: w.get_id(), rect: tiled.layout.cards[i]})) : [],
            windows: windows.map(w => ({id: w.get_id(), rect: rect(w.get_frame_rect())}))});
    }

    TileWorkspace() {
        return this.TileRecent(0);
    }

    // Tile the `limit` most recent windows (0: all). Minimize the others; restore brings them back.
    TileRecent(limit) {
        if (this._snapshot().length)
            throw new Error('Press Ctrl+R to restore the previous layout before tiling again');
        const {area, windows: all} = this._place();
        const windows = limit > 0 ? all.slice(0, limit) : all;
        const hidden = all.slice(windows.length);
        if (windows.length < 2)
            throw new Error(`Open at least two normal windows on this monitor and workspace (found ${all.length})`);
        if (windows.some(w => w.is_fullscreen() || !w.allows_resize()))
            throw new Error('Exit fullscreen and use resizable windows before tiling');
        // Check the size first so a refusal changes nothing.
        const layout = stage(area, windows.length);
        this._layout = [...windows, ...hidden].map((w, i) => ({w, id: w.get_id(),
            rect: rect(w.get_frame_rect()), maximized: w.get_maximize_flags(),
            tiled: i < windows.length}));
        this._save();
        windows.forEach(w => w.unmaximize());
        hidden.forEach(w => w.minimize());
        windows.forEach(w => this._glow(w));
        // Every window gets the big size; cards are the same windows drawn small.
        const {big} = layout;
        windows.forEach(w => w.move_resize_frame(false, big.x, big.y, big.width, big.height));
        this._tiled = {area, windows, big: windows[0], layout, cards: []};
        // Let the windows take their new size before measuring them for the cards.
        GLib.timeout_add(GLib.PRIORITY_DEFAULT, 150, () => {
            this._arrange(true);
            return GLib.SOURCE_REMOVE;
        });
        return windows.length;
    }

    RestoreLayout() {
        this._unstage();
        this._clearGlows();
        let count = 0;
        const live = new Set(global.get_window_actors().map(actor => actor.meta_window));
        for (const {w, rect: r, maximized, tiled} of this._snapshot()) {
            if (!live.has(w))
                continue;
            if (w.minimized)
                w.unminimize();
            if (tiled) {
                w.unmaximize();
                w.move_resize_frame(false, r.x, r.y, r.width, r.height);
                this._maximize(w, maximized);
            }
            count++;
        }
        this._layout = null;
        this._save();
        return count;
    }

    // The next normal window that opens (within 20 s) joins the stage as the big
    // window, or stages everything when nothing is tiled. DA Workstation calls this
    // right before it launches an app.
    StageNext(arm) {
        this._stageNext = arm ? GLib.get_monotonic_time() + 20 * 1000000 : 0;
        return arm;
    }

    // Auto-tiling (default on): every new normal window joins the stage. Off: only
    // windows DA Workstation opens (StageNext). The choice survives restarts.
    SetAutoStage(on) {
        this._autoStage = on;
        try {
            const path = GLib.build_filenamev([GLib.get_user_state_dir(), 'da-workstation', 'autostage']);
            GLib.mkdir_with_parents(GLib.path_get_dirname(path), 0o700);
            GLib.file_set_contents(path, on ? 'on' : 'off');
        } catch (error) {
            console.warn(`DA Workstation: auto-tiling setting not saved: ${error.message}`);
        }
        return on;
    }

    _loadAutoStage() {
        try {
            const path = GLib.build_filenamev([GLib.get_user_state_dir(), 'da-workstation', 'autostage']);
            const [, bytes] = GLib.file_get_contents(path);
            return new TextDecoder().decode(bytes).trim() !== 'off';
        } catch {
            return true;
        }
    }

    _windowCreated(w) {
        const armed = this._stageNext && GLib.get_monotonic_time() <= this._stageNext;
        if (!armed && !this._autoStage)
            return;
        if (w.get_window_type() !== Meta.WindowType.NORMAL || isOwn(w) || w.get_transient_for())
            return;
        this._stageNext = 0;
        const actor = w.get_compositor_private();
        const go = () => {
            const tiled = this._tiled;
            // Only windows on the workspace being looked at.
            if (w.get_workspace() !== global.workspace_manager.get_active_workspace())
                return;
            if (tiled) {
                this._join(w);
            } else {
                try {
                    if (this._snapshot().length)
                        this.RestoreLayout();
                    this.TileRecent(0);
                } catch (error) {
                    console.warn(`DA Workstation: could not stage the new window: ${error.message}`);
                }
            }
        };
        // Wait for the first frame: the window has its size and is on screen.
        if (actor) {
            const id = actor.connect('first-frame', () => {
                actor.disconnect(id);
                go();
            });
        } else {
            GLib.timeout_add(GLib.PRIORITY_DEFAULT, 300, () => {
                go();
                return GLib.SOURCE_REMOVE;
            });
        }
    }

    // Add a window to the stage as the big one; Ctrl+Super+R puts it back too.
    _join(w) {
        const tiled = this._tiled;
        if (!tiled || tiled.windows.includes(w) || w.is_fullscreen() || !w.allows_resize())
            return;
        this._layout = [...(this._snapshot() ?? []), {w, id: w.get_id(),
            rect: rect(w.get_frame_rect()), maximized: w.get_maximize_flags(), tiled: true}];
        this._save();
        w.unmaximize();
        const {big} = tiled.layout;
        w.move_resize_frame(false, big.x, big.y, big.width, big.height);
        this._glow(w);
        tiled.windows.unshift(w);
        tiled.big = w;
        GLib.timeout_add(GLib.PRIORITY_DEFAULT, 150, () => {
            this._arrange(true);
            return GLib.SOURCE_REMOVE;
        });
    }

    // Show the big window at full size and the others as cards on the right.
    _arrange(animate) {
        const tiled = this._tiled;
        if (!tiled)
            return;
        tiled.windows = tiled.windows.filter(w => this._glows.has(w) && !w.minimized);
        if (tiled.windows.length < 2) {
            // Too few windows left: end tiling properly, so the next tile is not
            // blocked by a leftover restore point.
            this.RestoreLayout();
            return;
        }
        if (!tiled.windows.includes(tiled.big))
            tiled.big = tiled.windows[0];
        tiled.layout = stage(tiled.area, tiled.windows.length);
        const {cards, scale} = tiled.layout;
        tiled.cards.forEach(card => card.destroy());
        tiled.cards = [];
        const move = (actor, props) => animate
            ? actor.ease({...props, duration: 220, mode: Clutter.AnimationMode.EASE_OUT_QUAD})
            : actor.set(props);
        let k = 0;
        for (const w of tiled.windows) {
            const actor = w.get_compositor_private();
            if (!actor)
                continue;
            actor.set_pivot_point(0, 0);
            if (w === tiled.big) {
                move(actor, {scale_x: 1, scale_y: 1, translation_x: 0, translation_y: 0});
                w.raise();
                continue;
            }
            // The actor includes client-side shadows; place the frame itself on the card.
            const card = cards[k++];
            const frame = w.get_frame_rect();
            const buffer = w.get_buffer_rect();
            move(actor, {scale_x: scale, scale_y: scale,
                translation_x: card.x - buffer.x - (frame.x - buffer.x) * scale,
                translation_y: card.y - buffer.y - (frame.y - buffer.y) * scale});
            tiled.cards.push(this._card(w, card));
        }
        this._showCards();
        this._styleGlows();
    }

    // A click target over a card: the app never receives clicks meant for the card.
    _card(w, r) {
        const style = hover => `border-radius: 10px; border: ${hover ? 2 : 1}px solid ` +
            `rgba(56,189,248,${hover ? 0.9 : 0.35}); background-color: rgba(0,0,0,${hover ? 0 : 0.08});`;
        const card = new St.Button({reactive: true, track_hover: true, can_focus: false,
            style: style(false), x: r.x, y: r.y, width: r.width, height: r.height});
        // Two lines: the app, then what it shows (for a browser, the page title).
        const app = Shell.WindowTracker.get_default().get_window_app(w)?.get_name() ?? '';
        const name = new St.Label({text: app,
            style: 'font-size: 11px; font-weight: bold; color: white;'});
        const title = new St.Label({text: w.get_title() ?? '',
            style: 'font-size: 10px; color: rgba(255,255,255,0.8);'});
        for (const label of [name, title])
            label.clutter_text.ellipsize = 3; // Pango.EllipsizeMode.END
        const caption = new St.BoxLayout({vertical: true, x_expand: true,
            style: 'background-color: rgba(9,9,11,0.82); padding: 3px 7px; border-radius: 0 0 9px 9px;'});
        caption.add_child(name);
        caption.add_child(title);
        const box = new St.BoxLayout({vertical: true, x_expand: true, y_expand: true});
        box.add_child(new St.Widget({y_expand: true}));
        box.add_child(caption);
        card.set_child(box);
        // Keep the caption current: a browser's title follows its tab.
        const retitle = w.connect('notify::title', () => title.set_text(w.get_title() ?? ''));
        card.connect('destroy', () => {
            try {
                w.disconnect(retitle);
            } catch {} // The window may already be gone.
        });
        card.connect('notify::hover', () => (card.style = style(card.hover)));
        card.connect('clicked', () => Main.activateWindow(w));
        Main.layoutManager.uiGroup.add_child(card);
        return card;
    }

    // Cards show only on the tiled workspace and outside the Activities overview.
    _showCards() {
        const tiled = this._tiled;
        if (!tiled)
            return;
        const here = tiled.big.get_workspace() === global.workspace_manager.get_active_workspace();
        const show = here && !Main.overview.visible;
        tiled.cards.forEach(card => (card.visible = show));
    }

    // Put every window back to normal drawing and remove the cards.
    _unstage() {
        const tiled = this._tiled;
        this._tiled = null;
        if (!tiled)
            return;
        tiled.cards.forEach(card => card.destroy());
        for (const w of tiled.windows) {
            const actor = w.get_compositor_private();
            if (actor) {
                actor.remove_all_transitions();
                actor.set({scale_x: 1, scale_y: 1, translation_x: 0, translation_y: 0});
            }
        }
    }

    // A frame just behind the window that follows it and shows only while it shows.
    _glow(w) {
        const actor = w.get_compositor_private();
        if (!actor || this._glows.has(w))
            return;
        const frame = new St.Widget({reactive: false});
        const bars = [0, 1, 2, 3].map(() => new St.Widget({reactive: false}));
        bars.forEach(bar => frame.add_child(bar));
        frame.bars = bars;
        global.window_group.add_child(frame);
        // GNOME destroys the actor before the window is unmanaged; never touch it after.
        let dead = false;
        const destroyed = actor.connect('destroy', () => {
            dead = true;
            this._unglow(w);
        });
        const place = () => {
            if (dead)
                return;
            const r = w.get_frame_rect();
            const width = r.width + GLOW_PAD * 2;
            const height = r.height + GLOW_PAD * 2;
            frame.set_position(r.x - GLOW_PAD, r.y - GLOW_PAD);
            frame.set_size(width, height);
            const edges = [[0, 0, width, GLOW_BAR], [0, height - GLOW_BAR, width, GLOW_BAR],
                [0, 0, GLOW_BAR, height], [width - GLOW_BAR, 0, GLOW_BAR, height]];
            bars.forEach((bar, i) => {
                bar.set_position(edges[i][0], edges[i][1]);
                bar.set_size(edges[i][2], edges[i][3]);
            });
            frame.visible = actor.visible && !w.minimized &&
                (!this._tiled || this._tiled.big === w);
            if (actor.get_parent() === frame.get_parent())
                // Below the window: the window hides the frame's middle, so only the
                // rim and its glow show around it.
                global.window_group.set_child_below_sibling(frame, actor);
        };
        const ids = ['position-changed', 'size-changed', 'notify::minimized', 'raised']
            .map(signal => w.connect(signal, place));
        const shown = actor.connect('notify::visible', place);
        const gone = w.connect('unmanaged', () => {
            this._unglow(w);
            this._arrange(false);
        });
        this._glows.set(w, {frame, place, disconnect: () => {
            ids.forEach(id => w.disconnect(id));
            w.disconnect(gone);
            if (!dead) {
                actor.disconnect(shown);
                actor.disconnect(destroyed);
            }
        }});
        place();
        this._styleGlows();
    }

    _unglow(w) {
        const glow = this._glows.get(w);
        if (!glow)
            return;
        glow.disconnect();
        glow.frame.destroy();
        this._glows.delete(w);
    }

    _clearGlows() {
        for (const w of [...this._glows.keys()])
            this._unglow(w);
    }

    _styleGlows() {
        const focus = global.display.focus_window;
        for (const [w, {frame, place}] of this._glows) {
            frame.bars.forEach(bar => (bar.style = w === focus ? GLOW_FOCUSED : GLOW_IDLE));
            place();
        }
    }

    // GNOME 50 (Mutter 18) dropped the flags argument from maximize().
    _maximize(w, flags) {
        const {HORIZONTAL, VERTICAL} = Meta.MaximizeFlags;
        if ((flags & HORIZONTAL) && (flags & VERTICAL))
            w.maximize();
        else if (flags & HORIZONTAL)
            w.maximize_horizontally();
        else if (flags & VERTICAL)
            w.maximize_vertically();
    }

    FocusDirection(direction) {
        const tiled = this._tiled;
        if (tiled && tiled.windows.length > 1) {
            // Staged: left/up go to the previous window, right/down to the next.
            const list = tiled.windows;
            const step = direction === 'left' || direction === 'up' ? -1 : 1;
            const from = Math.max(0, list.indexOf(global.display.focus_window));
            Main.activateWindow(list[(from + step + list.length) % list.length]);
            return true;
        }
        const {windows} = this._place();
        const focus = global.display.focus_window;
        const others = windows.filter(w => w !== focus);
        if (!focus || isOwn(focus) || !windows.includes(focus)) {
            if (!others.length)
                return false;
            Main.activateWindow(others[0]);
            return true;
        }
        const target = nearest(rect(focus.get_frame_rect()),
            others.map(w => ({w, rect: rect(w.get_frame_rect())})), direction);
        if (!target)
            return false;
        Main.activateWindow(target.w);
        return true;
    }

    // Dock the sidebar on the left edge at full work-area height; optionally raise it.
    PlaceSidebar(activate) {
        return this.PlaceSidebarAt('left', activate);
    }

    // Dock the app: 'left' (full height) or 'bottom' (full width, BAR_HEIGHT tall).
    PlaceSidebarAt(placement, activate) {
        const sidebar = global.get_window_actors().map(a => a.meta_window)
            .find(w => w.get_wm_class() === SIDEBAR_CLASS);
        if (!sidebar)
            return false;
        if (sidebar.minimized)
            sidebar.unminimize();
        const area = sidebar.get_work_area_current_monitor();
        sidebar.unmaximize();
        if (placement === 'bottom')
            sidebar.move_resize_frame(false, area.x, area.y + area.height - BAR_HEIGHT, area.width, BAR_HEIGHT);
        else
            sidebar.move_resize_frame(false, area.x, area.y, SIDEBAR_WIDTH, area.height);
        // A docked panel moves the tiling area; arrange the stage again if tiled.
        if (this._tiled)
            this._tiled.area = this._place().area;
        this._arrange?.(false);
        if (activate)
            Main.activateWindow(sidebar);
        return true;
    }

    // Every normal window on every workspace, for the sidebar.
    ListWindows() {
        const tracker = Shell.WindowTracker.get_default();
        const focus = global.display.focus_window;
        const windows = global.display.get_tab_list(Meta.TabList.NORMAL_ALL, null)
            .filter(w => w.get_window_type() === Meta.WindowType.NORMAL && !isOwn(w));
        return JSON.stringify(windows.map(w => ({
            id: w.get_id(),
            title: w.get_title() ?? '',
            app: tracker.get_window_app(w)?.get_name() ?? w.get_wm_class() ?? '',
            appId: tracker.get_window_app(w)?.get_id() ?? '',
            workspace: w.get_workspace()?.index() ?? -1,
            focused: w === focus,
            minimized: w.minimized,
            attention: w.demands_attention || w.urgent,
        })));
    }

    FocusWindow(id) {
        const w = global.get_window_actors().map(a => a.meta_window).find(m => m.get_id() === id);
        if (!w)
            return false;
        Main.activateWindow(w);
        return true;
    }

    // The restore snapshot survives an extension reload in the same GNOME session.
    _statePath() {
        return GLib.build_filenamev([GLib.get_user_state_dir(), 'da-workstation', 'layout.json']);
    }

    _shellPid() {
        return GLib.file_read_link('/proc/self');
    }

    _save() {
        const path = this._statePath();
        try {
            if (!this._layout?.length) {
                Gio.File.new_for_path(path).delete(null);
                return;
            }
            GLib.mkdir_with_parents(GLib.path_get_dirname(path), 0o700);
            GLib.file_set_contents(path, JSON.stringify({shell: this._shellPid(),
                windows: this._layout.map(({id, rect: r, maximized, tiled}) =>
                    ({id, rect: r, maximized, tiled}))}));
        } catch (error) {
            if (!error.matches?.(Gio.IOErrorEnum, Gio.IOErrorEnum.NOT_FOUND))
                console.warn(`DA Workstation: layout snapshot not saved: ${error.message}`);
        }
    }

    _snapshot() {
        if (this._layout)
            return this._layout;
        try {
            const [, bytes] = GLib.file_get_contents(this._statePath());
            const saved = JSON.parse(new TextDecoder().decode(bytes));
            if (saved.shell !== this._shellPid())
                return [];
            const byId = new Map(global.get_window_actors()
                .map(a => [a.meta_window.get_id(), a.meta_window]));
            this._layout = saved.windows.filter(e => byId.has(e.id))
                .map(e => ({...e, w: byId.get(e.id)}));
        } catch {
            this._layout = [];
        }
        return this._layout;
    }

    enable() {
        this._layout = null;
        this._glows = new Map();
        this._tiled = null;

        this._focusId = global.display.connect('notify::focus-window', () => {
            const focus = global.display.focus_window;
            const tiled = this._tiled;
            if (focus && tiled && tiled.big !== focus && tiled.windows.includes(focus)) {
                tiled.big = focus;
                this._arrange(true);
            }
            this._styleGlows();
        });
        // GNOME reorders window actors only; put each frame back under its window.
        this._restackId = global.display.connect('restacked', () => this._styleGlows());
        this._stageNext = 0;
        this._autoStage = this._loadAutoStage();
        this._createdId = global.display.connect('window-created', (_, w) => this._windowCreated(w));
        this._workspaceId = global.workspace_manager.connect('active-workspace-changed',
            () => this._showCards());
        this._overviewIds = ['showing', 'hidden'].map(signal =>
            Main.overview.connect(signal, () => this._showCards()));
        this._dbus = Gio.DBusExportedObject.wrapJSObject(INTERFACE, this);
        this._dbus.export(Gio.DBus.session, '/dev/da/Workstation/Windows');
    }

    disable() {
        this._unstage();
        global.workspace_manager.disconnect(this._workspaceId);
        global.display.disconnect(this._createdId);
        this._overviewIds.forEach(id => Main.overview.disconnect(id));
        this._clearGlows();
        global.display.disconnect(this._focusId);
        global.display.disconnect(this._restackId);
        this._dbus?.unexport();
        this._dbus = null;
        this._layout = null;
    }
}
