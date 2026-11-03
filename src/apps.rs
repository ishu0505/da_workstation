//! Linux desktop discovery and launch boundary. Search text never becomes a shell command.

use gio::prelude::*;

#[derive(Clone, Debug)]
pub struct DesktopApp {
    pub id: String,
    pub name: String,
    pub description: String,
    pub search_text: String,
}

pub fn discover() -> Vec<DesktopApp> {
    let mut apps: Vec<_> = gio::AppInfo::all()
        .into_iter()
        .filter(|app| app.should_show())
        .filter_map(|app| {
            let id = app.id()?.to_string();
            let name = app.display_name().to_string();
            let description = app.description().map(|s| s.to_string()).unwrap_or_default();
            let desktop = gio::DesktopAppInfo::new(&id);
            let keywords = desktop
                .as_ref()
                .map(|d| d.keywords().join(" "))
                .unwrap_or_default();
            let categories = desktop
                .as_ref()
                .and_then(|d| d.categories())
                .unwrap_or_default();
            let search_text =
                format!("{id} {name} {description} {keywords} {categories}").to_lowercase();
            Some(DesktopApp {
                id,
                name,
                description,
                search_text,
            })
        })
        .collect();
    apps.sort_by_key(|app| app.name.to_lowercase());
    apps.dedup_by(|a, b| a.id == b.id);
    apps
}

pub fn matches(apps: &[DesktopApp], query: &str) -> Vec<usize> {
    let query = query.trim().to_lowercase();
    let query = query.strip_prefix("use-w ").unwrap_or(&query).trim();
    let mut matches: Vec<_> = apps
        .iter()
        .enumerate()
        .filter_map(|(index, app)| {
            let name = app.name.to_lowercase();
            let score = if query.is_empty() || name == query {
                0
            } else if name.starts_with(query) {
                1
            } else if name.contains(query) {
                2
            } else if query
                .split_whitespace()
                .all(|word| app.search_text.contains(word))
            {
                3
            } else {
                return None;
            };
            Some((score, index))
        })
        .collect();
    matches.sort_by_key(|&(score, index)| (score, index));
    matches.into_iter().map(|(_, index)| index).collect()
}

/// Acceptance means the desktop accepted the request, not that a window appeared.
pub fn launch(id: &str) -> Result<(), String> {
    let app = gio::DesktopAppInfo::new(id).ok_or_else(|| {
        "This application is no longer installed. Restart to refresh the list.".to_string()
    })?;
    app.launch(&[], None::<&gio::AppLaunchContext>)
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Vec<DesktopApp> {
        [
            ("browser.desktop", "Browser", "browser web internet"),
            (
                "com.mitchellh.ghostty.desktop",
                "Ghostty",
                "ghostty terminal emulator",
            ),
            (
                "org.gnome.Terminal.desktop",
                "Terminal",
                "terminal emulator",
            ),
        ]
        .into_iter()
        .map(|(id, name, search)| DesktopApp {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            search_text: search.into(),
        })
        .collect()
    }

    #[test]
    fn terminal_alias_matches_ghostty_but_exact_name_wins() {
        assert_eq!(matches(&fixture(), " terminal "), vec![2, 1]);
    }

    #[test]
    fn command_prefix_and_case_are_normalized() {
        assert_eq!(matches(&fixture(), "USE-W GHOST"), vec![1]);
    }

    #[test]
    fn unknown_or_shell_text_cannot_be_launched_as_a_search_result() {
        assert!(matches(&fixture(), "ghostty; touch /tmp/unwanted").is_empty());
        assert!(matches(&fixture(), "missing app").is_empty());
        assert_eq!(matches(&fixture(), ""), vec![0, 1, 2]);
    }
}
