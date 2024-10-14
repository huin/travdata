use std::path::PathBuf;

use relm4_components::save_dialog::SaveDialogMsg;

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

/// If the given `opt_selected` is a `Some(SelectedFileIo)` with the given
/// `SelectedFileIo.io_type == for_io_type`, returns a [SaveDialogMsg::SaveAs],
/// otherwise [SaveDialogMsg::Save].
pub fn save_dialog_msg(
    opt_selected: &Option<SelectedFileIo>,
    for_io_type: IoType,
) -> SaveDialogMsg {
    match opt_selected.as_ref() {
        Some(selected) if selected.io_type == for_io_type => match selected.path.to_str() {
            Some(path_str) => SaveDialogMsg::SaveAs(path_str.to_string()),
            None => SaveDialogMsg::Save,
        },
        _ => SaveDialogMsg::Save,
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

/// Get a `&'static str` reference to the filename within the XDG configuration directory.
///
/// NOTE: Leaks the [String] that backs the return value, because the Relm4 field that uses it
/// ([OpenButtonSettings::recently_opened_files]) requires a `&'static str`, but the value must
/// be dynamically generated based on the XDG configuration path.
pub fn xdg_cfg_static_str<X: AsRef<xdg::BaseDirectories>>(
    xdg_dirs: X,
    filename: &str,
) -> Option<&'static str> {
    xdg_dirs
        .as_ref()
        .place_config_file(filename)
        .map_err(|e| {
            log::warn!("Could not create {:?} file: {:?}", filename, e);
            e
        })
        .ok()
        .and_then(|p: PathBuf| {
            p.to_str().map(|s: &str| {
                let static_str: &'static str = s.to_owned().leak();
                static_str
            })
        })
}
