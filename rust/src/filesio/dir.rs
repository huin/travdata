use std::{
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};

use crate::filesio::FilesIoError;

use super::{check_fully_relative, BoxRead, BoxWrite, ReadWriter, Reader};

#[derive(Debug)]
pub struct DirReadWriter {
    dir_path: PathBuf,
}

impl DirReadWriter {
    pub fn new<P>(dir_path: P) -> Self
    where
        P: Into<PathBuf>,
    {
        Self {
            dir_path: dir_path.into(),
        }
    }
}

impl<'a> Reader<'a> for DirReadWriter {
    fn open_read(&self, path: &Path) -> Result<BoxRead<'a>> {
        check_fully_relative(path)?;
        let full_path = self.dir_path.join(path);

        let f: File = match File::open(full_path) {
            Ok(f) => f,
            Err(e) => {
                return Err(anyhow!(match e.kind() {
                    std::io::ErrorKind::NotFound => anyhow!(FilesIoError::NotFound),
                    _ => anyhow!(e),
                }));
            }
        };
        Ok(Box::new(f))
    }

    fn iter_files(&self) -> Box<dyn Iterator<Item = Result<PathBuf>> + 'a> {
        let dir_path = self.dir_path.to_owned();
        Box::new(
            walkdir::WalkDir::new(&dir_path)
                .follow_links(false)
                .same_file_system(true)
                .into_iter()
                .filter_map(move |dir_entry| match dir_entry {
                    Err(e) => match e.io_error() {
                        // NotFound for dir_path implies no entries at all,
                        // which is not an error, just an empty reader.
                        Some(io_err)
                            if io_err.kind() == std::io::ErrorKind::NotFound
                                && e.path() == Some(&dir_path) =>
                        {
                            None
                        }
                        // Pass other errors through.
                        _ => Some(Err(anyhow!(e))),
                    },
                    Ok(dir_entry) if dir_entry.file_type().is_file() => {
                        match dir_entry.path().strip_prefix(&dir_path) {
                            Err(e) => Some(Err(anyhow!(e))),
                            Ok(rel_path) => Some(Ok(rel_path.to_owned())),
                        }
                    }
                    _ => None,
                }),
        )
    }

    fn exists(&self, path: &Path) -> bool {
        let full_path = self.dir_path.join(path);
        full_path.exists()
    }
}

impl<'a> ReadWriter<'a> for DirReadWriter {
    fn open_write(&self, path: &Path) -> Result<BoxWrite<'a>> {
        check_fully_relative(path)?;
        let full_path = self.dir_path.join(path);

        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let f = File::create(full_path)?;
        Ok(Box::new(f))
    }
}
