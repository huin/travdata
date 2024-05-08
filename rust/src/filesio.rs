use std::{
    error::Error,
    fmt::{Debug, Display},
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};

pub trait DebugRead<'a>: Debug + Read + 'a {}
pub trait DebugWrite<'a>: Debug + Write + 'a {}

pub type BoxRead<'a> = Box<dyn DebugRead<'a>>;
pub type BoxWrite<'a> = Box<dyn DebugWrite<'a>>;

/// Concrete error type returned by `FilesIo` implementations for cases that
/// might reasonably be handled by callers.
#[derive(Debug, Eq, PartialEq)]
pub enum FilesIoError {
    NonRelativePath(NonRelativePathType),
    NotFound,
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
#[derive(Debug, Eq, PartialEq)]
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
    fn open_read(&self, path: &Path) -> Result<BoxRead<'a>>;

    /// Iterates over all files that the reader has. The order is undefined.
    fn iter_files(&self) -> Box<dyn Iterator<Item = &'a Path> + 'a>;

    /// Return `true` if the file exists.
    fn exists(&self, path: &Path) -> bool;
}

// Protocol for reading and writing files in the collection.
pub trait ReadWriter<'a>: Reader<'a> {
    /// Open a text file for writing. `path` is the path of the file to write.
    fn open_write(&self, path: &Path) -> Result<BoxWrite<'a>>;
}

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

    fn iter_files(&self) -> Box<dyn Iterator<Item = &'a Path> + 'a> {
        // Incomplete implementation. Write failing tests.
        Box::new(vec![].into_iter())
    }

    fn exists(&self, _path: &Path) -> bool {
        // Incomplete implementation. Write failing tests.
        false
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

impl<'a> DebugRead<'a> for File {}
impl<'a> DebugWrite<'a> for File {}

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

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use anyhow::Result;
    use googletest::{
        assert_that, expect_that,
        matchers::{eq, err, ok},
    };
    use tempfile::{tempdir, TempDir};

    use crate::{
        filesio::{check_fully_relative, FilesIoError, NonRelativePathType},
        testutil::anyhow_downcasts_to,
    };

    use super::{BoxRead, DirReadWriter, ReadWriter, Reader};

    type NewBoxTestIo = &'static dyn Fn() -> Result<BoxTestIo>;
    type BoxTestIo = Box<dyn TestIo>;
    type BoxReader<'a> = Box<dyn Reader<'a>>;
    type BoxReadWriter<'a> = Box<dyn ReadWriter<'a>>;

    trait TestIo {
        fn make_reader(&self) -> BoxReader<'static>;
        fn make_read_writer(&self) -> BoxReadWriter<'static>;
    }

    struct TestDir {
        temp_dir: TempDir,
    }

    impl TestDir {
        fn new() -> Result<BoxTestIo> {
            Ok(Box::new(Self {
                temp_dir: tempdir()?,
            }))
        }

        fn dir_path(&self) -> PathBuf {
            self.temp_dir.path().join("dir")
        }
    }

    impl TestIo for TestDir {
        fn make_reader(&self) -> BoxReader<'static> {
            Box::new(DirReadWriter::new(self.dir_path()))
        }

        fn make_read_writer(&self) -> BoxReadWriter<'static> {
            Box::new(DirReadWriter::new(self.dir_path()))
        }
    }

    #[googletest::test]
    fn test_dir() {
        test_io(&TestDir::new);
    }

    fn test_io(new_test_io: NewBoxTestIo) {
        empty_reader_has_no_files(new_test_io);
        empty_reader_not_exists(new_test_io);
        empty_reader_open_read_returns_not_found_err(new_test_io);

        read_writer_reads_own_file(new_test_io);
    }

    fn empty_reader_has_no_files(new_test_io: NewBoxTestIo) {
        let test_io = new_test_io().unwrap();
        let reader = test_io.make_reader();
        assert_that!(reader.iter_files().count(), eq(0));
    }

    fn empty_reader_not_exists(new_test_io: NewBoxTestIo) {
        let test_io = new_test_io().unwrap();
        let reader = test_io.make_reader();
        assert_that!(reader.exists(Path::new("not-exist")), eq(false));
    }

    fn empty_reader_open_read_returns_not_found_err(new_test_io: NewBoxTestIo) {
        let test_io = new_test_io().unwrap();
        let reader = test_io.make_reader();
        assert_that!(
            reader.open_read(Path::new("not-exist")),
            err(anyhow_downcasts_to::<FilesIoError>(eq(
                FilesIoError::NotFound
            ))),
        );
    }

    fn read_writer_reads_own_file(new_test_io: NewBoxTestIo) {
        let test_io = new_test_io().unwrap();
        let read_writer = test_io.make_read_writer();

        let path = Path::new("file.txt");
        let contents = b"contents";

        let mut w = read_writer.open_write(&path).expect("should open");
        w.write_all(contents).expect("should write");
        drop(w);

        let mut r = read_writer.open_read(&path).expect("should open");
        let actual_contents = read_vec(&mut r).expect("should read");
        assert_that!(&actual_contents, eq(contents));
    }

    fn read_vec(r: &mut BoxRead) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        r.read_to_end(&mut buf)?;
        Ok(buf)
    }

    #[googletest::test]
    fn test_is_fully_relative() {
        expect_that!(check_fully_relative(Path::new(r#"foo"#)), ok(()));
        expect_that!(check_fully_relative(Path::new(r#"foo/bar"#)), ok(()));
        expect_that!(
            check_fully_relative(Path::new(r#"/foo"#)),
            err(anyhow_downcasts_to::<FilesIoError>(eq(
                FilesIoError::NonRelativePath(NonRelativePathType::RootDir)
            )))
        );
    }

    #[cfg(target_os = "windows")]
    #[googletest::test]
    fn test_is_fully_relative_on_windows() {
        expect_that!(check_fully_relative(Path::new(r#"foo\bar"#)), ok(()));
        expect_that!(
            check_fully_relative(Path::new(r#"C:\foo"#)),
            err(anyhow_downcasts_to::<FilesIoError>(eq(
                FilesIoError::NonRelativePath(NonRelativePathType::Prefix)
            )))
        );
        expect_that!(
            check_fully_relative(Path::new(r#"C:/foo"#)),
            err(anyhow_downcasts_to::<FilesIoError>(eq(
                FilesIoError::NonRelativePath(NonRelativePathType::Prefix)
            )))
        );
        expect_that!(
            check_fully_relative(Path::new(r#"C:foo"#)),
            err(anyhow_downcasts_to::<FilesIoError>(eq(
                FilesIoError::NonRelativePath(NonRelativePathType::Prefix)
            )))
        );
        expect_that!(
            check_fully_relative(Path::new(r#"c:foo"#)),
            err(anyhow_downcasts_to::<FilesIoError>(eq(
                FilesIoError::NonRelativePath(NonRelativePathType::Prefix)
            )))
        );
        expect_that!(
            check_fully_relative(Path::new(r#"\\server\share\foo"#)),
            err(anyhow_downcasts_to::<FilesIoError>(eq(
                FilesIoError::NonRelativePath(NonRelativePathType::Prefix)
            )))
        );
    }
}
