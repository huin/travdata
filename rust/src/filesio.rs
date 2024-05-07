use std::{
    io::Read,
    path::{Path, PathBuf},
};

use anyhow::{bail, Result};

type BoxRead = Box<dyn Read>;

/// Protocol for reading files from the collection.
pub trait Reader {
    /// Open a text file for reading. `path` is the path of the file to read.
    fn open_read(&self, path: &Path) -> Result<BoxRead>;

    /// Iterates over all files that the reader has. The order is undefined.
    fn iter_files(&self) -> Box<dyn Iterator<Item = &Path>>;

    /// Return `true` if the file exists.
    fn exists(&self, path: &Path) -> bool;
}

pub struct DirReader {
    base_dir: PathBuf,
}

/// Returns an error if `path` is not strictly relative. That is satisfying both:
/// * Has no prefix component.
/// * Has no root component.
fn check_fully_relative(path: &Path) -> Result<()> {
    use std::path::Component::{Prefix, RootDir};
    match path.components().next() {
        Some(Prefix(p)) => bail!("{:?} has a prefix {:?}", path, p),
        Some(RootDir) => bail!("{:?} is absolute", path),
        _ => Ok(()),
    }

    // Should check for parent paths? Although this isn't foolproof in itself.
    // We should be checking configuration that leads to this. Not worth
    // checking all symlink type situations.
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use googletest::{
        expect_that,
        matchers::{anything, err, ok},
    };

    use crate::filesio::check_fully_relative;

    #[googletest::test]
    fn test_is_fully_relative() {
        expect_that!(check_fully_relative(Path::new(r#"foo"#)), ok(()));
        expect_that!(check_fully_relative(Path::new(r#"foo/bar"#)), ok(()));
        expect_that!(check_fully_relative(Path::new(r#"/foo"#)), err(anything()));
    }

    #[cfg(target_os = "windows")]
    #[googletest::test]
    fn test_is_fully_relative_on_windows() {
        expect_that!(check_fully_relative(Path::new(r#"foo\bar"#)), ok(()));
        expect_that!(
            check_fully_relative(Path::new(r#"C:\foo"#)),
            err(anything())
        );
        expect_that!(
            check_fully_relative(Path::new(r#"C:/foo"#)),
            err(anything())
        );
        expect_that!(check_fully_relative(Path::new(r#"C:foo"#)), err(anything()));
        expect_that!(check_fully_relative(Path::new(r#"c:foo"#)), err(anything()));
        expect_that!(
            check_fully_relative(Path::new(r#"\\server\share\foo"#)),
            err(anything())
        );
    }
}

impl Reader for DirReader {
    fn open_read(&self, path: &Path) -> Result<BoxRead> {
        check_fully_relative(path)?;
        todo!()
    }

    fn iter_files(&self) -> Box<dyn Iterator<Item = &Path>> {
        todo!()
    }

    fn exists(&self, path: &Path) -> bool {
        todo!()
    }
}
