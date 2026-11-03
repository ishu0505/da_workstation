mod activity;
mod apps;
mod help;
mod mcp;
mod notify;
mod provider;
mod sidebar;
mod switcher;
mod tasks;
mod text_input;
mod windows;

use apps::DesktopApp;
use gpui::{prelude::*, *};
use text_input::TextInput;

actions!(
    launcher,
    [
        Launch,
        LaunchNew,
        Next,
        Previous,
        Dismiss,
        Clear,
        Tile,
        RestoreLayout,
        OpenSidebar
    ]
);

/// Tasks the bar shows above the apps.
const BAR_TASKS: usize = 3;

struct Launcher {
    input: Entity<TextInput>,
    apps: Vec<DesktopApp>,
    matches: Vec<usize>,
    tasks: Vec<tasks::Task>,
    /// Matching tasks; the list shows these first, then `matches`.
    task_matches: Vec<usize>,
    selected: usize,
    query: String,
    message: String,
    failed: bool,
    launching: bool,
    _input_subscription: Subscription,
}

impl Launcher {
    fn new(apps: Vec<DesktopApp>, cx: &mut Context<Self>) -> Self {
        let input = cx.new(TextInput::new);
        let subscription = cx.observe(&input, |this, _, cx| {
            let query = this.input.read(cx).value().to_string();
            if query != this.query {
                this.matches = apps::matches(&this.apps, &query);
                this.task_matches = tasks::bar_matches(&this.tasks, &query, BAR_TASKS);
                this.selected = 0;
                this.query = query;
                cx.notify();
            }
        });
        let matches = apps::matches(&apps, "");
        // Read only: the sidebar owns runs and their state.
        let tasks = tasks::Store::open()
            .map(|store| store.read_all().0)
            .unwrap_or_default();
        let task_matches = tasks::bar_matches(&tasks, "", BAR_TASKS);
        Self {
            message: format!(
                "{} installed apps · Try terminal, browser, or an app name",
                apps.len()
            ),
            input,
            apps,
            matches,
            tasks,
            task_matches,
            selected: 0,
            query: String::new(),
            failed: false,
            launching: false,
            _input_subscription: subscription,
        }
    }

    fn entries(&self) -> usize {
        self.task_matches.len() + self.matches.len()
    }

    fn next(&mut self, _: &Next, _: &mut Window, cx: &mut Context<Self>) {
        if self.entries() > 0 {
            self.selected = (self.selected + 1) % self.entries();
        }
        cx.notify();
    }

    fn previous(&mut self, _: &Previous, _: &mut Window, cx: &mut Context<Self>) {
        if self.entries() > 0 {
            self.selected = (self.selected + self.entries() - 1) % self.entries();
        }
        cx.notify();
    }

    /// Show a task in the sidebar, starting the sidebar if needed.
    fn open_task(&mut self, index: usize, cx: &mut Context<Self>) {
        let task = &self.tasks[index];
        let result = std::env::current_exe().and_then(|exe| {
            std::process::Command::new(exe)
                .args(["--sidebar", "--open", &task.id])
                .spawn()
        });
        self.failed = result.is_err();
        self.message = match result {
            Ok(_) => format!("Opening task in the sidebar: {}", task.title),
            Err(error) => format!("Could not open the sidebar: {error}"),
        };
        cx.notify();
    }

    fn launch(&mut self, _: &Launch, _: &mut Window, cx: &mut Context<Self>) {
        self.open(false, cx);
    }

    fn launch_new(&mut self, _: &LaunchNew, _: &mut Window, cx: &mut Context<Self>) {
        self.open(true, cx);
    }

    fn open(&mut self, force_launch: bool, cx: &mut Context<Self>) {
        if self.launching {
            return;
        }
        if let Some(&index) = self.task_matches.get(self.selected) {
            self.open_task(index, cx);
            return;
        }
        let Some(&index) = self.matches.get(self.selected - self.task_matches.len()) else {
            return;
        };
        let app = self.apps[index].clone();
        self.launching = true;
        self.failed = false;
        self.message = format!("Opening {}…", app.name);
        cx.notify();
        let task = cx.background_executor().spawn(async move {
            let result = if force_launch {
                apps::launch(&app.id).map(|()| windows::Activation::LaunchRequested)
            } else {
                windows::activate(&app.id)
            };
            (app.name, result)
        });
        cx.spawn(async move |this, cx| {
            let (name, result) = task.await;
            let _ = this.update(cx, |this, cx| {
                this.launching = false;
                this.failed = result.is_err();
                this.message = match result {
                    Ok(windows::Activation::FocusRequested) => format!("Focus requested: {name}."),
                    Ok(windows::Activation::LaunchRequested) => {
                        format!("Launch requested: {name}. Use Alt+Tab to return here.")
                    }
                    Err(error) => format!("Could not open {name}: {error}"),
                };
                cx.notify();
            });
        })
        .detach();
    }

    fn tile(&mut self, _: &Tile, _: &mut Window, cx: &mut Context<Self>) {
        self.change_layout(windows::LayoutRequest::Tile(0), cx);
    }

    fn restore_layout(&mut self, _: &RestoreLayout, _: &mut Window, cx: &mut Context<Self>) {
        self.change_layout(windows::LayoutRequest::Restore, cx);
    }

    /// Ctrl+2 to Ctrl+9 show that many of the most recent windows.
    fn key_down(&mut self, event: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let keystroke = &event.keystroke;
        let modifiers = &keystroke.modifiers;
        if modifiers.control && !modifiers.alt && !modifiers.shift && !modifiers.platform {
            if let Ok(count @ 2..=9) = keystroke.key.parse::<u32>() {
                self.change_layout(windows::LayoutRequest::Tile(count), cx);
                cx.stop_propagation();
            }
        }
    }

    fn change_layout(&mut self, request: windows::LayoutRequest, cx: &mut Context<Self>) {
        let restore = request == windows::LayoutRequest::Restore;
        if self.launching {
            return;
        }
        self.launching = true;
        let task = cx
            .background_executor()
            .spawn(async move { windows::layout(request) });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                this.launching = false;
                this.failed = result.is_err();
                this.message = match result {
                    Ok(0) if restore => "No saved layout to restore. Press Ctrl+T first.".into(),
                    Ok(count) => format!(
                        "{} requested for {count} windows. Check window placement.",
                        if restore { "Restore" } else { "Tiling" }
                    ),
                    Err(error) => error,
                };
                cx.notify();
            });
        })
        .detach();
    }

    fn open_sidebar(&mut self, _: &OpenSidebar, _: &mut Window, cx: &mut Context<Self>) {
        let result = std::env::current_exe()
            .and_then(|exe| std::process::Command::new(exe).arg("--sidebar").spawn())
            .map(|_| ());
        self.failed = result.is_err();
        self.message = match result {
            Ok(()) => "Sidebar requested.".into(),
            Err(error) => format!("Could not open the sidebar: {error}"),
        };
        cx.notify();
    }

    fn clear(&mut self, _: &Clear, _: &mut Window, cx: &mut Context<Self>) {
        self.input.update(cx, |input, cx| {
            input.reset();
            cx.notify();
        });
    }

    fn dismiss(&mut self, _: &Dismiss, _: &mut Window, cx: &mut Context<Self>) {
        cx.quit();
    }
}

impl Render for Launcher {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let start = self.selected.saturating_sub(4);
        div()
            .key_context("Launcher")
            .size_full()
            .flex()
            .flex_col()
            .gap_3()
            .p_5()
            .bg(rgb(0x09090b))
            .text_color(rgb(0xfafafa))
            .on_action(cx.listener(Self::launch))
            .on_action(cx.listener(Self::launch_new))
            .on_action(cx.listener(Self::next))
            .on_action(cx.listener(Self::previous))
            .on_action(cx.listener(Self::dismiss))
            .on_action(cx.listener(Self::clear))
            .on_action(cx.listener(Self::tile))
            .on_action(cx.listener(Self::restore_layout))
            .on_key_down(cx.listener(Self::key_down))
            .on_action(cx.listener(Self::open_sidebar))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .text_sm()
                    .text_color(rgb(0xa1a1aa))
                    .child("DA WORKSTATION")
                    .child("FOCUS BAR"),
            )
            .child(
                div()
                    .p_3()
                    .rounded_lg()
                    .bg(rgb(0x18181b))
                    .border_1()
                    .border_color(rgb(0x38bdf8))
                    .child(self.input.clone()),
            )
            .child(
                div()
                    .h(px(300.))
                    .flex()
                    .flex_col()
                    .gap_1()
                    .when(self.entries() == 0, |view| {
                        view.child(
                            div()
                                .p_4()
                                .text_color(rgb(0xa1a1aa))
                                .child("No matching apps or tasks. Try another name."),
                        )
                    })
                    .children(
                        self.task_matches
                            .iter()
                            .map(|&i| {
                                let task = &self.tasks[i];
                                let (icon, label) = task.state.label();
                                (format!("Task: {}", task.title), format!("{icon} {label} · open in the sidebar"))
                            })
                            .chain(self.matches.iter().map(|&i| {
                                (self.apps[i].name.clone(), self.apps[i].id.clone())
                            }))
                            .enumerate()
                            .skip(start)
                            .take(5)
                            .map(|(position, (name, detail))| {
                            let selected = position == self.selected;
                            div()
                                .id(("app", position))
                                .flex()
                                .flex_col()
                                .px_3()
                                .py_2()
                                .rounded_md()
                                .cursor_pointer()
                                .bg(if selected {
                                    rgb(0x163344)
                                } else {
                                    rgb(0x09090b)
                                })
                                .hover(|style| style.bg(rgb(0x27272a)))
                                .child(div().text_base().child(name))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(0xa1a1aa))
                                        .overflow_hidden()
                                        .child(detail),
                                )
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.selected = position;
                                    this.launch(&Launch, window, cx);
                                }))
                        }),
                    ),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(if self.failed {
                        rgb(0xfca5a5)
                    } else {
                        rgb(0xa1a1aa)
                    })
                    .child(self.message.clone()),
            )
            .child(
                div()
                    .mt_auto()
                    .text_xs()
                    .text_color(rgb(0xa1a1aa))
                    .child("Ctrl+T  tile all · Ctrl+2–9  recent N · Ctrl+R  restore · Ctrl+B  sidebar · ↑ ↓ / Tab  select · Enter  focus/open · Ctrl+Enter  launch · Ctrl+L  clear · Esc  close"),
            )
    }
}

#[derive(Debug, PartialEq)]
enum WindowCommand {
    Layout(windows::LayoutRequest),
    Focus(String),
}

/// Window commands skip app discovery, so global shortcuts stay fast.
fn parse_window_command(args: &[String]) -> Option<Result<WindowCommand, String>> {
    let args: Vec<_> = args.iter().map(String::as_str).collect();
    Some(match args.as_slice() {
        ["--tile"] => Ok(WindowCommand::Layout(windows::LayoutRequest::Tile(0))),
        ["--tile", count] => match count.parse::<u32>() {
            Ok(count @ 2..=9) => Ok(WindowCommand::Layout(windows::LayoutRequest::Tile(count))),
            _ => Err("Usage: da-workstation --tile [2-9]".into()),
        },
        ["--restore-layout"] => Ok(WindowCommand::Layout(windows::LayoutRequest::Restore)),
        ["--focus", direction] => Ok(WindowCommand::Focus(direction.to_string())),
        ["--tile" | "--restore-layout" | "--focus", ..] => Err(
            "Usage: da-workstation --tile [2-9] | --restore-layout | --focus left|right|up|down"
                .into(),
        ),
        _ => return None,
    })
}

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if let Some(command) = parse_window_command(&args) {
        let result = match command {
            Ok(WindowCommand::Layout(request)) => windows::layout(request)
                .map(|count| println!("Layout requested for {count} windows")),
            Ok(WindowCommand::Focus(direction)) => {
                windows::focus_direction(&direction).map(|focused| {
                    println!(
                        "{}",
                        if focused {
                            "Focus requested"
                        } else {
                            "No window there"
                        }
                    )
                })
            }
            Err(usage) => {
                eprintln!("{usage}");
                std::process::exit(2);
            }
        };
        if let Err(error) = result {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return;
    }
    if args.len() == 1 && args[0] == "--help-html" {
        print!("{}", help::html());
        return;
    }
    if args.len() == 1 && args[0] == "--mcp" {
        mcp::serve();
        return;
    }
    if args.first().is_some_and(|a| a == "--sidebar") {
        match &args[1..] {
            [] => sidebar::run(None),
            [flag] if flag == "--search" => sidebar::run(Some(sidebar::SEARCH_REQUEST)),
            [flag, id] if flag == "--open" => sidebar::run(Some(id)),
            _ => {
                eprintln!("Usage: da-workstation --sidebar [--search | --open TASK_ID]");
                std::process::exit(2);
            }
        }
        return;
    }
    let apps = apps::discover();
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args
        .first()
        .is_some_and(|arg| arg == "--activate" || arg == "--app-state")
    {
        if args.len() != 2 {
            eprintln!("Usage: da-workstation {} DESKTOP_ID", args[0]);
            std::process::exit(2);
        }
        let result = if args[0] == "--app-state" {
            windows::app_state(&args[1]).map(|(count, focused)| {
                println!("windows={count} focused={focused}");
            })
        } else {
            windows::activate(&args[1]).map(|outcome| match outcome {
                windows::Activation::FocusRequested => println!("Focus requested: {}", args[1]),
                windows::Activation::LaunchRequested => println!("Launch requested: {}", args[1]),
            })
        };
        if let Err(error) = result {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return;
    }
    if args.first().is_some_and(|arg| arg == "--launch") {
        if args.len() != 2 {
            eprintln!("Usage: da-workstation --launch DESKTOP_ID");
            std::process::exit(2);
        }
        match apps::launch(&args[1]) {
            Ok(()) => println!("Launch requested: {}", args[1]),
            Err(error) => {
                eprintln!("Launch failed: {error}");
                std::process::exit(1);
            }
        }
        return;
    }
    if args.first().is_some_and(|arg| arg == "--list") {
        let query = args[1..].join(" ");
        for index in apps::matches(&apps, &query) {
            let app = &apps[index];
            println!("{}\t{}\t{}", app.id, app.name, app.description);
        }
        return;
    }
    if !args.is_empty() {
        eprintln!(
            "Usage: da-workstation [--list [QUERY] | --launch DESKTOP_ID | --activate DESKTOP_ID | --app-state DESKTOP_ID | --tile [2-9] | --restore-layout | --focus DIRECTION | --sidebar [--search | --open TASK_ID] | --mcp | --help-html]"
        );
        std::process::exit(2);
    }
    Application::new().run(move |cx: &mut App| {
        TextInput::bind_keys(cx);
        cx.bind_keys([
            KeyBinding::new("enter", Launch, Some("Launcher")),
            KeyBinding::new("ctrl-enter", LaunchNew, Some("Launcher")),
            KeyBinding::new("down", Next, Some("Launcher")),
            KeyBinding::new("tab", Next, Some("Launcher")),
            KeyBinding::new("up", Previous, Some("Launcher")),
            KeyBinding::new("shift-tab", Previous, Some("Launcher")),
            KeyBinding::new("escape", Dismiss, Some("Launcher")),
            KeyBinding::new("ctrl-l", Clear, Some("Launcher")),
            KeyBinding::new("ctrl-t", Tile, Some("Launcher")),
            KeyBinding::new("ctrl-r", RestoreLayout, Some("Launcher")),
            KeyBinding::new("ctrl-b", OpenSidebar, Some("Launcher")),
        ]);
        let bounds = Bounds::centered(None, size(px(700.), px(600.)), cx);
        let window = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                app_id: Some("dev.da.Workstation".into()),
                window_min_size: Some(size(px(560.), px(600.))),
                ..Default::default()
            },
            |window, cx| {
                window.set_window_title("DA Workstation — Focus Bar");
                let view = cx.new(|cx| Launcher::new(apps, cx));
                window.focus(&view.read(cx).input.focus_handle(cx));
                view
            },
        );
        if let Err(error) = window {
            eprintln!("Could not open focus bar: {error}");
            cx.quit();
        }
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();
        cx.activate(true);
    });
}

#[cfg(test)]
mod tests {
    use super::{WindowCommand, parse_window_command, windows};
    use windows::LayoutRequest::{Restore, Tile};

    fn parse(args: &[&str]) -> Option<Result<WindowCommand, String>> {
        parse_window_command(&args.iter().map(|a| a.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn window_commands_parse_counts_and_directions() {
        assert_eq!(parse(&["--tile"]), Some(Ok(WindowCommand::Layout(Tile(0)))));
        assert_eq!(
            parse(&["--tile", "4"]),
            Some(Ok(WindowCommand::Layout(Tile(4))))
        );
        assert_eq!(
            parse(&["--restore-layout"]),
            Some(Ok(WindowCommand::Layout(Restore)))
        );
        assert_eq!(
            parse(&["--focus", "left"]),
            Some(Ok(WindowCommand::Focus("left".into())))
        );
        assert!(matches!(parse(&["--tile", "1"]), Some(Err(_))));
        assert!(matches!(parse(&["--tile", "x"]), Some(Err(_))));
        assert!(matches!(parse(&["--focus"]), Some(Err(_))));
        assert_eq!(parse(&["--list"]), None);
    }
}
