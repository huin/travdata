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
    fn iter_files(&self) -> Box<dyn Iterator<Item = Result<PathBuf>> + 'a>;

    /// Return `true` if the file exists.
    fn exists(&self, path: &Path) -> bool;
}

/// Protocol for reading and writing files in the collection.
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
        matchers::{eq, err, ok, unordered_elements_are},
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
        fn make_read_writer_as_reader(&self) -> BoxReader<'static>;

        fn run_with_reader_and_read_writer(&self, f: &dyn Fn(&'static str, BoxReader<'static>)) {
            f("Reader", self.make_reader());
            f("ReadWriter", self.make_read_writer_as_reader());
        }
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

        fn make_read_writer_as_reader(&self) -> BoxReader<'static> {
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
        Case("reads_created_files", &reads_created_files),
        Case("readers_iter_files", &readers_iter_files),
        Case("read_writer_overwrites_file", &read_writer_overwrites_file),
    ];

    /// Checks the `test_casing` count in `io_test`.
    #[test]
    fn io_test_count() {
        assert_eq!(7, COMMON_IO_TESTS.iter().count() * IO_TYPES.iter().count());
    }

    #[test_casing(7, Product((IO_TYPES, COMMON_IO_TESTS)))]
    fn io_test(io_type: &IoType, case: &Case) {
        case.1(io_type);
    }

    fn empty_reader_has_no_files(io_type: &IoType) {
        let test_io = io_type.new_env();
        let reader = test_io.make_reader();
        let actual_files = read_iter_files(reader.as_ref());
        assert_that!(actual_files, unordered_elements_are![]);
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

    fn reads_created_files(io_type: &IoType) {
        let test_io = io_type.new_env();
        let files: Vec<(&Path, &[u8])> = vec![
            (Path::new("file.txt"), b"file contents"),
            (Path::new("subdir/other.txt"), b"other contents"),
        ];

        {
            let read_writer = test_io.make_read_writer();
            for (path, contents) in &files {
                let mut w = read_writer.open_write(path).expect("should open");
                w.write_all(*contents).expect("should write");
            }

            for (path, contents) in &files {
                // Should be present in ReadWriter that created them.
                let mut r = read_writer.open_read(path).expect("should open");
                let read_contents = read_vec(&mut r).expect("should read");
                assert_that!(&read_contents, eq(contents));
            }
        }

        // Should be present in Reader implementations.
        test_io.run_with_reader_and_read_writer(&|desc, reader| {
            for (path, contents) in &files {
                println!("Testing {}", desc);
                let mut r = reader.open_read(path).expect("should open");
                let read_contents = read_vec(&mut r).expect("should read");
                assert_that!(&read_contents, eq(contents));
            }
        });
    }

    fn readers_iter_files(io_type: &IoType) {
        let test_io = io_type.new_env();
        let files: Vec<&Path> = vec![
            Path::new("file.txt"),
            Path::new("subdir/other.txt"),
            Path::new("subdir/anotherdir/file.txt"),
        ];

        {
            let read_writer = test_io.make_read_writer();
            for path in &files {
                let mut w = read_writer.open_write(path).expect("should open");
                w.write_all(b"ignored content").expect("should write");
            }

            // Should be present in ReadWriter that created them.
            let actual_files = read_writer_iter_files(read_writer.as_ref());
            assert_that!(
                actual_files,
                unordered_elements_are![
                    ok(eq(Path::new("file.txt"))),
                    ok(eq(Path::new("subdir/other.txt"))),
                    ok(eq(Path::new("subdir/anotherdir/file.txt"))),
                ]
            );
        }

        // Should be present in Reader implementations.
        test_io.run_with_reader_and_read_writer(&|desc, reader| {
            println!("Testing {}", desc);
            // Should be present in ReadWriter that created them.
            let actual_files = read_iter_files(reader.as_ref());
            assert_that!(
                actual_files,
                unordered_elements_are![
                    ok(eq(Path::new("file.txt"))),
                    ok(eq(Path::new("subdir/other.txt"))),
                    ok(eq(Path::new("subdir/anotherdir/file.txt"))),
                ]
            );
        });
    }

    fn read_writer_overwrites_file(io_type: &IoType) {
        let path = Path::new("file.txt");
        let v1 = b"content v1";
        let v2 = b"content v2";
        let v3 = b"content v3";

        let test_io = io_type.new_env();

        {
            let read_writer = test_io.make_read_writer();
            {
                let mut w = read_writer.open_write(path).expect("should open");
                w.write_all(v1).expect("should write");
            }
            {
                let mut r = read_writer.open_read(path).expect("should open");
                assert_that!(read_vec(&mut r), ok(eq(v1)));
            }
        }

        {
            let read_writer = test_io.make_read_writer();
            {
                let mut r = read_writer.open_read(path).expect("should open");
                assert_that!(read_vec(&mut r), ok(eq(v1)));
            }
            {
                let mut w = read_writer.open_write(path).expect("should open");
                w.write_all(v2).expect("should write");
            }
            {
                let mut r = read_writer.open_read(path).expect("should open");
                assert_that!(read_vec(&mut r), ok(eq(v2)));
            }
            {
                let mut w = read_writer.open_write(path).expect("should open");
                w.write_all(v3).expect("should write");
            }
            {
                let mut r = read_writer.open_read(path).expect("should open");
                assert_that!(read_vec(&mut r), ok(eq(v3)));
            }
        }

        {
            let reader = test_io.make_reader();
            let mut r = reader.open_read(path).expect("should open");
            assert_that!(read_vec(&mut r), ok(eq(v3)));
        }
    }

    const VALID_RELATIVE_PATHS: &[&str] = &[r#"foo"#, r#"foo/bar"#];

    /// Checks the `test_casing` count in `test_is_fully_relative`.
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

    /// Checks the `test_casing` count in `test_invalid_relative_path`.
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

    /// Checks the `test_casing` count in `test_invalid_relative_path_on_windows`.
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

    fn read_iter_files(reader: &dyn Reader) -> Vec<Result<PathBuf>> {
        reader.iter_files().collect()
    }

    fn read_writer_iter_files(reader: &dyn ReadWriter) -> Vec<Result<PathBuf>> {
        reader.iter_files().collect()
    }
}
