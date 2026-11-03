//! Task model and its store: one JSON file per task, written atomically.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskState {
    Ready,
    Running,
    NeedsPermission,
    Done,
    Failed,
    Stopped,
    /// The workstation restarted during a run; the real outcome is unknown.
    Interrupted,
}

impl TaskState {
    /// Icon and text, so status never depends on color alone.
    pub fn label(self) -> (&'static str, &'static str) {
        match self {
            Self::Ready => ("○", "Ready"),
            Self::Running => ("◐", "Running"),
            Self::NeedsPermission => ("⚠", "Needs permission"),
            Self::Done => ("✓", "Done"),
            Self::Failed => ("✕", "Failed"),
            Self::Stopped => ("■", "Stopped"),
            Self::Interrupted => ("?", "Interrupted"),
        }
    }

    pub fn is_active(self) -> bool {
        matches!(self, Self::Running | Self::NeedsPermission)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    User,
    Agent,
    Event,
    Error,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub at: u64,
    pub role: Role,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Approval {
    pub id: String,
    pub summary: String,
    /// The user's choice once sent (true: allow). The request stays shown until the
    /// agent confirms it.
    #[serde(default)]
    pub answered: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub goal: String,
    pub project_dir: String,
    pub provider: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub state: TaskState,
    #[serde(default)]
    pub session_ref: Option<String>,
    #[serde(default)]
    pub run_count: u32,
    #[serde(default)]
    pub pending_approval: Option<Approval>,
    #[serde(default)]
    pub messages: Vec<Message>,
    /// Files the agent changed over the whole task, oldest first, no repeats.
    #[serde(default)]
    pub changed_files: Vec<String>,
    /// Files changed in the latest run.
    #[serde(default)]
    pub run_files: Vec<String>,
}

/// The summary at the top of a task: what it is for, what changed, how it went, what next.
#[derive(Debug, PartialEq)]
pub struct Card {
    pub goal: String,
    pub changes: Vec<String>,
    pub more_changes: usize,
    pub outcome: String,
    pub next: &'static str,
}

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default()
}

impl Task {
    pub fn new(goal: &str, project_dir: &str, provider: &str) -> Self {
        let at = now();
        let title: String = goal.chars().take(48).collect();
        Self {
            id: format!("{at}-{:04x}", std::process::id() as u16 ^ (at as u16)),
            title: if goal.chars().count() > 48 {
                format!("{title}…")
            } else {
                title
            },
            goal: goal.into(),
            project_dir: project_dir.into(),
            provider: provider.into(),
            created_at: at,
            updated_at: at,
            state: TaskState::Ready,
            session_ref: None,
            run_count: 0,
            pending_approval: None,
            messages: Vec::new(),
            changed_files: Vec::new(),
            run_files: Vec::new(),
        }
    }

    pub fn push(&mut self, role: Role, text: impl Into<String>) {
        self.updated_at = now();
        self.messages.push(Message {
            at: self.updated_at,
            role,
            text: text.into(),
        });
    }

    /// Change state; returns true only for a real transition, so alerts fire once.
    pub fn transition(&mut self, state: TaskState) -> bool {
        if self.state == state {
            return false;
        }
        self.state = state;
        self.updated_at = now();
        true
    }

    /// Record files the agent changed, once each, in the latest run and the task.
    pub fn record_files(&mut self, files: Vec<String>) {
        for file in files {
            if !self.run_files.contains(&file) {
                self.run_files.push(file.clone());
            }
            self.changed_files.retain(|f| f != &file);
            self.changed_files.push(file);
        }
    }

    /// Changed files relative to the project folder, most recent first.
    pub fn card(&self) -> Card {
        const SHOWN: usize = 4;
        let base = format!("{}/", self.project_dir.trim_end_matches('/'));
        let changes: Vec<String> = self
            .changed_files
            .iter()
            .rev()
            .take(SHOWN)
            .map(|f| f.strip_prefix(&base).unwrap_or(f).to_string())
            .collect();
        let last_answer = self
            .messages
            .iter()
            .rev()
            .find(|m| m.role == Role::Agent)
            .and_then(|m| m.text.lines().find(|l| !l.trim().is_empty()))
            .map(|l| {
                let l = l.trim();
                if l.chars().count() > 140 {
                    format!("{}…", l.chars().take(140).collect::<String>())
                } else {
                    l.to_string()
                }
            });
        let (icon, label) = self.state.label();
        let outcome = match (self.state, last_answer) {
            (TaskState::Ready, _) => format!("{icon} {label}: not run yet."),
            (_, Some(answer)) => format!("{icon} {label}. Last answer: {answer}"),
            (_, None) => format!("{icon} {label}. No answer yet."),
        };
        let next = match self.state {
            TaskState::Ready => "Start the task.",
            TaskState::Running => "Wait for the agent, or Stop (Ctrl+.).",
            TaskState::NeedsPermission => "Allow once or Reject the request below.",
            TaskState::Done => "Check the changes, then close or send a follow-up.",
            TaskState::Failed => "Read the error below, then send a follow-up to retry.",
            TaskState::Stopped => "Send a follow-up to continue.",
            TaskState::Interrupted => {
                "The app restarted during a run. Send a follow-up to continue."
            }
        };
        Card {
            goal: self.goal.clone(),
            changes,
            more_changes: self.changed_files.len().saturating_sub(SHOWN),
            outcome,
            next,
        }
    }

    /// A portable handoff for a follow-up run: goal, recent conversation, new instruction.
    pub fn follow_up_prompt(&self, instruction: &str) -> String {
        let recent: Vec<_> = self
            .messages
            .iter()
            .filter(|m| matches!(m.role, Role::User | Role::Agent))
            .rev()
            .take(12)
            .collect();
        let mut prompt = format!("Task goal: {}\n\nRecent conversation:\n", self.goal);
        for message in recent.into_iter().rev() {
            let who = if message.role == Role::User {
                "User"
            } else {
                "Agent"
            };
            prompt.push_str(&format!("{who}: {}\n", message.text));
        }
        prompt.push_str(&format!("\nNew instruction: {instruction}"));
        prompt
    }
}

/// The default project folder for a new task: the newest task's folder, if it still exists.
pub fn last_project_dir(tasks: &[Task]) -> Option<String> {
    tasks
        .iter()
        .max_by_key(|t| t.created_at)
        .map(|t| t.project_dir.clone())
        .filter(|dir| Path::new(dir).is_dir())
}

/// Tasks for the focus bar: with no query, active tasks then the newest; else matches on
/// every word of the query in the title or goal. At most `limit`.
pub fn bar_matches(tasks: &[Task], query: &str, limit: usize) -> Vec<usize> {
    let words: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
    let mut found: Vec<usize> = (0..tasks.len())
        .filter(|&i| {
            let text = format!("{} {}", tasks[i].title, tasks[i].goal).to_lowercase();
            words.iter().all(|w| text.contains(w.as_str()))
        })
        .collect();
    found.sort_by_key(|&i| {
        (
            !tasks[i].state.is_active(),
            std::cmp::Reverse(tasks[i].updated_at),
        )
    });
    found.truncate(limit);
    found
}

pub struct Store {
    dir: PathBuf,
}

impl Store {
    pub fn open() -> Result<Self, String> {
        let base = std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| Path::new(&h).join(".local/share")))
            .ok_or("No HOME or XDG_DATA_HOME set")?;
        Self::at(base.join("da-workstation/tasks"))
    }

    pub fn at(dir: PathBuf) -> Result<Self, String> {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Cannot create task folder {}: {e}", dir.display()))?;
        Ok(Self { dir })
    }

    /// Load every task, newest first. A run that was active is marked interrupted.
    /// Only the sidebar (which owns runs) may call this; others use `read_all`.
    pub fn load(&self) -> (Vec<Task>, Vec<String>) {
        let (mut tasks, problems) = self.read_all();
        for task in &mut tasks {
            if task.state.is_active() {
                task.transition(TaskState::Interrupted);
                task.pending_approval = None;
                task.push(
                    Role::Event,
                    "The workstation restarted during this run. Its real outcome is unknown; check the project folder, then send a follow-up.",
                );
                let _ = self.save(task);
            }
        }
        (tasks, problems)
    }

    /// Every task as saved, newest first, without changing anything.
    pub fn read_all(&self) -> (Vec<Task>, Vec<String>) {
        let mut tasks = Vec::new();
        let mut problems = Vec::new();
        let entries = match std::fs::read_dir(&self.dir) {
            Ok(entries) => entries,
            Err(e) => return (tasks, vec![format!("Cannot read tasks: {e}")]),
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_none_or(|ext| ext != "json") {
                continue;
            }
            match std::fs::read_to_string(&path)
                .map_err(|e| e.to_string())
                .and_then(|text| serde_json::from_str::<Task>(&text).map_err(|e| e.to_string()))
            {
                Ok(task) => tasks.push(task),
                Err(e) => problems.push(format!("{}: {e}", path.display())),
            }
        }
        tasks.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        (tasks, problems)
    }

    /// Ask the sidebar to open a task. The sidebar takes the request on its next tick.
    pub fn request_open(&self, task_id: &str) -> Result<(), String> {
        std::fs::write(self.open_request_path(), task_id)
            .map_err(|e| format!("Cannot ask the sidebar to open the task: {e}"))
    }

    /// The task the bar asked to open, once.
    pub fn take_open_request(&self) -> Option<String> {
        let path = self.open_request_path();
        let id = std::fs::read_to_string(&path).ok()?;
        let _ = std::fs::remove_file(&path);
        Some(id.trim().to_string()).filter(|id| !id.is_empty())
    }

    fn open_request_path(&self) -> PathBuf {
        self.dir.join("open-request")
    }

    pub fn save(&self, task: &Task) -> Result<(), String> {
        let path = self.dir.join(format!("{}.json", task.id));
        let tmp = path.with_extension("json.tmp");
        let text = serde_json::to_string_pretty(task).map_err(|e| e.to_string())?;
        std::fs::write(&tmp, text)
            .and_then(|()| std::fs::rename(&tmp, &path))
            .map_err(|e| format!("Cannot save task {}: {e}", task.id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_store() -> Store {
        static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("da-tasks-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        Store::at(dir).unwrap()
    }

    #[test]
    fn card_shows_recent_changes_outcome_and_next_step() {
        let mut task = Task::new("Fix the parser", "/work/app/", "muse");
        assert_eq!(task.card().next, "Start the task.");
        task.record_files(vec!["/work/app/a.rs".into(), "/work/app/b.rs".into()]);
        task.record_files(vec!["/work/app/a.rs".into(), "/elsewhere/c.txt".into()]);
        assert_eq!(task.run_files.len(), 3);
        task.push(Role::Agent, "\nFixed the parser.\nDetails follow.");
        task.transition(TaskState::Done);
        let card = task.card();
        assert_eq!(card.goal, "Fix the parser");
        assert_eq!(card.changes, ["/elsewhere/c.txt", "a.rs", "b.rs"]);
        assert_eq!(card.more_changes, 0);
        assert_eq!(card.outcome, "✓ Done. Last answer: Fixed the parser.");
        assert!(card.next.starts_with("Check the changes"));
    }

    #[test]
    fn bar_lists_active_tasks_first_and_matches_words() {
        let mut old = Task::new("Fix the parser tests", "/tmp", "muse");
        old.updated_at = 10;
        let mut running = Task::new("Write release notes", "/tmp", "muse");
        running.updated_at = 5;
        running.state = TaskState::Running;
        let mut new = Task::new("Summarize the readme", "/tmp", "muse");
        new.updated_at = 20;
        let tasks = [old, running, new];
        assert_eq!(bar_matches(&tasks, "", 3), [1, 2, 0]);
        assert_eq!(bar_matches(&tasks, "", 2), [1, 2]);
        assert_eq!(bar_matches(&tasks, "PARSER fix", 3), [0]);
        assert!(bar_matches(&tasks, "nothing", 3).is_empty());
    }

    #[test]
    fn open_requests_are_taken_once_and_last_folder_must_exist() {
        let store = temp_store();
        assert_eq!(store.take_open_request(), None);
        store.request_open("t1").unwrap();
        assert_eq!(store.take_open_request().as_deref(), Some("t1"));
        assert_eq!(store.take_open_request(), None);
        let mut a = Task::new("a", "/nonexistent/da-dir", "muse");
        a.created_at = 2;
        let mut b = Task::new("b", "/tmp", "muse");
        b.created_at = 1;
        assert_eq!(last_project_dir(&[b.clone()]).as_deref(), Some("/tmp"));
        assert_eq!(last_project_dir(&[a, b]), None);
    }

    #[test]
    fn transitions_fire_once() {
        let mut task = Task::new("Run tests", "/tmp", "muse");
        assert!(task.transition(TaskState::Running));
        assert!(!task.transition(TaskState::Running));
        assert!(task.transition(TaskState::Done));
    }

    #[test]
    fn restart_marks_active_runs_interrupted_and_keeps_history() {
        let store = temp_store();
        let mut task = Task::new("Compare two parsers", "/tmp", "muse");
        task.push(Role::User, "Compare two parsers");
        task.transition(TaskState::NeedsPermission);
        task.pending_approval = Some(Approval {
            id: "1".into(),
            summary: "run cargo".into(),
            answered: None,
        });
        store.save(&task).unwrap();
        let (tasks, problems) = store.load();
        assert!(problems.is_empty(), "{problems:?}");
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].state, TaskState::Interrupted);
        assert_eq!(tasks[0].pending_approval, None);
        assert_eq!(tasks[0].messages[0].text, "Compare two parsers");
        assert_eq!(tasks[0].messages.last().unwrap().role, Role::Event);
    }

    #[test]
    fn follow_up_carries_goal_and_recent_conversation() {
        let mut task = Task::new("Fix the build", "/tmp", "muse");
        task.push(Role::User, "Fix the build");
        task.push(Role::Agent, "Changed Cargo.toml");
        task.push(Role::Event, "noise");
        let prompt = task.follow_up_prompt("Now run the tests");
        assert!(prompt.contains("Task goal: Fix the build"));
        assert!(prompt.contains("Agent: Changed Cargo.toml"));
        assert!(!prompt.contains("noise"));
        assert!(prompt.ends_with("New instruction: Now run the tests"));
    }
}
