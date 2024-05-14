use std::{
    cmp::min,
    collections::HashMap,
    fmt::Debug,
    io::{Read, Write},
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::anyhow;

use super::{FileRead, FileReadImpl, FileWrite, FileWriteImpl, FilesIoError, ReadWriter, Reader};

type FileMap = HashMap<PathBuf, Arc<[u8]>>;

#[derive(Clone, Default)]
pub struct MemFilesHandle {
    file_map: Arc<Mutex<FileMap>>,
}

pub struct MemReadWriter {
    files: MemFilesHandle,
}

impl MemReadWriter {
    pub fn new(files: MemFilesHandle) -> Self {
        Self { files }
    }
}

impl<'a> Reader<'a> for MemReadWriter {
    fn open_read(&self, path: &std::path::Path) -> anyhow::Result<super::FileRead<'a>> {
        let files_guard = self.files.file_map.lock().expect("failed to lock file map");
        match files_guard.get(path) {
            None => Err(anyhow!(FilesIoError::NotFound)),
            Some(buf) => Ok(FileRead::new(MemFileRead::new(
                path.to_owned(),
                buf.clone(),
            ))),
        }
    }

    fn iter_files(&self) -> Box<dyn Iterator<Item = anyhow::Result<std::path::PathBuf>> + 'a> {
        let paths: Vec<_> = self
            .files
            .file_map
            .lock()
            .expect("failed to lock file map")
            .keys()
            .cloned()
            .map(Ok)
            .collect();
        Box::new(paths.into_iter())
    }

    fn exists(&self, path: &std::path::Path) -> bool {
        let files_guard = self.files.file_map.lock().expect("failed to lock file map");
        files_guard.contains_key(path)
    }
}

impl<'a> ReadWriter<'a> for MemReadWriter {
    fn open_write(&self, path: &std::path::Path) -> anyhow::Result<super::FileWrite<'a>> {
        Ok(FileWrite::new(MemFileWrite {
            files: self.files.clone(),
            path: path.to_owned(),
            buf: Vec::new(),
        }))
    }
}

struct MemFileRead {
    path: PathBuf,
    buf: Arc<[u8]>,
    pos: usize,
}

impl MemFileRead {
    fn new(path: PathBuf, buf: Arc<[u8]>) -> Self {
        Self { path, buf, pos: 0 }
    }
}

impl<'a> Debug for MemFileRead {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemFileRead")
            .field("path", &self.path)
            .finish()
    }
}

impl<'a> FileReadImpl<'a> for MemFileRead {}

impl Read for MemFileRead {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let bytes_remaining = self.buf.len() - self.pos;
        let n = min(buf.len(), bytes_remaining);
        buf[..n].copy_from_slice(&self.buf[self.pos..self.pos + n]);
        self.pos += n;
        Ok(n)
    }
}

struct MemFileWrite {
    files: MemFilesHandle,
    path: PathBuf,
    buf: Vec<u8>,
}

impl<'a> Debug for MemFileWrite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemFileWrite")
            .field("path", &self.path)
            .finish()
    }
}

impl<'a> FileWriteImpl<'a> for MemFileWrite {
    fn commit(self: Box<Self>) -> anyhow::Result<()> {
        let mut files_guard = self
            .files
            .file_map
            .lock()
            .map_err(|e| anyhow!("failed to lock file map: {}", e))?;
        files_guard.insert(self.path, self.buf.into());
        Ok(())
    }

    fn discard(self: Box<Self>) -> anyhow::Result<()> {
        Ok(())
    }
}

impl Write for MemFileWrite {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buf.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
