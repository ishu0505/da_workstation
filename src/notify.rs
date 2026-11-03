//! Desktop notifications through the freedesktop D-Bus interface, with a click action.

use gio::glib::{self, variant::ToVariant};
use std::collections::HashMap;
use std::sync::mpsc::Sender;

const NAME: &str = "org.freedesktop.Notifications";
const PATH: &str = "/org/freedesktop/Notifications";

/// Show a notification; returns its id so a click can be matched to a task.
pub fn send(summary: &str, body: &str) -> Result<u32, String> {
    let connection = gio::bus_get_sync(gio::BusType::Session, None::<&gio::Cancellable>)
        .map_err(|e| e.to_string())?;
    let hints: HashMap<String, glib::Variant> = HashMap::new();
    let params = (
        "DA Workstation",
        0u32,
        "",
        summary,
        body,
        vec!["default", "Open task"],
        hints,
        -1i32,
    )
        .to_variant();
    connection
        .call_sync(
            Some(NAME),
            PATH,
            NAME,
            "Notify",
            Some(&params),
            None,
            gio::DBusCallFlags::NONE,
            2000,
            None::<&gio::Cancellable>,
        )
        .map_err(|e| e.to_string())?
        .get::<(u32,)>()
        .map(|(id,)| id)
        .ok_or_else(|| "Unexpected notification reply".into())
}

/// Forward every clicked notification id. GPUI does not run a GLib loop, so this owns one.
pub fn listen_for_clicks(clicked: Sender<u32>) {
    std::thread::spawn(move || {
        let context = glib::MainContext::new();
        let _ = context.with_thread_default(|| {
            let Ok(connection) =
                gio::bus_get_sync(gio::BusType::Session, None::<&gio::Cancellable>)
            else {
                return;
            };
            let _subscription = connection.subscribe_to_signal(
                Some(NAME),
                Some(NAME),
                Some("ActionInvoked"),
                Some(PATH),
                None,
                gio::DBusSignalFlags::NONE,
                move |signal| {
                    if let Some((id, _action)) = signal.parameters.get::<(u32, String)>() {
                        let _ = clicked.send(id);
                    }
                },
            );
            glib::MainLoop::new(Some(&context), false).run();
        });
    });
}
