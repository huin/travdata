use std::sync::mpsc;

/// Sends `value` on `sender`, logging a warning if it fails to send.
pub fn send_or_log_warning<T>(sender: &mpsc::SyncSender<T>, description: &str, value: T) {
    if sender.send(value).is_err() {
        log::warn!("Failed to send {} on channel.", description);
    }
}
