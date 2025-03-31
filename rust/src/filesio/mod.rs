mod dir;
#[cfg(test)]
pub mod mem;
#[cfg(test)]
mod tests;
mod util;
mod zip;

use std::{
    error::Error,
    fmt::{Debug, Display},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};

use clap::ValueEnum;

use crate::filesio::{
    dir::DirReadWriter,
    zip::{ZipReadWriter, ZipReader},
};

trait FileReadImpl<'a>: Debug + Read + 'a {}
trait FileWriteImpl<'a>: Debug + Write + 'a {
    fn commit(self: Box<Self>) -> Result<()>;
    fn discard(self: Box<Self>) -> Result<()>;
}

pub struct FileRead<'a> {
    delegate: Box<dyn FileReadImpl<'a>>,
}

impl<'a> FileRead<'a> {
    fn new<T>(delegate: T) -> Self
    where
        T: FileReadImpl<'a>,
    {
        Self {
            delegate: Box::new(delegate),
        }
    }
}

impl<'a> Debug for FileRead<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.delegate.fmt(f)
    }
}

impl<'a> Read for FileRead<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.delegate.read(buf)
    }
}

pub struct FileWrite<'a> {
    delegate: Box<dyn FileWriteImpl<'a>>,
}

impl<'a> FileWrite<'a> {
    fn new<T>(delegate: T) -> Self
    where
        T: FileWriteImpl<'a>,
    {
        Self {
            delegate: Box::new(delegate),
        }
    }

    pub fn commit(self) -> Result<()> {
        self.delegate.commit()
    }

    #[allow(dead_code)]
    pub fn discard(self) -> Result<()> {
        self.delegate.discard()
    }
}

impl<'a> Debug for FileWrite<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.delegate.fmt(f)
    }
}

impl<'a> Write for FileWrite<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.delegate.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.delegate.flush()
    }
}

/// Concrete error type returned by `FilesIo` implementations for cases that
/// might reasonably be handled by callers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FilesIoError {
    NonLinearRelativePath(NonRelativePathType),
    NotFound,
}

impl FilesIoError {
    pub fn eq_anyhow(&self, err: &anyhow::Error) -> bool {
        if let Some(&err) = err.downcast_ref::<Self>() {
            err == *self
        } else {
            false
        }
    }
}

impl Display for FilesIoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use FilesIoError::*;
        match self {
            NonLinearRelativePath(t) => write!(
                f,
                "path is not linear relative because it contains a {} component",
                t
            ),
            NotFound => write!(f, "file not found"),
        }
    }
}

/// Type of path `Component` causing a path to be non-relative.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NonRelativePathType {
    CurDir,
    ParentDir,
    Prefix,
    RootDir,
}

impl Display for NonRelativePathType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use NonRelativePathType::*;
        match self {
            CurDir => write!(f, "current directory"),
            ParentDir => write!(f, "parent directory"),
            Prefix => write!(f, "prefix"),
            RootDir => write!(f, "root directory"),
        }
    }
}

impl Error for FilesIoError {}

/// Protocol for reading files from the collection.
pub trait Reader<'a> {
    /// Open a text file for reading. `path` is the path of the file to read.
    fn open_read(&self, path: &Path) -> Result<FileRead<'a>>;

    /// Iterates over all files that the reader has. The order is undefined.
    fn iter_files(&self) -> Box<dyn Iterator<Item = Result<PathBuf>> + 'a>;

    /// Return `true` if the file exists.
    fn exists(&self, path: &Path) -> bool;
}

/// Protocol for reading and writing files in the collection.
pub trait ReadWriter<'a>: Reader<'a> {
    /// Open a text file for writing. `path` is the path of the file to write.
    fn open_write(&self, path: &Path) -> Result<FileWrite<'a>>;

    /// Close the `ReadWriter` and flush its changes. Any changes commited via
    /// `FileWrite` may or may not be visible to other `Reader`s and
    /// `ReadWriter`s if this is not called instead of dropping.
    fn close(self: Box<Self>) -> Result<()>;
}

/// Returns an error if `path` is not strictly linear and relative. That is
/// satisfying both:
/// * Has no leading current directory component.
/// * Has no parent directory component.
/// * Has no prefix component.
/// * Has no root directory component.
fn check_fully_relative(path: &Path) -> Result<()> {
    use std::path::Component::{CurDir, ParentDir, Prefix, RootDir};

    path.components()
        .find_map(|component| match component {
            Prefix(_) => Some(FilesIoError::NonLinearRelativePath(
                NonRelativePathType::Prefix,
            )),
            RootDir => Some(FilesIoError::NonLinearRelativePath(
                NonRelativePathType::RootDir,
            )),
            CurDir => Some(FilesIoError::NonLinearRelativePath(
                NonRelativePathType::CurDir,
            )),
            ParentDir => Some(FilesIoError::NonLinearRelativePath(
                NonRelativePathType::ParentDir,
            )),
            _ => None,
        })
        .map(|err| Err(anyhow!(err)))
        .unwrap_or(Ok(()))
}

#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
pub enum IoType {
    /// Access files stored in a directory.
    Dir,
    /// Access files stored in a ZIP archive.
    Zip,
}

impl IoType {
    /// Creates a `Reader` of the given type for the directory or archive at the
    /// given path.
    pub fn new_reader<'r>(self, path: &Path) -> Result<Box<dyn Reader<'r>>> {
        use IoType::*;
        match self {
            Dir => Ok(Box::new(DirReadWriter::new(path))),
            Zip => Ok(Box::new(ZipReader::new(path, false)?)),
        }
    }

    /// Creates a `ReadWriter` of the given type for the directory or archive at
    /// the given path.
    pub fn new_read_writer<'r>(self, path: &Path) -> Result<Box<dyn ReadWriter<'r>>> {
        use IoType::*;
        match self {
            Dir => Ok(Box::new(DirReadWriter::new(path))),
            Zip => Ok(Box::new(ZipReadWriter::new(path)?)),
        }
    }

    // Resolves the `Option<IoType>` into a concrete type, based on the given
    // path, attempting to guess a reasonable value.
    pub fn resolve_auto(io_type: Option<IoType>, path: &Path) -> Self {
        match io_type {
            Some(io_type) => io_type,
            None => {
                if !path.exists() {
                    if path.extension().and_then(|oss| oss.to_str()) == Some("zip") {
                        IoType::Zip
                    } else {
                        IoType::Dir
                    }
                } else if path.is_file() {
                    IoType::Zip
                } else {
                    IoType::Dir
                }
            }
        }
    }
}

impl Display for IoType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use IoType::*;
        match self {
            Dir => f.write_str("folder"),
            Zip => f.write_str("ZIP file"),
        }
    }
}

/// An [IoType] associated with a [PathBuf].
#[derive(Clone, Debug)]
pub struct FileIoPath {
    pub io_type: IoType,
    pub path: PathBuf,
}

impl FileIoPath {
    /// Create a [FileIoPath] with type [IoType::Dir].
    pub fn for_dir(path: PathBuf) -> Self {
        Self {
            io_type: IoType::Dir,
            path,
        }
    }

    /// Create a [FileIoPath] with type [IoType::Zip].
    pub fn for_zip(path: PathBuf) -> Self {
        Self {
            io_type: IoType::Zip,
            path,
        }
    }

    /// Create a [Reader].
    pub fn new_reader<'r>(&self) -> Result<Box<dyn Reader<'r>>> {
        self.io_type.new_reader(&self.path)
    }

    /// Create a [ReadWriter].
    pub fn new_read_writer<'r>(&self) -> Result<Box<dyn ReadWriter<'r>>> {
        self.io_type.new_read_writer(&self.path)
    }
}

impl Display for FileIoPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {:?}", self.io_type, self.path)
    }
}
