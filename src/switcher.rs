//! The sidebar's search ("jump to"): agent sessions, open apps and installed apps
//! that match the query, in that order.

use crate::activity::AppGroup;
use crate::apps::{self, DesktopApp};
use crate::tasks::{self, Task};

#[derive(Clone, Debug, PartialEq)]
pub enum Hit {
    /// Index into the task list.
    Session(usize),
    /// An open app, by its key from `activity::app_key`.
    OpenApp(String),
    /// Index into the installed apps: Enter focuses it, or launches it.
    Installed(usize),
    /// A workstation command, typed as `/name`.
    Command(Command),
}

/// Commands for the typing box. Type `/` to list them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Tile,
    Restore,
    BottomBar,
    SidePanel,
    AutoTileOn,
    AutoTileOff,
    Help,
}

impl Command {
    pub const ALL: [Command; 7] = [
        Command::Tile,
        Command::Restore,
        Command::BottomBar,
        Command::SidePanel,
        Command::AutoTileOn,
        Command::AutoTileOff,
        Command::Help,
    ];

    /// The name typed after `/`, and what it does.
    pub fn text(self) -> (&'static str, &'static str) {
        match self {
            Command::Tile => ("tile", "One big window, the others as cards (Ctrl+Super+T)"),
            Command::Restore => ("restore", "Put tiled windows back (Ctrl+Super+R)"),
            Command::BottomBar => ("bottom", "Dock this panel along the bottom"),
            Command::SidePanel => ("side", "Dock this panel on the left"),
            Command::AutoTileOn => ("autotile on", "Tile every new window automatically"),
            Command::AutoTileOff => ("autotile off", "Tile only apps opened from here"),
            Command::Help => (
                "help",
                "Open the guide in your browser: every shortcut and command (Ctrl+/)",
            ),
        }
    }
}

/// One choice under the typing box: open a match, or start a Muse task with the text.
#[derive(Clone, Debug, PartialEq)]
pub enum Row {
    Ask,
    Hit(Hit),
}

/// Choices for the typed text. A short query (three words or fewer, no `@folder`)
/// that matches something puts the matches first, so "chrome" + Enter opens Chrome;
/// a sentence puts "Ask Muse" first, so Enter starts a task.
pub fn rows(hits: &[Hit], query: &str) -> Vec<Row> {
    let query = query.trim();
    if query.is_empty() {
        return Vec::new();
    }
    if query.starts_with('/') {
        // Commands only; a slash is never a task for Muse.
        return hits.iter().cloned().map(Row::Hit).collect();
    }
    let ask_first =
        hits.is_empty() || query.starts_with('@') || query.split_whitespace().count() > 3;
    let matches = hits.iter().cloned().map(Row::Hit);
    if ask_first {
        std::iter::once(Row::Ask).chain(matches).collect()
    } else {
        matches.chain(std::iter::once(Row::Ask)).collect()
    }
}

const SESSIONS: usize = 4;
const OPEN: usize = 4;
const INSTALLED: usize = 6;

/// Matches for a non-empty query. Installed apps that are already open are left out.
pub fn search(
    tasks: &[Task],
    groups: &[AppGroup],
    installed: &[DesktopApp],
    query: &str,
) -> Vec<Hit> {
    let query = query.trim();
    if query.is_empty() {
        return Vec::new();
    }
    if let Some(name) = query.strip_prefix('/') {
        let name = name.trim().to_lowercase();
        return Command::ALL
            .into_iter()
            .filter(|c| {
                let full = c.text().0;
                full.starts_with(&name) || full.split(' ').any(|word| word.starts_with(&name))
            })
            .map(Hit::Command)
            .collect();
    }
    let words: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
    let mut hits: Vec<Hit> = tasks::bar_matches(tasks, query, SESSIONS)
        .into_iter()
        .map(Hit::Session)
        .collect();
    hits.extend(
        groups
            .iter()
            .filter(|g| {
                let text = std::iter::once(g.name.as_str())
                    .chain(g.windows.iter().map(|w| w.title.as_str()))
                    .collect::<Vec<_>>()
                    .join(" ")
                    .to_lowercase();
                words.iter().all(|w| text.contains(w.as_str()))
            })
            .take(OPEN)
            .map(|g| Hit::OpenApp(g.key.clone())),
    );
    hits.extend(
        apps::matches(installed, query)
            .into_iter()
            .filter(|&i| !groups.iter().any(|g| g.key == installed[i].id))
            .take(INSTALLED)
            .map(Hit::Installed),
    );
    hits
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::windows::OpenWindow;

    fn app(id: &str, name: &str) -> DesktopApp {
        DesktopApp {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            search_text: name.to_lowercase(),
        }
    }

    #[test]
    fn sessions_then_open_apps_then_installed_apps() {
        let tasks = [Task::new("Fix the chrome extension test", "/tmp", "muse")];
        let window = OpenWindow {
            id: 1,
            title: "Inbox".into(),
            app: "Google Chrome".into(),
            app_id: "google-chrome.desktop".into(),
            workspace: 0,
            focused: false,
            minimized: false,
            attention: false,
        };
        let groups = crate::activity::group(&[window]);
        let installed = [
            app("google-chrome.desktop", "Google Chrome"),
            app("chromium.desktop", "Chromium"),
        ];
        assert_eq!(
            search(&tasks, &groups, &installed, "chrom"),
            [
                Hit::Session(0),
                Hit::OpenApp("google-chrome.desktop".into()),
                Hit::Installed(1)
            ]
        );
        assert_eq!(
            search(&tasks, &groups, &installed, "chrome test"),
            [Hit::Session(0)]
        );
        assert_eq!(
            search(&tasks, &groups, &installed, "inbox")[0],
            Hit::OpenApp("google-chrome.desktop".into())
        );
        assert!(search(&tasks, &groups, &installed, "  ").is_empty());
    }

    #[test]
    fn short_matches_open_first_and_sentences_ask_muse_first() {
        let hits = [Hit::Installed(0)];
        assert_eq!(
            rows(&hits, "chrome"),
            [Row::Hit(Hit::Installed(0)), Row::Ask]
        );
        assert_eq!(
            rows(&hits, "open chrome and tile it big"),
            [Row::Ask, Row::Hit(Hit::Installed(0))]
        );
        assert_eq!(rows(&hits, "@~/p run tests")[0], Row::Ask);
        assert_eq!(rows(&[], "chrome"), [Row::Ask]);
        assert!(rows(&hits, " ").is_empty());
    }

    #[test]
    fn slash_lists_commands_and_never_asks_muse() {
        let all = search(&[], &[], &[], "/");
        assert_eq!(all.len(), Command::ALL.len());
        assert_eq!(search(&[], &[], &[], "/til"), [Hit::Command(Command::Tile)]);
        assert_eq!(
            search(&[], &[], &[], "/autotile o"),
            [
                Hit::Command(Command::AutoTileOn),
                Hit::Command(Command::AutoTileOff)
            ]
        );
        let hits = search(&[], &[], &[], "/bottom");
        assert_eq!(
            rows(&hits, "/bottom"),
            [Row::Hit(Hit::Command(Command::BottomBar))]
        );
        assert!(rows(&[], "/nothing").is_empty());
    }
}
