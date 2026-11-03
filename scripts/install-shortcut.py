"""Install the current checkout's focus bar in GNOME's custom shortcuts."""

from pathlib import Path
import gi

gi.require_version("Gio", "2.0")
from gi.repository import Gio

root = Path(__file__).resolve().parent.parent
binary = root / "target/debug/da-workstation"
if not binary.is_file():
    raise SystemExit("Build first: cargo build --locked --offline")
desktop = Path.home() / ".local/share/applications/dev.da.Workstation.desktop"
content = (
    "[Desktop Entry]\nType=Application\nName=DA Workstation\n"
    f'Exec="{binary}"\nTerminal=false\nStartupWMClass=dev.da.Workstation\n'
)
if desktop.exists() and desktop.read_text() != content:
    raise SystemExit(f"Refusing to replace a different desktop entry: {desktop}")
desktop.parent.mkdir(parents=True, exist_ok=True)
desktop.write_text(content)
settings = Gio.Settings.new("org.gnome.settings-daemon.plugins.media-keys")
base = "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/"
shortcuts = [
    # The sidebar is the whole app: Ctrl+Alt+Space opens it with search focused.
    ("da-workstation", "DA Workstation Search", "--sidebar --search", "<Control><Alt>space"),
    ("da-workstation-sidebar", "DA Workstation Sidebar", "--sidebar", "<Control><Alt>b"),
    # Desktop-wide tiling; plain Ctrl+T belongs to apps (new tab).
    ("da-workstation-tile", "DA Workstation Tile", "--tile", "<Control><Super>t"),
    ("da-workstation-restore", "DA Workstation Restore Layout", "--restore-layout", "<Control><Super>r"),
] + [
    (f"da-workstation-focus-{d}", f"DA Workstation Focus {d.title()}", f"--focus {d}", f"<Control><Super>{d.title()}")
    for d in ("left", "right", "up", "down")
]
paths = settings.get_strv("custom-keybindings")
for key, name, args, binding in shortcuts:
    path = base + key + "/"
    shortcut = Gio.Settings.new_with_path(
        "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding", path
    )
    shortcut.set_string("name", name)
    shortcut.set_string("command", f'"{binary}" {args}')
    shortcut.set_string("binding", binding)
    if path not in paths:
        paths.append(path)
settings.set_strv("custom-keybindings", paths)
Gio.Settings.sync()
print("Installed Ctrl+Alt+Space, Ctrl+Alt+B, Ctrl+Super+T/R and Ctrl+Super+arrows. Existing shortcuts preserved.")
