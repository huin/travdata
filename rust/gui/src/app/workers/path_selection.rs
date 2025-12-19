use crate::app::ddo;

pub fn pick_single_file<T>(
    dialog: rfd::FileDialog,
    sender: egui_inbox::UiInboxSender<T>,
    to_selection: impl FnOnce(Result<Option<ddo::PathSelection>, String>) -> T + Send + 'static,
) where
    T: std::fmt::Debug + Send + 'static,
{
    std::thread::spawn(move || {
        let msg: T = match dialog.pick_file() {
            Some(path) => match path.file_name() {
                Some(file_name) => {
                    let as_string = file_name.to_string_lossy().to_string();
                    to_selection(Ok(Some(ddo::PathSelection { path, as_string })))
                }
                // This should not happen for a picked file.
                None => to_selection(Err(format!("No filename in path {path:?}."))),
            },
            None => to_selection(Ok(None)),
        };

        if let Err(err) = sender.send(msg) {
            log::warn!("Failed to send file selection message: {err:?}");
        }
    });
}
