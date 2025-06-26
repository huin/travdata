use std::fmt::Display;

/// Joins the slice of items, formatting each using their `Display`
/// implementation, separated by `separator`.
pub fn join_display_slice<T>(slice: &[T], separator: &str) -> String
where
    T: Display,
{
    format!("{}", DisplayJoin { slice, separator })
}

struct DisplayJoin<'c, 's, T> {
    slice: &'c [T],
    separator: &'s str,
}

impl<T> Display for DisplayJoin<'_, '_, T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iterator = self.slice.iter().peekable();
        while let Some(item) = iterator.next() {
            write!(f, "{}", item)?;
            if iterator.peek().is_some() {
                f.write_str(self.separator)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use googletest::{assert_that, matchers::eq};

    use super::*;

    #[test]
    fn test_join_display_slice() {
        let numbers: &[i32] = &[1, 2, 3];
        assert_that!(join_display_slice(numbers, "-"), eq("1-2-3"));
    }
}
