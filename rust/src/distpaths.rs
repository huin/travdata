/// Returns a possible path to the Tabula JAR file (as a [String] for CLI parsing), assuming that
/// the process is running as part of a distribution.
pub fn tabula_jar() -> Option<String> {
    let mut exec_path = std::env::current_exe().ok()?;
    exec_path.set_file_name("tabula.jar");
    if !exec_path.is_file() {
        return None;
    }
    exec_path.to_str().map(str::to_string)
}
