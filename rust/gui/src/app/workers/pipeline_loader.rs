use std::{
    path::{Path, PathBuf},
    thread,
};

use crate::app::ddo;

pub fn start_load<M>(
    path_selection: ddo::PathSelection,
    sender: egui_inbox::UiInboxSender<M>,
    to_result: impl FnOnce(ddo::PathSelection, Result<ddo::PipelineNodes, String>) -> M + Send + 'static,
) where
    M: std::fmt::Debug + Send + 'static,
{
    thread::spawn(move || {
        let result = load(&path_selection.path);
        if let Err(err) = sender.send(to_result(path_selection, result)) {
            log::warn!("Failed to send loaded pipeline: {err:?}");
        }
    });
}

fn load(path: &Path) -> Result<ddo::PipelineNodes, String> {
    let mut f = std::fs::File::open(path)
        .map_err(|err| format!("Could not open file to read pipeline: {err:?}"))?;
    serde_json::from_reader(&mut f).map_err(|err| format!("{err:?}"))
}

pub fn start_save<M>(
    pipeline: ddo::PipelineNodes,
    path: PathBuf,
    sender: egui_inbox::UiInboxSender<M>,
    to_result: impl FnOnce(Result<(), String>) -> M + Send + 'static,
) where
    M: std::fmt::Debug + Send + 'static,
{
    std::thread::spawn(move || {
        let result = save(&path, &pipeline);
        if let Err(err) = sender.send(to_result(result)) {
            log::warn!("Failed to send notification of saved pipeline: {err:?}");
        }
    });
}

fn save(path: &Path, pipeline: &ddo::PipelineNodes) -> Result<(), String> {
    // TODO: Make the save atomic.
    let mut f = std::fs::File::create(path)
        .map_err(|err| format!("Could not open file to write pipeline: {err:?}"))?;
    serde_json::to_writer_pretty(&mut f, pipeline).map_err(|err| format!("{err:?}"))
}
