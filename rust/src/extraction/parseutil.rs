use lazy_regex::{regex_find, regex_replace_all};

/// Cleans leading, trailing, and redundant whitespace from a string, in-place.
pub fn clean_text(s: &mut String) {
    let trimmed = s.trim();
    // Skip the copy/realloc if nothing to do.
    if trimmed.len() != s.len() || regex_find!(r"\s{2,}", &trimmed).is_some() {
        let new = regex_replace_all!(r"\s{2,}", trimmed, " ");
        *s = new.to_string();
    }
}
