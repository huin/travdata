use std::{
    cell::RefCell,
    collections::HashSet,
    fmt::Debug,
    fs::File,
    io::{ErrorKind, Read},
    path::{Path, PathBuf},
    rc::Rc,
};

use anyhow::{anyhow, Result};
use atomic_write_file::AtomicWriteFile;
use tempfile::TempDir;
use zip::{write::SimpleFileOptions, ZipArchive, ZipWriter};

use crate::filesio::FilesIoError;

use super::{
    check_fully_relative, dir::DirReadWriter, util::read_from_slice, FileRead, FileReadImpl,
    ReadWriter, Reader,
};

pub struct ZipReader {
    zip_archive: Option<RefCell<ZipArchive<File>>>,
}

impl ZipReader {
    pub fn new(path: &Path, ignore_not_exist: bool) -> Result<Self> {
        let f = match File::open(path) {
            Ok(f) => f,
            Err(e) if e.kind() == ErrorKind::NotFound && ignore_not_exist => {
                return Ok(Self { zip_archive: None });
            }
            Err(e) => {
                return Err(anyhow!(e));
            }
        };
        Ok(Self {
            zip_archive: Some(RefCell::new(ZipArchive::new(f)?)),
        })
    }
}

impl<'a> Reader<'a> for ZipReader {
    fn open_read(&self, path: &Path) -> anyhow::Result<super::FileRead<'a>> {
        check_fully_relative(path)?;
        match &self.zip_archive {
            Some(zip_archive) => {
                let mut zip_file_mut = zip_archive.borrow_mut();
                let index = zip_file_mut
                    .index_for_path(path)
                    .ok_or(FilesIoError::NotFound)?;

                let mut zip_file = zip_file_mut.by_index(index)?;
                let mut buf = Vec::new();
                zip_file.read_to_end(&mut buf)?;

                Ok(FileRead::new(ZipFileRead {
                    path: path.to_owned(),
                    buf,
                    pos: 0,
                }))
            }
            None => Err(anyhow!(FilesIoError::NotFound)),
        }
    }

    fn iter_files(&self) -> Box<dyn Iterator<Item = anyhow::Result<PathBuf>> + 'a> {
        match &self.zip_archive {
            Some(zip_archive) => {
                // Must clone the names in the ZIP archive, self reference does
                // not outlive the returned iterator.
                let paths: Vec<_> = zip_archive
                    .borrow()
                    .file_names()
                    .map(|s| Ok(PathBuf::from(s)))
                    .collect();
                Box::new(paths.into_iter())
            }
            None => Box::new(None.into_iter()),
        }
    }

    fn exists(&self, path: &Path) -> bool {
        match &self.zip_archive {
            Some(zip_archive) => zip_archive.borrow().index_for_path(path).is_some(),
            None => false,
        }
    }
}

struct ZipFileRead {
    path: PathBuf,
    buf: Vec<u8>,
    pos: usize,
}

impl Debug for ZipFileRead {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ZipFileRead")
            .field("path", &self.path)
            .finish()
    }
}

impl Read for ZipFileRead {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        read_from_slice(&mut self.pos, &self.buf, buf)
    }
}

impl FileReadImpl<'_> for ZipFileRead {}

pub struct ZipReadWriter {
    dest_path: PathBuf,
    // For reading existing files:
    zip_reader: ZipReader,
    // For creating/overwriting or reading new/overwritten files. Allow dead code because it must live until `ReadWriter::close` completes
    #[allow(dead_code)]
    read_write_tempdir: TempDir,
    read_writer: DirReadWriter,
}

impl ZipReadWriter {
    pub fn new(path: &Path) -> Result<Self> {
        let dest_path = path.to_owned();
        let zip_reader = ZipReader::new(path, true)?;
        let read_write_tempdir = TempDir::new()?;
        let read_writer = DirReadWriter::new(read_write_tempdir.path());

        Ok(Self {
            dest_path,
            zip_reader,
            read_write_tempdir,
            read_writer,
        })
    }
}

impl<'a> Reader<'a> for ZipReadWriter {
    fn open_read(&self, path: &Path) -> anyhow::Result<super::FileRead<'a>> {
        check_fully_relative(path)?;
        match self.read_writer.open_read(path) {
            Ok(fw) => Ok(fw),
            Err(err) if FilesIoError::NotFound.eq_anyhow(&err) => {
                // Continue on to try self.zip_reader.
                self.zip_reader.open_read(path)
            }
            Err(err) => Err(err),
        }
    }

    fn iter_files(&self) -> Box<dyn Iterator<Item = anyhow::Result<PathBuf>> + 'a> {
        let new_paths_mut = Rc::new(RefCell::new(HashSet::<PathBuf>::new()));
        let new_paths_read = new_paths_mut.clone();

        Box::new(
            self.read_writer
                .iter_files()
                .inspect(move |path_result| {
                    if let Ok(path) = path_result {
                        new_paths_mut.borrow_mut().insert(path.clone());
                    }
                })
                .chain(
                    self.zip_reader
                        .iter_files()
                        .filter(move |path_result| match path_result {
                            Ok(path) => !new_paths_read.borrow().contains(path),
                            Err(_) => true,
                        }),
                ),
        )
    }

    fn exists(&self, path: &Path) -> bool {
        self.read_writer.exists(path) || self.zip_reader.exists(path)
    }
}

impl<'a> ReadWriter<'a> for ZipReadWriter {
    fn open_write(&self, path: &Path) -> anyhow::Result<super::FileWrite<'a>> {
        check_fully_relative(path)?;
        self.read_writer.open_write(path)
    }

    fn close(self: Box<ZipReadWriter>) -> Result<()> {
        let mut zip_writer = ZipWriter::new(AtomicWriteFile::open(self.dest_path)?);

        // Copy over new/overwriting files.
        let mut new_paths: HashSet<String> = HashSet::new();
        for path_result in self.read_writer.iter_files() {
            let path = path_result?;
            let path_str = normalise_path_slashes(&path)?;

            new_paths.insert(path_str.to_owned());
            zip_writer.start_file(path_str, SimpleFileOptions::default())?;
            let mut r = self.read_writer.open_read(&path)?;
            std::io::copy(&mut r, &mut zip_writer)?;
        }

        // Copy over existing files that were not overwritten.
        if let Some(zip_archive) = self.zip_reader.zip_archive {
            let mut zip_archive_mut = zip_archive.borrow_mut();
            for index in 0..zip_archive_mut.len() {
                let zip_entry = zip_archive_mut.by_index(index)?;

                // Filter out files that have been overwritten.
                if new_paths.contains(zip_entry.name()) {
                    continue;
                }
                new_paths.insert(zip_entry.name().to_owned());

                zip_writer.raw_copy_file(zip_entry)?;
            }
        }

        // Complete writing the new ZIP archive, and commit the atomic file it
        // was writing to.
        let file = zip_writer.finish()?;
        file.commit()?;

        Ok(())
    }
}

/// Normalise a [Path] to use forward slashes, for uniformity of ZIP file entry
/// names between platforms.
fn normalise_path_slashes(p: &Path) -> Result<String> {
    Ok(p.to_str()
        .ok_or_else(|| anyhow!("could not convert path {:?} to UTF-8 string", p))?
        .replace('\\', "/"))
}
