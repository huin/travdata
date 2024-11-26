use std::path::PathBuf;

/// Returns a possible path to the Tabula JAR file (as a [String] for CLI parsing), assuming that
/// the process is running as part of a distribution.
pub fn tabula_jar() -> Option<String> {
    form_path("tabula.jar").and_then(|p| p.to_str().map(str::to_owned))
}

/// Returns a possible path to the configuration ZIP file, assuming that the process is running as
/// part of a distribution.
pub fn config_zip() -> Option<PathBuf> {
    form_path("config.zip")
}

fn form_path(file_name: &str) -> Option<PathBuf> {
    let mut exec_path = std::env::current_exe().ok()?;
    exec_path.set_file_name(file_name);
    if !exec_path.is_file() {
        return None;
    }
    Some(exec_path)
}
