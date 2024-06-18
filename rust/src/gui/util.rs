use std::path::PathBuf;

use crate::filesio::IoType;

const NOT_SELECTED: &str = "<not selected>";

#[derive(Debug)]
pub struct SelectedFileIo {
    pub io_type: IoType,
    pub path: PathBuf,
}

impl SelectedFileIo {
    pub fn for_dir(path: PathBuf) -> Self {
        Self {
            io_type: IoType::Dir,
            path,
        }
    }
    pub fn for_zip(path: PathBuf) -> Self {
        Self {
            io_type: IoType::Zip,
            path,
        }
    }
}

pub fn format_opt_selected_file_io(opt_selected: &Option<SelectedFileIo>) -> String {
    match opt_selected {
        None => NOT_SELECTED.to_string(),
        Some(selected) => format!("{:?} {:?}", selected.io_type, selected.path),
    }
}

pub fn format_opt_path(path: &Option<PathBuf>) -> &str {
    match path {
        None => NOT_SELECTED,
        Some(path) => match path.to_str() {
            None => "<selected - cannot be displayed>",
            Some(path_str) => path_str,
        },
    }
}
