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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    use std::{
        fmt::Debug,
        path::{Path, PathBuf},
    };

    use anyhow::Result;
    use googletest::{
        assert_that,
        matchers::{eq, err, ok},
    };
    use tempfile::{tempdir, TempDir};
    use test_casing::{test_casing, Product};

    use crate::{
        filesio::{check_fully_relative, FilesIoError, NonRelativePathType},
        testutil::anyhow_downcasts_to,
    };

    use super::{BoxRead, DirReadWriter, ReadWriter, Reader};

    type BoxIoTestEnvironment = Box<dyn IoTestEnvironment>;
    type BoxReader<'a> = Box<dyn Reader<'a>>;
    type BoxReadWriter<'a> = Box<dyn ReadWriter<'a>>;

    trait IoTestEnvironment {
        fn make_reader(&self) -> BoxReader<'static>;
        fn make_read_writer(&self) -> BoxReadWriter<'static>;
    }

    struct DirTestEnvironment {
        temp_dir: TempDir,
    }

    impl DirTestEnvironment {
        fn new() -> Result<BoxIoTestEnvironment> {
            Ok(Box::new(Self {
                temp_dir: tempdir()?,
            }))
        }

        fn dir_path(&self) -> PathBuf {
            self.temp_dir.path().join("dir")
        }
    }

    impl IoTestEnvironment for DirTestEnvironment {
        fn make_reader(&self) -> BoxReader<'static> {
            Box::new(DirReadWriter::new(self.dir_path()))
        }

        fn make_read_writer(&self) -> BoxReadWriter<'static> {
            Box::new(DirReadWriter::new(self.dir_path()))
        }
    }

    struct IoType {
        name: &'static str,
        new: &'static dyn Fn() -> Result<Box<dyn IoTestEnvironment>>,
    }

    impl IoType {
        fn new_env(&self) -> Box<dyn IoTestEnvironment> {
            (self.new)().expect("should not fail")
        }
    }

    impl Debug for IoType {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.name)
        }
    }

    const IO_TYPES: &[IoType] = &[IoType {
        name: "Dir",
        new: &DirTestEnvironment::new,
    }];

    struct Case(&'static str, &'static dyn Fn(&IoType));

    impl Debug for Case {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    const COMMON_IO_TESTS: &[Case] = &[
        Case("empty_reader_has_no_files", &empty_reader_has_no_files),
        Case("empty_reader_not_exists", &empty_reader_not_exists),
        Case(
            "empty_reader_open_read_returns_not_found_err",
            &empty_reader_open_read_returns_not_found_err,
        ),
        Case("read_writer_reads_own_file", &read_writer_reads_own_file),
    ];

    #[test]
    fn io_test_count() {
        assert_eq!(4, COMMON_IO_TESTS.iter().count() * IO_TYPES.iter().count());
    }

    #[test_casing(4, Product((IO_TYPES, COMMON_IO_TESTS)))]
    fn io_test(io_type: &IoType, case: &Case) {
        case.1(io_type);
    }

    fn empty_reader_has_no_files(io_type: &IoType) {
        let test_io = io_type.new_env();
        let reader = test_io.make_reader();
        assert_that!(reader.iter_files().count(), eq(0));
    }

    fn empty_reader_not_exists(io_type: &IoType) {
        let test_io = io_type.new_env();
        let reader = test_io.make_reader();
        assert_that!(reader.exists(Path::new("not-exist")), eq(false));
    }

    fn empty_reader_open_read_returns_not_found_err(io_type: &IoType) {
        let test_io = io_type.new_env();
        let reader = test_io.make_reader();
        assert_that!(
            reader.open_read(Path::new("not-exist")),
            err(anyhow_downcasts_to::<FilesIoError>(eq(
                FilesIoError::NotFound
            ))),
        );
    }

    fn read_writer_reads_own_file(io_type: &IoType) {
        let test_io = io_type.new_env();
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

    const VALID_RELATIVE_PATHS: &[&str] = &[r#"foo"#, r#"foo/bar"#];

    #[test]
    fn test_is_fully_relative_count() {
        assert_eq!(2, VALID_RELATIVE_PATHS.iter().count());
    }

    #[test_casing(2, VALID_RELATIVE_PATHS)]
    fn test_is_fully_relative(path: &str) {
        assert_that!(check_fully_relative(Path::new(path)), ok(()));
    }

    const INVALID_RELATIVE_PATHS: &[(&str, FilesIoError)] = &[(
        r#"/foo"#,
        FilesIoError::NonRelativePath(NonRelativePathType::RootDir),
    )];

    #[test]
    fn test_invalid_relative_path_count() {
        assert_eq!(1, INVALID_RELATIVE_PATHS.iter().count());
    }

    #[test_casing(1, INVALID_RELATIVE_PATHS)]
    fn test_invalid_relative_path(path: &str, expect_error: &FilesIoError) {
        assert_that!(
            check_fully_relative(Path::new(path)),
            err(anyhow_downcasts_to::<FilesIoError>(eq(*expect_error))),
        );
    }

    const INVALID_RELATIVE_PATHS_ON_WINDOWS: &[(&str, FilesIoError)] = &[
        (
            r#"\foo"#,
            FilesIoError::NonRelativePath(NonRelativePathType::RootDir),
        ),
        (
            r#"C:\foo"#,
            FilesIoError::NonRelativePath(NonRelativePathType::Prefix),
        ),
        (
            r#"C:/foo"#,
            FilesIoError::NonRelativePath(NonRelativePathType::Prefix),
        ),
        (
            r#"C:foo"#,
            FilesIoError::NonRelativePath(NonRelativePathType::Prefix),
        ),
        (
            r#"c:foo"#,
            FilesIoError::NonRelativePath(NonRelativePathType::Prefix),
        ),
        (
            r#"\\server\share\foo"#,
            FilesIoError::NonRelativePath(NonRelativePathType::Prefix),
        ),
    ];

    #[test]
    fn test_invalid_relative_path_on_windows_count() {
        assert_eq!(6, INVALID_RELATIVE_PATHS_ON_WINDOWS.iter().count());
    }

    #[cfg(target_os = "windows")]
    #[test_casing(6, INVALID_RELATIVE_PATHS_ON_WINDOWS)]
    fn test_invalid_relative_path_on_windows(path: &str, expect_error: &FilesIoError) {
        assert_that!(
            check_fully_relative(Path::new(path)),
            err(anyhow_downcasts_to::<FilesIoError>(eq(*expect_error))),
        );
    }

    // Utility code for tests:

    fn read_vec(r: &mut BoxRead) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        r.read_to_end(&mut buf)?;
        Ok(buf)
    }
}
