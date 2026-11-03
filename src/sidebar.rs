//! The sidebar is the whole app: search (jump to sessions and apps, launch apps),
//! agent sessions with chats, open apps with chats, approvals and tiling keys.

use crate::activity;
use crate::apps::{self, DesktopApp};
use crate::notify;
use crate::provider::{self, Provider, ProviderEvent, Run};
use crate::switcher::{self, Hit, Row};
use crate::tasks::{Role, Store, Task, TaskState, now};
use crate::text_input::TextInput;
use crate::windows::{self, OpenWindow};
use gpui::{prelude::*, *};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

actions!(
    sidebar,
    [
        Submit,
        Back,
        StopRun,
        Approve,
        Deny,
        SearchGo,
        SearchNext,
        SearchPrev,
        SearchClear,
        FocusSearch,
        Tile,
        RestoreLayout,
        ShowHelp
    ]
);

/// Open request that focuses the search box instead of a task (`--sidebar --search`).
pub const SEARCH_REQUEST: &str = "@search";

pub const APP_ID: &str = "dev.da.Workstation.Sidebar";

struct Alert {
    task_id: Option<String>,
    text: String,
}

pub struct Sidebar {
    /// The one typing box: on the home view it searches and asks Muse; in a
    /// task it sends follow-ups.
    input: Entity<TextInput>,
    query: String,
    hits: Vec<Hit>,
    hit: usize,
    installed: Vec<DesktopApp>,
    /// Focus the search box on the next frame (render has the window).
    focus_search: bool,
    /// Show every finished session, not only the recent ones.
    show_all: bool,
    placement: windows::Placement,
    /// Auto-tiling, as the extension reports it (None: no answer yet).
    auto_stage: Option<bool>,
    /// Result of the last tiling or launch request: (text, failed).
    status: Option<(String, bool)>,
    _search_sub: Subscription,
    store: Option<Store>,
    tasks: Vec<Task>,
    selected: Option<String>,
    windows: Vec<OpenWindow>,
    /// Agent messages read per task, and activity lines read per app.
    seen: HashMap<String, usize>,
    windows_note: Option<String>,
    alerts: Vec<Alert>,
    runs: HashMap<String, Box<dyn Run>>,
    providers: Vec<Box<dyn Provider>>,
    events_tx: Sender<(String, ProviderEvent)>,
    events_rx: Receiver<(String, ProviderEvent)>,
    clicks_rx: Receiver<u32>,
    sent_tx: Sender<(u32, String)>,
    sent_rx: Receiver<(u32, String)>,
    notifications: HashMap<u32, String>,
    default_dir: String,
    scroll: ScrollHandle,
}

/// `@folder goal` sets the project folder; otherwise the default folder is used.
pub fn parse_new_task(text: &str, default_dir: &str) -> (String, String) {
    let text = text.trim();
    if let Some(rest) = text.strip_prefix('@') {
        let (dir, goal) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
        let dir = match dir.strip_prefix("~/") {
            Some(tail) => std::env::var("HOME")
                .map(|h| format!("{h}/{tail}"))
                .unwrap_or(dir.into()),
            None => dir.to_string(),
        };
        return (dir, goal.trim().to_string());
    }
    (default_dir.to_string(), text.to_string())
}

impl Sidebar {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| {
            TextInput::new(cx)
                .with_placeholder("Describe a task for Muse…")
                .with_text_size(px(14.))
        });
        let search_sub = cx.observe(&input, |this: &mut Self, _, cx| {
            // Typing searches only on the home view; in a chat it is a reply.
            let home = this.selected.is_none();
            let query = if home {
                this.input.read(cx).value().to_string()
            } else {
                String::new()
            };
            if query != this.query {
                this.query = query;
                this.hit = 0;
                this.refresh_hits();
                cx.notify();
            }
        });
        let (store, tasks, mut alerts) = match Store::open() {
            Ok(store) => {
                let (tasks, problems) = store.load();
                let alerts = problems
                    .into_iter()
                    .map(|text| Alert {
                        task_id: None,
                        text,
                    })
                    .collect();
                (Some(store), tasks, alerts)
            }
            Err(text) => (
                None,
                Vec::new(),
                vec![Alert {
                    task_id: None,
                    text,
                }],
            ),
        };
        let providers = provider::all();
        if let Some(reason) = providers[0].unavailable_reason() {
            alerts.push(Alert {
                task_id: None,
                text: reason,
            });
        }
        let (events_tx, events_rx) = mpsc::channel();
        let (clicks_tx, clicks_rx) = mpsc::channel();
        let (sent_tx, sent_rx) = mpsc::channel();
        notify::listen_for_clicks(clicks_tx);
        // Messages that exist at start count as read.
        let seen = tasks
            .iter()
            .map(|t| (t.id.clone(), agent_messages(t)))
            .collect();
        let default_dir = crate::tasks::last_project_dir(&tasks).unwrap_or_else(|| {
            std::env::current_dir()
                .map(|d| d.display().to_string())
                .unwrap_or_else(|_| std::env::var("HOME").unwrap_or_default())
        });

        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(200))
                    .await;
                if this.update(cx, |this, cx| this.drain(cx)).is_err() {
                    break;
                }
            }
        })
        .detach();
        cx.spawn(async move |this, cx| {
            loop {
                let result = cx
                    .background_executor()
                    .spawn(async { windows::list_windows() })
                    .await;
                if this
                    .update(cx, |this, cx| {
                        match result {
                            Ok(list) => {
                                this.windows = list;
                                this.windows_note = None;
                            }
                            Err(error) => this.windows_note = Some(error),
                        }
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
                cx.background_executor()
                    .timer(Duration::from_millis(1500))
                    .await;
            }
        })
        .detach();

        Self {
            input,
            query: String::new(),
            hits: Vec::new(),
            hit: 0,
            installed: apps::discover(),
            focus_search: false,
            show_all: false,
            status: None,
            placement: windows::placement(),
            auto_stage: windows::auto_stage(),
            _search_sub: search_sub,
            store,
            tasks,
            selected: None,
            windows: Vec::new(),
            seen,
            windows_note: None,
            alerts,
            runs: HashMap::new(),
            providers,
            events_tx,
            events_rx,
            clicks_rx,
            sent_tx,
            sent_rx,
            notifications: HashMap::new(),
            default_dir,
            scroll: ScrollHandle::new(),
        }
    }

    pub fn input(&self) -> &Entity<TextInput> {
        &self.input
    }

    fn task_mut(&mut self, id: &str) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|t| t.id == id)
    }

    fn save(&mut self, id: &str) {
        let Some(store) = &self.store else { return };
        if let Some(task) = self.tasks.iter().find(|t| t.id == id) {
            if let Err(text) = store.save(task) {
                self.alerts.insert(
                    0,
                    Alert {
                        task_id: None,
                        text,
                    },
                );
            }
        }
    }

    /// One alert per real transition: in the sidebar, and as a desktop notification.
    fn alert(&mut self, task_id: &str, text: String) {
        self.alerts.insert(
            0,
            Alert {
                task_id: Some(task_id.into()),
                text: text.clone(),
            },
        );
        self.alerts.truncate(30);
        let title = self
            .tasks
            .iter()
            .find(|t| t.id == task_id)
            .map(|t| t.title.clone())
            .unwrap_or_default();
        let sent = self.sent_tx.clone();
        let id = task_id.to_string();
        std::thread::spawn(move || {
            if let Ok(notification) = notify::send(&text, &title) {
                let _ = sent.send((notification, id));
            }
        });
    }

    fn drain(&mut self, cx: &mut Context<Self>) {
        let mut changed = HashSet::new();
        while let Ok((id, event)) = self.events_rx.try_recv() {
            self.apply(&id, event);
            changed.insert(id);
        }
        for id in &changed {
            self.save(id);
        }
        while let Ok((notification, task)) = self.sent_rx.try_recv() {
            self.notifications.insert(notification, task);
        }
        let mut opened = false;
        if let Some(id) = self.store.as_ref().and_then(Store::take_open_request) {
            if id == SEARCH_REQUEST {
                self.selected = None;
                self.focus_search = true;
                opened = true;
            } else if self.tasks.iter().any(|t| t.id == id) {
                self.open_task(id);
                opened = true;
            }
        }
        while let Ok(notification) = self.clicks_rx.try_recv() {
            if let Some(task) = self.notifications.get(&notification).cloned() {
                self.open_task(task);
                opened = true;
            }
        }
        if opened {
            cx.background_executor()
                .spawn(async { windows::place_sidebar(true) })
                .detach();
        }
        if !changed.is_empty() || opened {
            if self.selected.as_ref().is_some_and(|s| changed.contains(s)) {
                self.scroll.scroll_to_bottom();
            }
            cx.notify();
        }
    }

    fn apply(&mut self, id: &str, event: ProviderEvent) {
        let mut alert = None;
        let finished = matches!(event, ProviderEvent::Finished { .. });
        {
            let Some(task) = self.task_mut(id) else {
                return;
            };
            match event {
                ProviderEvent::Started { session_ref } => {
                    task.push(Role::Event, format!("Started session {session_ref}"));
                    task.session_ref = Some(session_ref);
                    task.transition(TaskState::Running);
                }
                ProviderEvent::Output(line) => append(task, Role::Agent, line),
                ProviderEvent::Diagnostic(line) => append(task, Role::Event, line),
                ProviderEvent::FilesChanged(files) => task.record_files(files),
                ProviderEvent::NeedsApproval {
                    id: approval,
                    summary,
                } => {
                    task.push(Role::Event, format!("Permission requested: {summary}"));
                    task.pending_approval = Some(crate::tasks::Approval {
                        id: approval,
                        summary: summary.clone(),
                        answered: None,
                    });
                    if task.transition(TaskState::NeedsPermission) {
                        alert = Some(format!("⚠ Needs permission: {summary}"));
                    }
                }
                ProviderEvent::ApprovalResolved {
                    id: approval,
                    summary,
                } => {
                    if task
                        .pending_approval
                        .as_ref()
                        .is_some_and(|p| p.id == approval)
                    {
                        task.pending_approval = None;
                    }
                    task.push(Role::Event, summary);
                    if task.state == TaskState::NeedsPermission {
                        task.transition(TaskState::Running);
                    }
                }
                ProviderEvent::Finished { ok, summary } => {
                    task.pending_approval = None;
                    let files = if task.run_files.is_empty() {
                        "No files changed by the agent's file tools this run.".to_string()
                    } else {
                        let base = format!("{}/", task.project_dir.trim_end_matches('/'));
                        let names: Vec<_> = task
                            .run_files
                            .iter()
                            .map(|f| f.strip_prefix(&base).unwrap_or(f))
                            .collect();
                        format!("Files changed this run: {}.", names.join(", "))
                    };
                    task.push(Role::Event, format!("{summary} {files} The agent's last message is not proof of success; check the result."));
                    if task.state != TaskState::Stopped {
                        let state = if ok {
                            TaskState::Done
                        } else {
                            TaskState::Failed
                        };
                        if task.transition(state) {
                            let (icon, label) = state.label();
                            alert = Some(format!("{icon} {label}"));
                        }
                    }
                }
            }
        }
        if finished {
            self.runs.remove(id);
        }
        if let Some(text) = alert {
            self.alert(id, text);
        }
    }

    fn start_run(&mut self, id: &str, prompt: String, session: Option<String>) {
        let Some(task) = self.tasks.iter().find(|t| t.id == id) else {
            return;
        };
        let Some(provider) = self.providers.iter().find(|p| p.id() == task.provider) else {
            let text = format!("Unknown provider {}", task.provider);
            if let Some(task) = self.task_mut(id) {
                task.push(Role::Error, text);
            }
            return;
        };
        if let Some(reason) = provider.unavailable_reason() {
            if let Some(task) = self.task_mut(id) {
                task.push(Role::Error, reason);
            }
            self.save(id);
            return;
        }
        let result = provider.start(
            id,
            &task.project_dir.clone(),
            &prompt,
            session.as_deref(),
            self.events_tx.clone(),
        );
        match result {
            Ok(run) => {
                self.runs.insert(id.into(), run);
                if let Some(task) = self.task_mut(id) {
                    task.run_count += 1;
                    task.run_files.clear();
                    task.transition(TaskState::Running);
                }
            }
            Err(text) => {
                let changed = self.task_mut(id).is_some_and(|task| {
                    task.push(Role::Error, text.clone());
                    task.transition(TaskState::Failed)
                });
                if changed {
                    self.alert(id, format!("✕ Could not start: {text}"));
                }
            }
        }
        self.save(id);
    }

    fn submit(&mut self, _: &Submit, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.input.read(cx).value().trim().to_string();
        if text.is_empty() {
            return;
        }
        // Home view: Enter runs the chosen row; "Ask Muse" falls through to a new task.
        if self.selected.is_none() {
            if let Some(Row::Hit(hit)) = self.rows().get(self.hit).cloned() {
                self.go(hit, window, cx);
                return;
            }
        }
        self.query.clear();
        self.hits.clear();
        self.hit = 0;
        self.input.update(cx, |input, cx| {
            input.reset();
            cx.notify();
        });
        match self.selected.clone() {
            Some(id) => {
                if self.runs.contains_key(&id) {
                    if let Some(task) = self.task_mut(&id) {
                        task.push(Role::Error, "This Muse connection takes follow-ups between runs. Wait for the run to finish, or press Stop.");
                    }
                } else if let Some(task) = self.tasks.iter().find(|t| t.id == id) {
                    // Continue the provider's own session when it keeps context;
                    // otherwise send the portable handoff.
                    let session = task.session_ref.clone().filter(|s| {
                        self.providers
                            .iter()
                            .any(|p| p.id() == task.provider && p.resumes(s))
                    });
                    let prompt = match session {
                        Some(_) => text.clone(),
                        None => task.follow_up_prompt(&text),
                    };
                    if let Some(task) = self.task_mut(&id) {
                        task.push(Role::User, text);
                    }
                    self.start_run(&id, prompt, session);
                }
                self.save(&id);
            }
            None => {
                let (dir, goal) = parse_new_task(&text, &self.default_dir);
                if goal.is_empty() {
                    return;
                }
                if !std::path::Path::new(&dir).is_dir() {
                    self.alerts.push(Alert {
                        task_id: None,
                        text: format!(
                            "No folder {}. Check the @folder and try again.",
                            short_dir(&dir)
                        ),
                    });
                    cx.notify();
                    return;
                }
                self.default_dir = dir.clone();
                let mut task = Task::new(&goal, &dir, self.providers[0].id());
                task.push(Role::User, goal.clone());
                let id = task.id.clone();
                self.tasks.insert(0, task);
                self.selected = Some(id.clone());
                self.save(&id);
                self.start_run(&id, goal, None);
            }
        }
        self.scroll.scroll_to_bottom();
        cx.notify();
    }

    fn open_task(&mut self, id: String) {
        self.selected = Some(id);
        self.scroll.scroll_to_bottom();
    }

    /// New agent messages since the task was last open.
    fn unread(&self, task: &Task) -> usize {
        agent_messages(task).saturating_sub(self.seen.get(&task.id).copied().unwrap_or(0))
    }

    fn refresh_hits(&mut self) {
        let groups = activity::group(&self.windows);
        self.hits = switcher::search(&self.tasks, &groups, &self.installed, &self.query);
        let rows = switcher::rows(&self.hits, &self.query).len();
        self.hit = self.hit.min(rows.saturating_sub(1));
    }

    fn rows(&self) -> Vec<Row> {
        switcher::rows(&self.hits, &self.query)
    }

    fn clear_search(&mut self, cx: &mut Context<Self>) {
        self.input.update(cx, |input, cx| {
            input.reset();
            cx.notify();
        });
        self.query.clear();
        self.hits.clear();
        self.hit = 0;
    }

    fn search_next(&mut self, _: &SearchNext, _: &mut Window, cx: &mut Context<Self>) {
        let n = self.rows().len();
        if n > 0 {
            self.hit = (self.hit + 1) % n;
            cx.notify();
        }
    }

    fn search_prev(&mut self, _: &SearchPrev, _: &mut Window, cx: &mut Context<Self>) {
        let n = self.rows().len();
        if n > 0 {
            self.hit = (self.hit + n - 1) % n;
            cx.notify();
        }
    }

    fn search_clear(&mut self, _: &SearchClear, window: &mut Window, cx: &mut Context<Self>) {
        self.clear_search(cx);
        window.focus(&self.input.focus_handle(cx));
        cx.notify();
    }

    /// Ctrl+K: back to the home view with the typing box focused.
    fn focus_search_box(&mut self, _: &FocusSearch, window: &mut Window, cx: &mut Context<Self>) {
        self.selected = None;
        window.focus(&self.input.focus_handle(cx));
        cx.notify();
    }

    fn search_go(&mut self, _: &SearchGo, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(Row::Hit(hit)) = self.rows().get(self.hit).cloned() {
            self.go(hit, window, cx);
        }
    }

    /// Open a search result: a session's chat, an open app (focus it), or an
    /// installed app (focus its window, or launch it).
    fn go(&mut self, hit: Hit, window: &mut Window, cx: &mut Context<Self>) {
        match hit {
            Hit::Session(i) => {
                if let Some(id) = self.tasks.get(i).map(|t| t.id.clone()) {
                    self.open_task(id);
                }
            }
            Hit::OpenApp(key) => {
                if let Some(id) = self
                    .windows
                    .iter()
                    .find(|w| activity::app_key(w) == key)
                    .map(|w| w.id)
                {
                    // Opening from DA Workstation also tiles: the window becomes big.
                    cx.background_executor()
                        .spawn(async move {
                            if windows::focus_window(id).unwrap_or(false) {
                                windows::stage_focused();
                            }
                        })
                        .detach();
                }
            }
            Hit::Command(command) => self.run_command(command, cx),
            Hit::Installed(i) => {
                let Some(app) = self.installed.get(i).cloned() else {
                    return;
                };
                self.status = Some((format!("Opening {}…", app.name), false));
                let task = cx
                    .background_executor()
                    .spawn(async move { (app.name.clone(), windows::open_in_stage(&app.id)) });
                cx.spawn(async move |this, cx| {
                    let (name, result) = task.await;
                    let _ = this.update(cx, |this, cx| {
                        this.status = Some(match result {
                            Ok(windows::Activation::FocusRequested) => {
                                (format!("Focused {name}."), false)
                            }
                            Ok(windows::Activation::LaunchRequested) => {
                                (format!("Launching {name}."), false)
                            }
                            Err(error) => (format!("Could not open {name}: {error}"), true),
                        });
                        cx.notify();
                    });
                })
                .detach();
            }
        }
        self.clear_search(cx);
        window.focus(&self.input.focus_handle(cx));
        cx.notify();
    }

    /// Run a `/command` from the typing box.
    fn run_command(&mut self, command: switcher::Command, cx: &mut Context<Self>) {
        use switcher::Command::*;
        match command {
            Tile => self.change_layout(windows::LayoutRequest::Tile(0), cx),
            Restore => self.change_layout(windows::LayoutRequest::Restore, cx),
            BottomBar if self.placement == windows::Placement::Left => self.toggle_placement(cx),
            SidePanel if self.placement == windows::Placement::Bottom => self.toggle_placement(cx),
            BottomBar | SidePanel => {}
            AutoTileOn => self.set_auto_stage(true, cx),
            AutoTileOff => self.set_auto_stage(false, cx),
            Help => self.open_help(cx),
        }
    }

    fn set_auto_stage(&mut self, on: bool, cx: &mut Context<Self>) {
        let task = cx
            .background_executor()
            .spawn(async move { windows::set_auto_stage(on) });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                match result {
                    Ok(on) => {
                        this.auto_stage = Some(on);
                        this.status = Some((
                            if on {
                                "Auto-tiling on: every new window joins the tiles.".into()
                            } else {
                                "Auto-tiling off: only apps opened here are tiled.".into()
                            },
                            false,
                        ));
                    }
                    Err(error) => this.status = Some((error, true)),
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// Switch between the side panel and the bottom bar; remembered for next time.
    fn toggle_placement(&mut self, cx: &mut Context<Self>) {
        self.placement = match self.placement {
            windows::Placement::Left => windows::Placement::Bottom,
            windows::Placement::Bottom => windows::Placement::Left,
        };
        if let Err(error) = windows::set_placement(self.placement) {
            self.status = Some((error, true));
        }
        cx.background_executor()
            .spawn(async { windows::place_sidebar(true) })
            .detach();
        cx.notify();
    }

    fn tile(&mut self, _: &Tile, _: &mut Window, cx: &mut Context<Self>) {
        self.change_layout(windows::LayoutRequest::Tile(0), cx);
    }

    fn restore_layout(&mut self, _: &RestoreLayout, _: &mut Window, cx: &mut Context<Self>) {
        self.change_layout(windows::LayoutRequest::Restore, cx);
    }

    /// Ctrl+2 to Ctrl+9: tile that many recent windows.
    fn key_down(&mut self, event: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let keystroke = &event.keystroke;
        let m = &keystroke.modifiers;
        if m.control && !m.alt && !m.shift && !m.platform {
            if let Ok(count @ 2..=9) = keystroke.key.parse::<u32>() {
                self.change_layout(windows::LayoutRequest::Tile(count), cx);
                cx.stop_propagation();
            }
        }
    }

    fn change_layout(&mut self, request: windows::LayoutRequest, cx: &mut Context<Self>) {
        let restore = request == windows::LayoutRequest::Restore;
        let task = cx
            .background_executor()
            .spawn(async move { windows::layout(request) });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                this.status = Some(match result {
                    Ok(0) if restore => ("No saved layout. Press Ctrl+T first.".into(), false),
                    Ok(count) if restore => (format!("Restored {count} windows."), false),
                    Ok(count) => (
                        format!("Tiled {count} windows. Click a window to bring it forward; Ctrl+R restores."),
                        false,
                    ),
                    Err(error) => (error, true),
                });
                cx.notify();
            });
        })
        .detach();
    }

    fn show_help(&mut self, _: &ShowHelp, _: &mut Window, cx: &mut Context<Self>) {
        self.open_help(cx);
    }

    /// Open the Help page in the browser (it becomes the big window).
    fn open_help(&mut self, cx: &mut Context<Self>) {
        self.status = Some(("Opening Help in your browser…".into(), false));
        let task = cx
            .background_executor()
            .spawn(async { crate::help::open() });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                this.status = Some(match result {
                    Ok(_) => ("Help is open in your browser.".into(), false),
                    Err(error) => (error, true),
                });
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    fn back(&mut self, _: &Back, _: &mut Window, cx: &mut Context<Self>) {
        if !self.query.is_empty() {
            self.clear_search(cx);
            cx.notify();
            return;
        }
        self.selected = None;
        cx.notify();
    }

    fn stop(&mut self, _: &StopRun, _: &mut Window, cx: &mut Context<Self>) {
        let Some(id) = self.selected.clone() else {
            return;
        };
        let result = self.runs.get_mut(&id).map(|run| run.cancel());
        if let Some(task) = self.task_mut(&id) {
            match result {
                Some(Ok(())) => {
                    task.pending_approval = None;
                    task.transition(TaskState::Stopped);
                    task.push(Role::Event, "Stop requested.");
                }
                Some(Err(e)) => task.push(Role::Error, e),
                None => return,
            }
        }
        self.save(&id);
        cx.notify();
    }

    fn answer(&mut self, allow: bool, cx: &mut Context<Self>) {
        if let Some(id) = self.selected.clone() {
            self.answer_task(id, allow, cx);
        }
    }

    /// Answer a task's pending request, from its chat or from the "Needs you" inbox.
    fn answer_task(&mut self, id: String, allow: bool, cx: &mut Context<Self>) {
        let Some(approval) = self
            .tasks
            .iter()
            .find(|t| t.id == id)
            .and_then(|t| t.pending_approval.clone())
            .filter(|p| p.answered.is_none())
        else {
            return;
        };
        let result = match self.runs.get_mut(&id) {
            Some(run) => run.answer_approval(&approval.id, allow),
            None => Err("The run that asked has ended.".into()),
        };
        if let Some(task) = self.task_mut(&id) {
            match result {
                Ok(()) => {
                    if let Some(pending) = task.pending_approval.as_mut() {
                        pending.answered = Some(allow);
                    }
                    task.push(
                        Role::Event,
                        if allow {
                            "Sent: allow once. Waiting for Muse to confirm."
                        } else {
                            "Sent: reject. Waiting for Muse to confirm."
                        },
                    );
                }
                Err(e) => task.push(Role::Error, e),
            }
        }
        self.save(&id);
        cx.notify();
    }

    fn approve(&mut self, _: &Approve, _: &mut Window, cx: &mut Context<Self>) {
        self.answer(true, cx);
    }

    fn deny(&mut self, _: &Deny, _: &mut Window, cx: &mut Context<Self>) {
        self.answer(false, cx);
    }
}

fn append(task: &mut Task, role: Role, line: String) {
    match task.messages.last_mut() {
        Some(last) if last.role == role && now() - last.at < 30 => {
            last.text.push('\n');
            last.text.push_str(&line);
            last.at = now();
            task.updated_at = last.at;
        }
        _ => task.push(role, line),
    }
}

const BG: u32 = 0x0b0b0f;
const PANEL: u32 = 0x15151b;
const RAISED: u32 = 0x1d1d25;
const HOVER: u32 = 0x24242e;
const BORDER: u32 = 0x2a2a35;
const TEXT: u32 = 0xf4f4f5;
const MUTED: u32 = 0x8e8e9a;
const ACCENT: u32 = 0x38bdf8;
const MUSE: u32 = 0xa78bfa;
const WARN: u32 = 0xfbbf24;
const OK: u32 = 0x4ade80;
const ERROR: u32 = 0xf87171;

/// Color for a task state. Status text and icon always go with it.
fn state_color(state: TaskState) -> u32 {
    match state {
        TaskState::Running => ACCENT,
        TaskState::NeedsPermission => WARN,
        TaskState::Done => OK,
        TaskState::Failed => ERROR,
        TaskState::Ready | TaskState::Stopped | TaskState::Interrupted => MUTED,
    }
}

/// Small rounded label: icon and text in the state's color.
fn pill(text: String, color: u32) -> Div {
    div()
        .flex_none()
        .px_2()
        .rounded_full()
        .border_1()
        .border_color(rgba((color << 8) | 0x55))
        .bg(rgba((color << 8) | 0x14))
        .text_xs()
        .text_color(rgb(color))
        .child(text)
}

/// One line that never leaves its box: long text ends in "…".
fn line(text: String) -> Div {
    div()
        .w_full()
        .min_w_0()
        .whitespace_nowrap()
        .overflow_hidden()
        .text_ellipsis()
        .child(text)
}

/// `line` for use inside a column: a column does not give truncated text a width,
/// so the line sits in a row and fills it. Text styles set on the result apply.
fn col_line(text: String) -> Div {
    div().flex().min_w_0().child(line(text).flex_1())
}

fn section(title: &str, count: Option<usize>) -> Div {
    div()
        .mt_3()
        .mb_1()
        .flex()
        .items_center()
        .gap_2()
        .text_xs()
        .text_color(rgb(MUTED))
        .child(title.to_string())
        .when_some(count.filter(|&n| n > 0), |d, n| {
            d.child(div().text_color(rgb(0x5c5c68)).child(n.to_string()))
        })
        .child(div().flex_1().h(px(1.)).bg(rgb(BORDER)))
}

/// The session list's day heading for a time: Today, Yesterday, Previous 7 days, Older.
fn day_group(at: u64) -> &'static str {
    use gio::glib::DateTime;
    let (Ok(now), Ok(then)) = (DateTime::now_local(), DateTime::from_unix_local(at as i64)) else {
        return "Older";
    };
    let day = |d: &DateTime| d.ymd();
    if day(&then) == day(&now) {
        return "Today";
    }
    if now.add_days(-1).is_ok_and(|y| day(&y) == day(&then)) {
        return "Yesterday";
    }
    if now.add_days(-7).is_ok_and(|w| then >= w) {
        return "Previous 7 days";
    }
    "Older"
}

/// A request is waiting for the user's answer.
fn needs_you(task: &Task) -> bool {
    task.pending_approval
        .as_ref()
        .is_some_and(|a| a.answered.is_none())
}

/// Agent replies in a task, for unread counts.
fn agent_messages(task: &Task) -> usize {
    task.messages
        .iter()
        .filter(|m| m.role == Role::Agent)
        .count()
}

/// A small filled count, like an unread badge.
fn badge(count: usize, color: u32) -> Div {
    div()
        .flex_none()
        .px_1p5()
        .rounded_full()
        .bg(rgb(color))
        .text_xs()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(rgb(BG))
        .child(count.to_string())
}

/// A rounded square with the app's initials, colored from its key.
fn avatar(name: &str, key: &str, size: f32) -> Div {
    const COLORS: [u32; 6] = [0x38bdf8, 0xa78bfa, 0x4ade80, 0xfbbf24, 0xf472b6, 0x2dd4bf];
    let hash = key
        .bytes()
        .fold(0usize, |h, b| h.wrapping_mul(31).wrapping_add(b as usize));
    let color = COLORS[hash % COLORS.len()];
    div()
        .flex_none()
        .size(px(size))
        .rounded_md()
        .flex()
        .items_center()
        .justify_center()
        .bg(rgba((color << 8) | 0x2e))
        .border_1()
        .border_color(rgba((color << 8) | 0x66))
        .text_color(rgb(color))
        .text_size(px(size * 0.42))
        .font_weight(FontWeight::BOLD)
        .child(activity::initials(name))
}

fn button(id: &'static str, label: &str, color: u32) -> Stateful<Div> {
    button_id(id, label, color)
}

fn button_id(id: impl Into<ElementId>, label: &str, color: u32) -> Stateful<Div> {
    div()
        .id(id)
        .px_3()
        .py_1()
        .rounded_lg()
        .border_1()
        .border_color(rgba((color << 8) | 0x99))
        .text_color(rgb(color))
        .text_sm()
        .cursor_pointer()
        .hover(|s| s.bg(rgb(HOVER)))
        .child(label.to_string())
}

/// The main action: filled.
fn primary(id: &'static str, label: &str, color: u32) -> Stateful<Div> {
    primary_id(id, label, color)
}

fn primary_id(id: impl Into<ElementId>, label: &str, color: u32) -> Stateful<Div> {
    div()
        .id(id)
        .px_3()
        .py_1()
        .rounded_lg()
        .bg(rgb(color))
        .text_color(rgb(BG))
        .text_sm()
        .font_weight(FontWeight::SEMIBOLD)
        .cursor_pointer()
        .hover(|s| s.opacity(0.85))
        .child(label.to_string())
}

impl Sidebar {
    /// The one typing box and its hint line. `chat`: replying in a task.
    fn render_composer(&self, chat: bool, cx: &mut Context<Self>) -> Div {
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_2()
                    .py_1()
                    .rounded_xl()
                    .bg(rgb(RAISED))
                    .border_1()
                    .border_color(rgb(if chat { MUSE } else { ACCENT }))
                    .child(div().flex_1().min_w_0().child(self.input.clone()))
                    .when(!self.query.is_empty(), |d| {
                        d.child(
                            div()
                                .id("clear")
                                .flex_none()
                                .px_1()
                                .text_sm()
                                .text_color(rgb(MUTED))
                                .cursor_pointer()
                                .hover(|s| s.text_color(rgb(TEXT)))
                                .child("×")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.clear_search(cx);
                                    window.focus(&this.input.focus_handle(cx));
                                    cx.notify();
                                })),
                        )
                    })
                    .child(
                        div()
                            .flex_none()
                            .px_1p5()
                            .rounded_md()
                            .border_1()
                            .border_color(rgb(BORDER))
                            .text_xs()
                            .text_color(rgb(MUTED))
                            .child("↵"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .min_w_0()
                    .text_xs()
                    .text_color(rgb(MUTED))
                    .child(match chat {
                        true => line("Enter: follow-up · Esc: back · Ctrl+.: stop".into()),
                        false => div(),
                    })
                    .when(!chat, |d| {
                        let bottom = self.placement == windows::Placement::Bottom;
                        let auto = self.auto_stage.unwrap_or(true);
                        let link = |id: &'static str, text: String| {
                            div()
                                .id(id)
                                .flex_none()
                                .px_1p5()
                                .rounded_md()
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(HOVER)).text_color(rgb(TEXT)))
                                .child(text)
                        };
                        d.child(
                            pill(format!("in {}", short_dir(&self.default_dir)), ACCENT)
                                .flex_shrink(),
                        )
                        .child(div().flex_1())
                        .child(
                            link(
                                "placement",
                                if bottom { "⇤ Side" } else { "⇩ Bottom" }.into(),
                            )
                            .on_click(cx.listener(|this, _, _, cx| this.toggle_placement(cx))),
                        )
                        .child(
                            link(
                                "autotile",
                                format!("Auto-tile {}", if auto { "on" } else { "off" }),
                            )
                            .on_click(
                                cx.listener(move |this, _, _, cx| this.set_auto_stage(!auto, cx)),
                            ),
                        )
                    }),
            )
    }

    /// Choices for the typed text: Up/Down choose, Enter runs, a click runs too.
    fn render_results(&self, cx: &mut Context<Self>) -> Div {
        let groups = activity::group(&self.windows);
        let (dir, goal) = parse_new_task(&self.query, &self.default_dir);
        let rows = self.rows().into_iter().enumerate().map(|(i, row)| {
            let (lead, title, detail) = match &row {
                Row::Ask => (
                    avatar("Muse", "agent", 22.),
                    format!("Ask Muse: {goal}"),
                    format!(
                        "New task in {} · Muse can open apps, tile and browse",
                        short_dir(&dir)
                    ),
                ),
                Row::Hit(Hit::Session(t)) => {
                    let task = &self.tasks[*t];
                    let (icon, label) = task.state.label();
                    (
                        div()
                            .flex_none()
                            .w(px(22.))
                            .text_color(rgb(state_color(task.state)))
                            .child(icon),
                        task.title.clone(),
                        format!("Session · {label}"),
                    )
                }
                Row::Hit(Hit::OpenApp(key)) => {
                    let group = groups.iter().find(|g| &g.key == key);
                    let name = group.map_or(key.clone(), |g| g.name.clone());
                    let count = group.map_or(0, |g| g.windows.len());
                    (
                        avatar(&name, key, 22.),
                        name,
                        format!(
                            "Open app · {count} window{} · focus it",
                            if count == 1 { "" } else { "s" }
                        ),
                    )
                }
                Row::Hit(Hit::Command(command)) => {
                    let (name, about) = command.text();
                    (
                        div()
                            .flex_none()
                            .size(px(22.))
                            .rounded_md()
                            .flex()
                            .items_center()
                            .justify_center()
                            .bg(rgb(RAISED))
                            .text_color(rgb(ACCENT))
                            .child("/"),
                        format!("/{name}"),
                        about.to_string(),
                    )
                }
                Row::Hit(Hit::Installed(a)) => {
                    let app = &self.installed[*a];
                    (
                        avatar(&app.name, &app.id, 22.),
                        app.name.clone(),
                        "App · launch it".to_string(),
                    )
                }
            };
            div()
                .id(("row", i))
                .flex()
                .items_center()
                .gap_2()
                .px_2()
                .py_1()
                .rounded_md()
                .cursor_pointer()
                .when(i == self.hit, |d| d.bg(rgb(0x163344)))
                .hover(|s| s.bg(rgb(HOVER)))
                .child(lead)
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .child(col_line(title).text_sm())
                        .child(col_line(detail).text_xs().text_color(rgb(MUTED))),
                )
                .when(i == self.hit, |d| {
                    d.child(
                        div()
                            .flex_none()
                            .text_xs()
                            .text_color(rgb(MUTED))
                            .child("↵"),
                    )
                })
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.hit = i;
                    match row.clone() {
                        Row::Hit(hit) => this.go(hit, window, cx),
                        Row::Ask => this.submit(&Submit, window, cx),
                    }
                }))
        });
        div()
            .flex_1()
            .flex()
            .flex_col()
            // Side panel: results sit just above the box. Bottom bar: they start at the top.
            .when(self.placement == windows::Placement::Left, |d| {
                d.justify_end()
            })
            .gap_0p5()
            .min_h_0()
            .child(section("RESULTS", None))
            .children(rows)
    }

    /// Home, after the Claude and ChatGPT desktop apps: a "New task" button, what needs
    /// you, then sessions by day. Open apps live on the right as tiled cards.
    fn render_home(&self, cx: &mut Context<Self>) -> Div {
        let inbox = self
            .tasks
            .iter()
            .filter(|t| needs_you(t))
            .enumerate()
            .map(|(i, task)| {
                let summary = task
                    .pending_approval
                    .as_ref()
                    .map(|a| a.summary.clone())
                    .unwrap_or_default();
                let (open, allow, deny) = (task.id.clone(), task.id.clone(), task.id.clone());
                div()
                    .flex()
                    .flex_col()
                    .gap_1p5()
                    .p_2()
                    .rounded_lg()
                    .bg(rgba((WARN << 8) | 0x10))
                    .border_1()
                    .border_color(rgba((WARN << 8) | 0x88))
                    .child(
                        div()
                            .id(("inbox-title", i))
                            .cursor_pointer()
                            .child(
                                col_line(task.title.clone())
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD),
                            )
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.open_task(open.clone());
                                cx.notify();
                            })),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(WARN))
                            .child(format!("Muse asks to {summary}")),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(primary_id(("inbox-allow", i), "Allow once", WARN).on_click(
                                cx.listener(move |this, _, _, cx| {
                                    this.answer_task(allow.clone(), true, cx)
                                }),
                            ))
                            .child(button_id(("inbox-deny", i), "Reject", ERROR).on_click(
                                cx.listener(move |this, _, _, cx| {
                                    this.answer_task(deny.clone(), false, cx)
                                }),
                            )),
                    )
            })
            .collect::<Vec<_>>();
        // Sessions by day, newest first; "Older" folds behind a link.
        let mut order: Vec<usize> = (0..self.tasks.len())
            .filter(|&i| !needs_you(&self.tasks[i]))
            .collect();
        order.sort_by_key(|&i| std::cmp::Reverse(self.tasks[i].updated_at));
        let mut groups: Vec<(&'static str, Vec<usize>)> = Vec::new();
        for i in order {
            let name = day_group(self.tasks[i].updated_at);
            match groups.last_mut() {
                Some((last, list)) if *last == name => list.push(i),
                _ => groups.push((name, vec![i])),
            }
        }
        let older = groups
            .iter()
            .find(|(name, _)| *name == "Older")
            .map_or(0, |(_, list)| list.len());
        let mut rows: Vec<AnyElement> = Vec::new();
        for (name, list) in groups {
            if name == "Older" && !self.show_all {
                continue;
            }
            rows.push(
                div()
                    .px_2()
                    .pt_3()
                    .pb_1()
                    .text_xs()
                    .text_color(rgb(0x6b6b78))
                    .child(name)
                    .into_any_element(),
            );
            for i in list {
                rows.push(self.session_row(i, cx).into_any_element());
            }
        }
        let notices = self
            .alerts
            .iter()
            .filter(|a| a.task_id.is_none())
            .take(2)
            .map(|a| a.text.clone())
            .collect::<Vec<_>>();
        div()
            .flex_1()
            .flex()
            .flex_col()
            .gap_1()
            .min_h_0()
            // Window-control problems (such as an old extension) go first: they
            // also explain why the panel is not docked.
            .children(
                self.windows_note
                    .clone()
                    .into_iter()
                    .chain(notices)
                    .map(|note| {
                        div()
                            .p_2()
                            .rounded_lg()
                            .bg(rgba((WARN << 8) | 0x10))
                            .border_1()
                            .border_color(rgb(WARN))
                            .text_xs()
                            .text_color(rgb(WARN))
                            .child(format!("⚠ {note}"))
                    }),
            )
            .child(
                div()
                    .id("new-task")
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .py_2()
                    .rounded_lg()
                    .border_1()
                    .border_color(rgb(BORDER))
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(HOVER)))
                    .child(div().text_color(rgb(ACCENT)).child("＋"))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .child("New task"),
                    )
                    .child(div().flex_1())
                    .child(div().text_xs().text_color(rgb(0x6b6b78)).child("Ctrl K"))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.selected = None;
                        this.clear_search(cx);
                        window.focus(&this.input.focus_handle(cx));
                        cx.notify();
                    })),
            )
            .when(!inbox.is_empty(), |d| {
                d.child(section("NEEDS YOU", Some(inbox.len())))
                    .children(inbox)
            })
            .child(
                div()
                    .id("sessions")
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .overflow_y_scroll()
                    .when(self.tasks.is_empty(), |d| {
                        d.child(div().px_2().py_3().text_sm().text_color(rgb(MUTED)).child(
                            "No tasks yet. Type below: open an app, run /tile, or ask Muse.",
                        ))
                    })
                    .children(rows)
                    .when(older > 0, |d| {
                        d.child(
                            div()
                                .id("show-all")
                                .px_2()
                                .py_1()
                                .text_xs()
                                .text_color(rgb(ACCENT))
                                .cursor_pointer()
                                .child(if self.show_all {
                                    "Hide older".to_string()
                                } else {
                                    format!("Show {older} older")
                                })
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.show_all = !this.show_all;
                                    cx.notify();
                                })),
                        )
                    }),
            )
    }

    /// One session: status icon (with its color), title, then unread count or age.
    fn session_row(&self, i: usize, cx: &mut Context<Self>) -> Stateful<Div> {
        let task = &self.tasks[i];
        let (icon, label) = task.state.label();
        let unread = self.unread(task);
        let id = task.id.clone();
        div()
            .id(("task", i))
            .flex()
            .items_center()
            .gap_2()
            .px_2()
            .py_1p5()
            .rounded_md()
            .cursor_pointer()
            .hover(|s| s.bg(rgb(HOVER)))
            .child(
                div()
                    .flex_none()
                    .w(px(16.))
                    .text_sm()
                    .text_color(rgb(state_color(task.state)))
                    .child(icon),
            )
            .child(
                line(task.title.clone())
                    .flex_1()
                    .text_sm()
                    .when(unread > 0, |d| d.font_weight(FontWeight::SEMIBOLD))
                    .text_color(rgb(if unread > 0 { TEXT } else { 0xc4c4cc })),
            )
            .child(if unread > 0 {
                badge(unread, ACCENT)
            } else if task.state.is_active() {
                div()
                    .flex_none()
                    .text_xs()
                    .text_color(rgb(ACCENT))
                    .child(label)
            } else {
                div()
                    .flex_none()
                    .text_xs()
                    .text_color(rgb(0x6b6b78))
                    .child(ago(task.updated_at))
            })
            .on_click(cx.listener(move |this, _, _, cx| {
                this.open_task(id.clone());
                cx.notify();
            }))
    }

    fn render_task(&self, task: &Task, cx: &mut Context<Self>) -> Div {
        let (icon, label) = task.state.label();
        let agent = self
            .providers
            .iter()
            .find(|p| p.id() == task.provider)
            .map_or("Agent", |p| p.name());
        // Slack-style: avatar, name and time, then the text. Tool steps and other
        // events are quiet lines aligned with the text.
        let messages = task.messages.iter().rev().take(150).rev().map(|m| {
            if m.role == Role::Event {
                return div()
                    .pl(px(36.))
                    .pr_2()
                    .text_xs()
                    .text_color(rgb(0x6b6b78))
                    .child(m.text.clone());
            }
            let (who, key) = match m.role {
                Role::User => ("You", "you"),
                Role::Agent => (agent, "agent"),
                _ => ("Error", "error"),
            };
            div()
                .flex()
                .gap_2()
                .px_1()
                .pt_1()
                .child(avatar(who, key, 26.))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(rgb(match m.role {
                                            Role::Error => ERROR,
                                            _ => TEXT,
                                        }))
                                        .child(who),
                                )
                                .child(div().text_xs().text_color(rgb(0x6b6b78)).child(ago(m.at))),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(if m.role == Role::Error {
                                    ERROR
                                } else {
                                    0xd4d4d8
                                }))
                                .child(m.text.clone()),
                        ),
                )
        });
        let color = state_color(task.state);
        div()
            .flex_1()
            .flex()
            .flex_col()
            .gap_2()
            .min_h_0()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        button("back", "← Tasks", MUTED)
                            .on_click(cx.listener(|this, _, w, cx| this.back(&Back, w, cx))),
                    )
                    .when(task.state.is_active(), |d| {
                        d.child(
                            button("stop", "■ Stop", ERROR)
                                .on_click(cx.listener(|this, _, w, cx| this.stop(&StopRun, w, cx))),
                        )
                    }),
            )
            .child(
                div()
                    .text_base()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(task.title.clone()),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .min_w_0()
                    .child(pill(format!("{icon} {label}"), color))
                    .child(
                        line(format!(
                            "{} · {} · run {}{}",
                            agent,
                            short_dir(&task.project_dir),
                            task.run_count,
                            task.session_ref
                                .as_ref()
                                .map(|s| format!(" · {}", s.chars().take(8).collect::<String>()))
                                .unwrap_or_default()
                        ))
                        .flex_1()
                        .text_xs()
                        .text_color(rgb(MUTED)),
                    ),
            )
            .child(context_card(task))
            .when_some(task.pending_approval.clone(), |d, approval| {
                d.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .p_3()
                        .rounded_lg()
                        .bg(rgba((WARN << 8) | 0x12))
                        .border_1()
                        .border_color(rgb(WARN))
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(rgb(WARN))
                                .child("⚠ MUSE ASKS PERMISSION"),
                        )
                        .child(div().text_sm().child(approval.summary.clone()))
                        .child(match approval.answered {
                            Some(allow) => div().text_xs().text_color(rgb(MUTED)).child(format!(
                                "Sent: {}. Waiting for Muse to confirm.",
                                if allow { "allow once" } else { "reject" }
                            )),
                            None => div()
                                .flex()
                                .gap_2()
                                .child(primary("approve", "Allow once", WARN).on_click(
                                    cx.listener(|this, _, w, cx| this.approve(&Approve, w, cx)),
                                ))
                                .child(button("deny", "Reject", ERROR).on_click(
                                    cx.listener(|this, _, w, cx| this.deny(&Deny, w, cx)),
                                )),
                        }),
                )
            })
            .child(
                div()
                    .id("messages")
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll)
                    .children(messages),
            )
    }
}

/// Goal, recent changes, outcome and next action, so a task makes sense at a glance.
fn context_card(task: &Task) -> Div {
    let card = task.card();
    let row = |label: &str, text: String, color: u32| {
        div()
            .flex()
            .gap_2()
            .child(
                div()
                    .w(px(68.))
                    .flex_none()
                    .text_xs()
                    .text_color(rgb(0x6b6b78))
                    .child(label.to_uppercase()),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .text_xs()
                    .text_color(rgb(color))
                    .child(text),
            )
    };
    let changes = if card.changes.is_empty() {
        "None recorded yet.".to_string()
    } else if card.more_changes > 0 {
        format!("{} (+{} more)", card.changes.join(", "), card.more_changes)
    } else {
        card.changes.join(", ")
    };
    div()
        .flex()
        .flex_col()
        .gap_1p5()
        .p_3()
        .rounded_lg()
        .bg(rgb(PANEL))
        .border_1()
        .border_color(rgb(BORDER))
        .child(row("Goal", card.goal, TEXT))
        .child(row("Changed", changes, TEXT))
        .child(row("Outcome", card.outcome, TEXT))
        .child(row("Next", card.next.to_string(), ACCENT))
}

fn short_dir(dir: &str) -> String {
    match std::env::var("HOME") {
        Ok(home) if dir.starts_with(&home) => format!("~{}", &dir[home.len()..]),
        _ => dir.to_string(),
    }
}

fn ago(at: u64) -> String {
    let secs = now().saturating_sub(at);
    match secs {
        0..60 => "now".into(),
        60..3600 => format!("{}m", secs / 60),
        3600..86400 => format!("{}h", secs / 3600),
        _ => format!("{}d", secs / 86400),
    }
}

impl Render for Sidebar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if std::mem::take(&mut self.focus_search) {
            window.focus(&self.input.focus_handle(cx));
        }
        let hint = if self.selected.is_some() {
            "Reply to Muse…"
        } else {
            "Search, open an app, or ask Muse…"
        };
        self.input.update(cx, |input, _| {
            input.set_placeholder(hint);
        });
        if !self.query.is_empty() {
            // Tasks and windows change under an open search; keep indexes current.
            self.refresh_hits();
        }
        // What is on screen counts as read.
        if let Some(task) = self
            .selected
            .as_ref()
            .and_then(|id| self.tasks.iter().find(|t| &t.id == id))
        {
            self.seen.insert(task.id.clone(), agent_messages(task));
        }
        let bottom = self.placement == windows::Placement::Bottom;
        let selected = self
            .selected
            .as_ref()
            .and_then(|id| self.tasks.iter().find(|t| &t.id == id))
            .cloned();
        let attention = self
            .tasks
            .iter()
            .filter(|t| {
                t.pending_approval
                    .as_ref()
                    .is_some_and(|a| a.answered.is_none())
            })
            .count();
        let running = self
            .tasks
            .iter()
            .filter(|t| t.state == TaskState::Running)
            .count();
        div()
            .key_context("Sidebar")
            .size_full()
            .flex()
            .flex_col()
            .gap_2()
            .when(bottom, |d| d.px_3().py_2())
            .when(!bottom, |d| d.p_3())
            .bg(rgb(BG))
            .text_color(rgb(TEXT))
            .on_action(cx.listener(Self::submit))
            .on_action(cx.listener(Self::back))
            .on_action(cx.listener(Self::stop))
            .on_action(cx.listener(Self::approve))
            .on_action(cx.listener(Self::deny))
            .on_action(cx.listener(Self::search_go))
            .on_action(cx.listener(Self::search_next))
            .on_action(cx.listener(Self::search_prev))
            .on_action(cx.listener(Self::search_clear))
            .on_action(cx.listener(Self::focus_search_box))
            .on_action(cx.listener(Self::tile))
            .on_action(cx.listener(Self::restore_layout))
            .on_action(cx.listener(Self::show_help))
            .on_key_down(cx.listener(Self::key_down))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .pb_2()
                    .border_b_1()
                    .border_color(rgb(BORDER))
                    .child(div().size(px(8.)).rounded_full().bg(rgb(ACCENT)))
                    .child(
                        div()
                            .flex_1()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("DA Workstation"),
                    )
                    .when(attention > 0, |d| {
                        d.child(pill(format!("⚠ {attention} need you"), WARN))
                    })
                    .child(pill(
                        format!("◐ {running} running"),
                        if running > 0 { ACCENT } else { MUTED },
                    ))
                    .child(
                        div()
                            .id("help")
                            .flex_none()
                            .size(px(24.))
                            .rounded_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .border_1()
                            .border_color(rgb(BORDER))
                            .text_sm()
                            .text_color(rgb(MUTED))
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(HOVER)).text_color(rgb(TEXT)))
                            .child("?")
                            .on_click(cx.listener(|this, _, _, cx| this.open_help(cx))),
                    )
            )
            .when_some(self.status.clone(), |d, (text, failed)| {
                d.child(
                    div()
                        .id("status")
                        .px_2()
                        .text_xs()
                        .text_color(rgb(if failed { ERROR } else { MUTED }))
                        .cursor_pointer()
                        .child(text)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.status = None;
                            cx.notify();
                        })),
                )
            })
            .map(|d| {
                let body = match &selected {
                    _ if !self.query.trim().is_empty() => self.render_results(cx),
                    Some(task) => self.render_task(task, cx),
                    None if bottom => div()
                        .flex_1()
                        .flex()
                        .items_end()
                        .px_2()
                        .text_sm()
                        .text_color(rgb(MUTED))
                        .child("Pick a session or an app, or type below: search, open an app, or ask Muse."),
                    None => self.render_home(cx),
                };
                let composer = Some(self.render_composer(selected.is_some(), cx));
                if bottom {
                    d.child(
                        div()
                            .flex_1()
                            .min_h_0()
                            .flex()
                            .gap_3()
                            .child(
                                div()
                                    .w(px(300.))
                                    .flex_none()
                                    .flex()
                                    .flex_col()
                                    .min_h_0()
                                    .child(self.render_home(cx)),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .min_h_0()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    // The bar is short: the chat or results scroll above the box.
                                    .child(
                                        div()
                                            .id("middle")
                                            .flex_1()
                                            .min_h_0()
                                            .flex()
                                            .flex_col()
                                            .overflow_y_scroll()
                                            .child(body),
                                    )
                                    .children(composer),
                            )
                    )
                } else {
                    d.child(body).children(composer)
                }
            })
    }
}

pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        // Choices under the typing box.
        KeyBinding::new("down", SearchNext, Some("Sidebar")),
        KeyBinding::new("up", SearchPrev, Some("Sidebar")),
        KeyBinding::new("ctrl-k", FocusSearch, Some("Sidebar")),
        KeyBinding::new("ctrl-/", ShowHelp, Some("Sidebar")),
        KeyBinding::new("ctrl-t", Tile, Some("Sidebar")),
        KeyBinding::new("ctrl-r", RestoreLayout, Some("Sidebar")),
        KeyBinding::new("enter", Submit, Some("Sidebar")),
        KeyBinding::new("escape", Back, Some("Sidebar")),
        KeyBinding::new("ctrl-.", StopRun, Some("Sidebar")),
    ]);
}

/// Open the sidebar window, or raise the one that is already open. With a task id,
/// the sidebar shows that task.
pub fn run(open: Option<&str>) {
    if let Some(id) = open {
        if let Err(error) = Store::open().and_then(|store| store.request_open(id)) {
            eprintln!("{error}");
        }
    }
    if windows::place_sidebar(true).unwrap_or(false) {
        println!("Sidebar already open; raised it.");
        return;
    }
    Application::new().run(|cx: &mut App| {
        TextInput::bind_keys(cx);
        bind_keys(cx);
        let bounds = Bounds::new(point(px(0.), px(0.)), size(px(380.), px(900.)));
        let window = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                app_id: Some(APP_ID.into()),
                // Low enough for the bottom bar (300 px).
                window_min_size: Some(size(px(320.), px(240.))),
                ..Default::default()
            },
            |window, cx| {
                window.set_window_title("DA Workstation — Sidebar");
                let view = cx.new(Sidebar::new);
                window.focus(&view.read(cx).input().focus_handle(cx));
                view
            },
        );
        if let Err(error) = window {
            eprintln!("Could not open sidebar: {error}");
            cx.quit();
            return;
        }
        // Wayland apps cannot place themselves; the GNOME extension docks the window.
        cx.background_executor()
            .spawn(async {
                for _ in 0..20 {
                    if windows::place_sidebar(false).unwrap_or(false) {
                        // GNOME can place a new window after this first move
                        // (seen at login); dock it again once it has settled.
                        std::thread::sleep(Duration::from_millis(1500));
                        let _ = windows::place_sidebar(false);
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(250));
                }
            })
            .detach();
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
    use super::parse_new_task;

    #[test]
    fn at_prefix_sets_the_project_folder() {
        assert_eq!(
            parse_new_task("@/tmp/x run tests", "/home"),
            ("/tmp/x".into(), "run tests".into())
        );
        assert_eq!(
            parse_new_task("fix the build", "/home"),
            ("/home".into(), "fix the build".into())
        );
        assert_eq!(
            parse_new_task("@/tmp/x", "/home"),
            ("/tmp/x".into(), String::new())
        );
    }
}
