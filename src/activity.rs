//! Open apps for the typing box's search: windows grouped per app, and avatar text.

use crate::windows::OpenWindow;

/// One app and its windows, most recently used first.
#[derive(Clone, Debug, PartialEq)]
pub struct AppGroup {
    pub key: String,
    pub name: String,
    pub windows: Vec<OpenWindow>,
}

/// The app a window belongs to: its desktop id, else its app name.
pub fn app_key(window: &OpenWindow) -> String {
    if window.app_id.is_empty() {
        window.app.clone()
    } else {
        window.app_id.clone()
    }
}

/// Group windows by app, keeping the most recently used app first.
pub fn group(windows: &[OpenWindow]) -> Vec<AppGroup> {
    let mut groups: Vec<AppGroup> = Vec::new();
    for window in windows {
        let key = app_key(window);
        match groups.iter_mut().find(|g| g.key == key) {
            Some(group) => group.windows.push(window.clone()),
            None => groups.push(AppGroup {
                key,
                name: window.app.clone(),
                windows: vec![window.clone()],
            }),
        }
    }
    groups
}

/// Two-letter avatar text for an app name: "Google Chrome" gives "GC".
pub fn initials(name: &str) -> String {
    let words: Vec<&str> = name.split_whitespace().collect();
    let letters: String = match words.as_slice() {
        [] => "?".into(),
        [one] => one.chars().take(2).collect(),
        [first, second, ..] => first
            .chars()
            .take(1)
            .chain(second.chars().take(1))
            .collect(),
    };
    letters.to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn w(id: u64, app: &str, title: &str) -> OpenWindow {
        OpenWindow {
            id,
            title: title.into(),
            app: app.into(),
            app_id: format!("{}.desktop", app.to_lowercase()),
            workspace: 0,
            focused: false,
            minimized: false,
            attention: false,
        }
    }

    #[test]
    fn windows_group_per_app_in_recent_order() {
        let groups = group(&[
            w(1, "Chrome", "a"),
            w(2, "Ghostty", "b"),
            w(3, "Chrome", "c"),
        ]);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].name, "Chrome");
        assert_eq!(groups[0].windows.len(), 2);
        assert_eq!(groups[1].key, "ghostty.desktop");
    }

    #[test]
    fn initials_make_short_avatars() {
        assert_eq!(initials("Google Chrome"), "GC");
        assert_eq!(initials("ghostty"), "GH");
        assert_eq!(initials(""), "?");
    }
}
