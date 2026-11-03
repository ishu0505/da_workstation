"""Verify window focus in a disposable GNOME bus and headless compositor."""

import json
import ast
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time

if '--inside' not in sys.argv:
    # Keep GNOME's background-service diagnostics out of the test result stream.
    bus_log = Path(tempfile.mkdtemp(prefix='da-focus-bus-')) / 'dbus.log'
    child_env = os.environ.copy()
    child_env['DA_FOCUS_PARENT_BUS'] = os.environ.get('DBUS_SESSION_BUS_ADDRESS', '')
    with bus_log.open('w') as output:
        result = subprocess.run(
            ['dbus-run-session', '--', sys.executable, str(Path(__file__).resolve()), '--inside'],
            env=child_env, stderr=output,
        )
    print('D-Bus diagnostics: ' + str(bus_log))
    sys.exit(result.returncode)

if os.environ.get('DBUS_SESSION_BUS_ADDRESS', '') == os.environ.get('DA_FOCUS_PARENT_BUS', ''):
    raise RuntimeError('Refusing to test on the parent desktop bus')

root = Path(__file__).resolve().parents[2]
binary = str(root / 'target/debug/da-workstation')
work = Path(tempfile.mkdtemp(prefix='da-focus-check-'))
env = os.environ.copy()
env.update(XDG_DATA_HOME=str(work / 'data'), XDG_CONFIG_HOME=str(work / 'config'),
           XDG_STATE_HOME=str(work / 'state'),  # never the user's own layout or settings
           GSETTINGS_BACKEND='keyfile', WAYLAND_DISPLAY='da-focus-check-' + str(os.getpid()),
           GDK_BACKEND='wayland', GSK_RENDERER='cairo', LIBGL_ALWAYS_SOFTWARE='1')
env.pop('DISPLAY', None)
extdir = work / 'data/gnome-shell/extensions/da-workstation@local'
shutil.copytree(root / 'gnome-extension', extdir)
appdir = work / 'data/applications'
appdir.mkdir(parents=True)
fixture = work / 'fixture.js'
fixture.write_text('''import Gtk from 'gi://Gtk?version=4.0';
const app = new Gtk.Application({application_id: ARGV[0]});
app.connect('activate', () => {
    const window = new Gtk.ApplicationWindow({application: app, title: ARGV[0], default_width: 400, default_height: 300});
    window.present();
});
app.run([]);
''')
ids = ['dev.da.FocusFixtureA', 'dev.da.FocusFixtureB']
extra = ['dev.da.FocusFixtureC', 'dev.da.FocusFixtureD']
late = 'dev.da.FocusFixtureE'
for appid in ids + extra + [late]:
    (appdir / (appid + '.desktop')).write_text('[Desktop Entry]\nType=Application\nName=Focus fixture\nExec=/usr/bin/gjs -m ' + str(fixture) + ' ' + appid + '\n')

def run(args):
    with tempfile.TemporaryFile(mode='w+') as out, tempfile.TemporaryFile(mode='w+') as err:
        result = subprocess.run(args, env=env, stdout=out, stderr=err, text=True, timeout=10)
        out.seek(0)
        err.seek(0)
        result.stdout, result.stderr = out.read(), err.read()
        return result

def wait_for(args, text, returncode=0):
    last = None
    for _ in range(60):
        last = run(args)
        if last.returncode == returncode and text in last.stdout + last.stderr:
            return last.stdout.strip()
        if shell.poll() is not None:
            raise RuntimeError('Isolated GNOME exited: ' + str(shell.returncode))
        time.sleep(0.5)
    raise RuntimeError('Timed out: ' + repr(last))

children = []
log = (work / 'shell.log').open('w')
try:
    settings = run(['gsettings', 'set', 'org.gnome.shell', 'enabled-extensions', "['da-workstation@local']"])
    assert settings.returncode == 0, settings.stderr
    shell = subprocess.Popen(['gnome-shell', '--headless', '--no-x11', '--virtual-monitor', '1280x800', '--wayland-display', env['WAYLAND_DISPLAY']], env=env, stdout=log, stderr=log)
    children.append(shell)
    wait_for(['gdbus', 'introspect', '--session', '--dest', 'org.gnome.Shell', '--object-path', '/dev/da/Workstation/Windows'], 'dev.da.Workstation.Windows')
    print('PASS: real GNOME 50 loaded the extension and exported the interface', flush=True)
    # Auto-tiling is on by default; these checks open windows one by one, so turn it
    # off here and test it on purpose further down.
    assert 'false' in run(['gdbus', 'call', '--session', '--dest', 'org.gnome.Shell',
                           '--object-path', '/dev/da/Workstation/Windows',
                           '--method', 'dev.da.Workstation.Windows.SetAutoStage', 'false']).stdout
    for appid in ids:
        result = run([binary, '--activate', appid + '.desktop'])
        assert result.returncode == 0 and 'Launch requested' in result.stdout, result
        wait_for([binary, '--app-state', appid + '.desktop'], 'windows=1')
        print('PASS: no window -> launch for ' + appid, flush=True)
    def layout_state():
        result = run(['gdbus', 'call', '--session', '--dest', 'org.gnome.Shell',
                      '--object-path', '/dev/da/Workstation/Windows',
                      '--method', 'dev.da.Workstation.Windows.GetLayoutState'])
        assert result.returncode == 0, result
        return json.loads(ast.literal_eval(result.stdout)[0])
    def check_tiled(state, count):
        # Stage (D-013): every tiled window has the same big frame inside the area;
        # the others show as cards beside it, in the column, without overlap.
        area = state['area']
        rects = [w['rect'] for w in state['windows']]
        assert len(rects) == count, state
        big = rects[0]
        for r in rects:
            assert r == big, state
        assert big['x'] >= area['x'] and big['x'] + big['width'] <= area['x'] + area['width'], state
        assert big['y'] >= area['y'] and big['y'] + big['height'] <= area['y'] + area['height'], state
        cards = [c['rect'] for c in state['cards']]
        assert len(cards) == count - 1 and state['big'] is not None, state
        for i, c in enumerate(cards):
            assert c['x'] >= big['x'] + big['width'], state
            assert c['x'] + c['width'] <= area['x'] + area['width'], state
            for d in cards[i + 1:]:
                assert c['y'] + c['height'] <= d['y'] or d['y'] + d['height'] <= c['y'], state
    before = layout_state()
    result = run([binary, '--tile'])
    assert result.returncode == 0 and '2 windows' in result.stdout, result
    time.sleep(1)
    state = layout_state()
    area = state['area']
    rects = [w['rect'] for w in state['windows']]
    assert len(rects) == 2, state
    for r in rects:
        assert r['x'] >= area['x'] and r['y'] >= area['y'], state
        assert r['x'] + r['width'] <= area['x'] + area['width'], state
        assert r['y'] + r['height'] <= area['y'] + area['height'], state
    check_tiled(state, 2)
    print('PASS: two windows tile as a big window and a card', flush=True)
    assert state['glows'] == 2, state
    print('PASS: tiled windows get glow frames', flush=True)
    result = run([binary, '--restore-layout'])
    assert result.returncode == 0 and '2 windows' in result.stdout, result
    time.sleep(1)
    after = layout_state()
    assert sorted(before['windows'], key=lambda w: w['id']) == sorted(after['windows'], key=lambda w: w['id']), (before, after)
    print('PASS: original window rectangles restored', flush=True)
    assert after['glows'] == 0, after
    print('PASS: restore removes glow frames', flush=True)
    def call(method, *args):
        result = run(['gdbus', 'call', '--session', '--dest', 'org.gnome.Shell',
                      '--object-path', '/dev/da/Workstation/Windows',
                      '--method', 'dev.da.Workstation.Windows.' + method, *args])
        assert result.returncode == 0, result
        return result.stdout
    def window_list():
        return json.loads(ast.literal_eval(call('ListWindows'))[0])
    for appid in extra:
        result = run([binary, '--activate', appid + '.desktop'])
        assert result.returncode == 0 and 'Launch requested' in result.stdout, result
        wait_for([binary, '--app-state', appid + '.desktop'], 'windows=1')
    time.sleep(1)
    before4 = layout_state()
    assert len(before4['windows']) == 4, before4
    result = run([binary, '--tile'])
    assert result.returncode == 0 and '4 windows' in result.stdout, result
    time.sleep(1)
    check_tiled(layout_state(), 4)
    print('PASS: four windows tile as one big window and three cards', flush=True)
    master = min(layout_state()['windows'], key=lambda w: w['rect']['x'])['id']
    call('FocusWindow', str(master))
    time.sleep(0.5)
    focused = [w['id'] for w in window_list() if w['focused']]
    assert focused == [master], (master, focused)
    result = run([binary, '--focus', 'right'])
    assert result.returncode == 0 and 'Focus requested' in result.stdout, result
    time.sleep(0.5)
    moved = [w['id'] for w in window_list() if w['focused']]
    assert moved and moved != focused, (focused, moved)
    result = run([binary, '--focus', 'left'])
    assert result.returncode == 0 and 'Focus requested' in result.stdout, result
    time.sleep(0.5)
    assert [w['id'] for w in window_list() if w['focused']] == [master], window_list()
    print('PASS: directional focus moves between tiled windows', flush=True)
    places = {w['id']: w['rect'] for w in layout_state()['windows']}
    for target in [c['id'] for c in layout_state()['cards']]:
        call('FocusWindow', str(target))
        time.sleep(0.8)
        state = layout_state()
        assert state['big'] == target, (target, state)
        assert target not in [c['id'] for c in state['cards']], state
        assert {w['id']: w['rect'] for w in state['windows']} == places, state
        check_tiled(state, 4)
    print('PASS: clicking or focusing a card makes it the big window; no window is resized', flush=True)
    result = run([binary, '--restore-layout'])
    assert result.returncode == 0 and '4 windows' in result.stdout, result
    time.sleep(1)
    after4 = layout_state()
    assert sorted(before4['windows'], key=lambda w: w['id']) == sorted(after4['windows'], key=lambda w: w['id']), (before4, after4)
    result = run([binary, '--tile', '2'])
    assert result.returncode == 0 and '2 windows' in result.stdout, result
    time.sleep(1)
    check_tiled(layout_state(), 2)
    assert sum(w['minimized'] for w in window_list()) == 2, window_list()
    print('PASS: --tile 2 shows the two most recent windows and minimizes the rest', flush=True)
    result = run([binary, '--tile'])
    assert result.returncode == 1 and 'restore' in result.stderr, result
    for action in ('disable', 'enable'):
        result = run(['gnome-extensions', action, 'da-workstation@local'])
        assert result.returncode == 0, result
        time.sleep(1)
    wait_for(['gdbus', 'introspect', '--session', '--dest', 'org.gnome.Shell', '--object-path', '/dev/da/Workstation/Windows'], 'TileRecent')
    result = run([binary, '--restore-layout'])
    assert result.returncode == 0 and '4 windows' in result.stdout, result
    time.sleep(1)
    assert not any(w['minimized'] for w in window_list()), window_list()
    after_reload = layout_state()
    assert sorted(before4['windows'], key=lambda w: w['id']) == sorted(after_reload['windows'], key=lambda w: w['id']), (before4, after_reload)
    print('PASS: restore after extension reload un-minimizes and restores rectangles', flush=True)
    result = run([binary, '--restore-layout'])
    assert result.returncode == 0 and '0 windows' in result.stdout, result
    assert 'false' in call('PlaceSidebar', 'false')
    listed = window_list()
    assert len(listed) == 4 and {'id', 'title', 'app', 'appId', 'workspace', 'focused', 'minimized', 'attention'} <= set(listed[0]), listed
    print('PASS: ListWindows reports all four windows; PlaceSidebar reports no sidebar', flush=True)
    # Opening an app from DA Workstation tiles it: StageNext, then launch.
    assert 'true' in call('StageNext', 'true')
    result = run([binary, '--launch', late + '.desktop'])
    assert result.returncode == 0, result
    wait_for([binary, '--app-state', late + '.desktop'], 'windows=1')
    time.sleep(1.5)
    staged = layout_state()
    new_id = next(w['id'] for w in window_list() if w['appId'] == late + '.desktop')
    assert staged['big'] == new_id and len(staged['cards']) == 4, staged
    check_tiled(staged, 5)
    result = run([binary, '--restore-layout'])
    assert result.returncode == 0 and '5 windows' in result.stdout, result
    time.sleep(1)
    print('PASS: an app opened through DA Workstation is tiled as the big window', flush=True)
    # Auto-tiling on: an app opened any other way (here the plain launch path, as the
    # GNOME menu does) also joins the stage as the big window.
    assert 'true' in call('SetAutoStage', 'true')
    result = run([binary, '--tile', '2'])
    assert result.returncode == 0, result
    time.sleep(1)
    subprocess.run(['gdbus', 'call', '--session', '--dest', 'org.gnome.Shell', '--object-path',
                    '/dev/da/Workstation/Windows', '--method', 'dev.da.Workstation.Windows.FocusApp',
                    ids[0] + '.desktop'], env=env, capture_output=True, timeout=10)
    time.sleep(1)
    state = layout_state()
    assert state['autoStage'] is True and state['big'] is not None, state
    late_id = new_id
    subprocess.run(['pkill', '-f', 'FocusFixtureE'], env=env)
    wait_for([binary, '--app-state', late + '.desktop'], 'windows=0')
    subprocess.Popen(['gjs', '-m', str(fixture), late], env=dict(env, GDK_BACKEND='wayland'),
                     stdout=log, stderr=log)
    wait_for([binary, '--app-state', late + '.desktop'], 'windows=1')
    time.sleep(1.5)
    staged = layout_state()
    newest = next(w['id'] for w in window_list() if w['appId'] == late + '.desktop')
    assert staged['big'] == newest and len(staged['cards']) >= 2, staged
    result = run([binary, '--restore-layout'])
    assert result.returncode == 0, result
    assert 'false' in call('SetAutoStage', 'false')
    time.sleep(1)
    print('PASS: with auto-tiling on, an app opened outside DA Workstation joins the stage', flush=True)
    sidebar_env = dict(env, XDG_DATA_HOME=str(work / 'sidebar-data'))
    sidebar = subprocess.Popen([binary, '--sidebar'], env=sidebar_env, stdout=log, stderr=log)
    children.append(sidebar)
    wait_for(['gdbus', 'call', '--session', '--dest', 'org.gnome.Shell', '--object-path', '/dev/da/Workstation/Windows',
              '--method', 'dev.da.Workstation.Windows.PlaceSidebar', 'false'], 'true')
    time.sleep(1)
    state = layout_state()
    assert state['area']['x'] >= 380, state
    result = run([binary, '--tile', '2'])
    assert result.returncode == 0 and '2 windows' in result.stdout, result
    time.sleep(1.5)
    tiled = layout_state()
    check_tiled(tiled, 2)
    assert all(w['rect']['x'] >= 380 for w in tiled['windows']), tiled
    result = run([binary, '--restore-layout'])
    assert result.returncode == 0, result
    print('PASS: sidebar opens, docks left, and tiling leaves its width free', flush=True)
    result = run([binary, '--tile', '3'])
    assert result.returncode == 0 and '3 windows' in result.stdout, result
    time.sleep(1.5)
    tiled = layout_state()
    check_tiled(tiled, 3)
    assert all(w['rect']['x'] >= 380 for w in tiled['windows']), tiled
    result = run([binary, '--restore-layout'])
    assert result.returncode == 0, result
    print('PASS: a big window and two cards fit beside the docked sidebar', flush=True)
    placement = work / 'sidebar-data' / 'da-workstation' / 'placement'
    placement.parent.mkdir(parents=True, exist_ok=True)
    placement.write_text('bottom')
    # The CLI below runs with the test's own data folder; it must agree.
    cli_placement = work / 'data' / 'da-workstation' / 'placement'
    cli_placement.parent.mkdir(parents=True, exist_ok=True)
    cli_placement.write_text('bottom')
    result = subprocess.run([binary, '--sidebar'], env=sidebar_env, capture_output=True, text=True, timeout=10)
    assert 'raised' in result.stdout, result
    time.sleep(1)
    state = layout_state()
    assert state['area']['x'] == 0 and state['area']['y'] + state['area']['height'] <= 800 - 300 + 1, state
    result = run([binary, '--tile', '2'])
    assert result.returncode == 0 and '2 windows' in result.stdout, result
    time.sleep(1.5)
    tiled = layout_state()
    check_tiled(tiled, 2)
    result = run([binary, '--restore-layout'])
    assert result.returncode == 0, result
    placement.write_text('left')
    cli_placement.write_text('left')
    subprocess.run([binary, '--sidebar'], env=sidebar_env, capture_output=True, text=True, timeout=10)
    time.sleep(1)
    assert layout_state()['area']['x'] >= 380
    print('PASS: bottom-bar mode docks along the bottom and tiles above it; left mode returns', flush=True)
    # Startup activation is compositor policy. Explicitly focus B before testing A.
    result = run([binary, '--activate', ids[1] + '.desktop'])
    assert result.returncode == 0 and 'Focus requested' in result.stdout, result
    wait_for([binary, '--app-state', ids[1] + '.desktop'], 'windows=1 focused=true')
    result = run([binary, '--activate', ids[0] + '.desktop'])
    assert result.returncode == 0 and 'Focus requested' in result.stdout, result
    state = wait_for([binary, '--app-state', ids[0] + '.desktop'], 'windows=1 focused=true')
    print('PASS: existing app focused with one window: ' + state, flush=True)
    result = run([binary, '--activate', ids[0] + '.desktop'])
    assert result.returncode == 0 and 'Focus requested' in result.stdout, result
    state = wait_for([binary, '--app-state', ids[0] + '.desktop'], 'windows=1 focused=true')
    print('PASS: repeated activation keeps one window', flush=True)
    result = run(['gnome-extensions', 'disable', 'da-workstation@local'])
    assert result.returncode == 0, result
    wait_for([binary, '--activate', ids[0] + '.desktop'], 'Window control unavailable', returncode=1)
    result = run(['gnome-extensions', 'enable', 'da-workstation@local'])
    assert result.returncode == 0, result
    wait_for([binary, '--app-state', ids[0] + '.desktop'], 'windows=1')
    print('PASS: disabled extension reports an error and does not create a duplicate', flush=True)
finally:
    for child in reversed(children):
        child.terminate()
        try:
            child.wait(timeout=5)
        except subprocess.TimeoutExpired:
            child.kill()
            child.wait()
    log.close()
    print('GNOME log: ' + str(work / 'shell.log'), flush=True)
