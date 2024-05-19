mod dir;
mod mem;
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

pub use dir::DirReadWriter;

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

// pub type BoxFileWrite<'a> = Box<dyn FileWrite<'a>>;

/// Concrete error type returned by `FilesIo` implementations for cases that
/// might reasonably be handled by callers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FilesIoError {
    NonRelativePath(NonRelativePathType),
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
            NonRelativePath(t) => write!(
                f,
                "path is not relative because it contains a {} component",
                t
            ),
            NotFound => write!(f, "file not found"),
        }
    }
}

/// Type of path `Component` causing a path to be non-relative.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NonRelativePathType {
    Prefix,
    RootDir,
}

impl Display for NonRelativePathType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use NonRelativePathType::*;
        match self {
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

/// Returns an error if `path` is not strictly relative. That is satisfying both:
/// * Has no prefix component.
/// * Has no root component.
fn check_fully_relative(path: &Path) -> Result<()> {
    use std::path::Component::{Prefix, RootDir};
    match path.components().next() {
        Some(Prefix(_)) => Err(anyhow!(FilesIoError::NonRelativePath(
            NonRelativePathType::Prefix
        ))),
        Some(RootDir) => Err(anyhow!(FilesIoError::NonRelativePath(
            NonRelativePathType::RootDir
        ))),
        _ => Ok(()),
    }

    // Should check for parent paths? Although this isn't foolproof in itself.
    // We should be checking configuration that leads to this. Not worth
    // checking all symlink type situations.
}
