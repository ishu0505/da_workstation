//! Provider boundary. Tasks name a provider by id; only adapters know provider details.

use std::sync::mpsc::Sender;

pub mod muse;
pub mod muse_serve;

#[derive(Clone, Debug, PartialEq)]
pub enum ProviderEvent {
    /// The provider accepted the run; `session_ref` identifies its session or process.
    Started {
        session_ref: String,
    },
    Output(String),
    /// Diagnostic text from the provider, shown apart from the agent's answer.
    Diagnostic(String),
    /// The agent waits for the user's decision on one action.
    NeedsApproval {
        id: String,
        summary: String,
    },
    /// The agent recorded a decision for that request (the user's or its own).
    ApprovalResolved {
        id: String,
        summary: String,
    },
    /// Files the agent changed, as absolute paths (one tool call's worth).
    FilesChanged(Vec<String>),
    /// The run ended. `ok` is the process/session outcome, not proof the work is correct.
    Finished {
        ok: bool,
        summary: String,
    },
}

/// Events are tagged with the task id so one channel serves every run.
pub type EventSink = Sender<(String, ProviderEvent)>;

pub trait Run: Send {
    fn answer_approval(&mut self, id: &str, allow: bool) -> Result<(), String>;
    fn cancel(&mut self) -> Result<(), String>;
}

pub trait Provider: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    /// Why the provider cannot run now, in words the user can act on.
    fn unavailable_reason(&self) -> Option<String>;
    /// True when a run can continue this session with its context, so a follow-up
    /// needs only the new instruction.
    fn resumes(&self, _session_ref: &str) -> bool {
        false
    }
    /// `session` is the task's earlier session; adapters continue it when `resumes` says so.
    fn start(
        &self,
        task_id: &str,
        project_dir: &str,
        prompt: &str,
        session: Option<&str>,
        sink: EventSink,
    ) -> Result<Box<dyn Run>, String>;
}

/// The providers this build ships. Muse is first; others plug in here later.
pub fn all() -> Vec<Box<dyn Provider>> {
    vec![Box::new(muse::Muse::default())]
}
