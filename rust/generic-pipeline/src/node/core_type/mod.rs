//! Core types used within an extraction configuration [crate::node::GenericNode].
//!
//! Many of these have value validation, so their inner value is private.

#[cfg(any(test, feature = "testing"))]
mod test_defaults;
#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
#[error("got {value:?} which is not {expected}")]
pub struct ValueError<V> {
    pub value: V,
    pub expected: &'static str,
}

/// Unique identifier of an extraction configuration [crate::node::GenericNode] within a
/// [crate::pipeline::GenericPipeline].
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
pub struct NodeId(String);

impl NodeId {
    const EXPECTED: &str = r#"a node ID string matching ^[a-zA-Z0-9]([a-zA-Z0-9-]*[a-zA-Z0-9])?$"#;

    #[cfg(any(test, feature = "testing"))]
    pub fn test_node_id(s: &str) -> Self {
        s.to_string()
            .try_into()
            .expect("expected valid NodeId value")
    }

    fn valid_regex() -> &'static lazy_regex::Regex {
        lazy_regex::regex!(r#"^[a-zA-Z0-9]([a-zA-Z0-9-]*[a-zA-Z0-9])?$"#)
    }

    fn try_from_string<S>(value: S) -> std::result::Result<Self, S>
    where
        S: Into<String> + AsRef<str>,
    {
        let rx = Self::valid_regex();
        if rx.is_match(value.as_ref()) {
            std::result::Result::Ok(Self(value.into()))
        } else {
            std::result::Result::Err(value)
        }
    }

    #[cfg(any(test, feature = "testing"))]
    fn new_unchecked(value: String) -> Self {
        Self(value)
    }
}

impl From<&NodeId> for NodeId {
    fn from(value: &NodeId) -> Self {
        value.clone()
    }
}

impl TryFrom<&str> for NodeId {
    type Error = ValueError<String>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from_string(value).map_err(|value| ValueError {
            value: value.to_string(),
            expected: Self::EXPECTED,
        })
    }
}

impl TryFrom<String> for NodeId {
    type Error = ValueError<String>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from_string(value).map_err(|value| ValueError {
            value,
            expected: Self::EXPECTED,
        })
    }
}

impl<'de> Deserialize<'de> for NodeId {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        Self::try_from_string(s).map_err(|s| {
            serde::de::Error::invalid_value(serde::de::Unexpected::Str(&s), &Self::EXPECTED)
        })
    }
}

impl AsRef<str> for NodeId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Tag value that non-uniquely identifies a set of extraction configuration
/// [crate::node::GenericNode]s.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
pub struct Tag(String);

impl Tag {
    const EXPECTED: &str = r#"a valid tag string containing one or more slash delimited components, each matching ^[a-zA-Z0-9]([a-zA-Z0-9-]*[a-zA-Z0-9])?$"#;

    #[cfg(any(test, feature = "testing"))]
    pub fn test_tag(s: &str) -> Self {
        s.to_string().try_into().expect("expected valid Tag value")
    }

    fn valid_regex() -> &'static lazy_regex::Regex {
        lazy_regex::regex!(
            r#"^([a-zA-Z0-9]([a-zA-Z0-9-]*[a-zA-Z0-9])?)(/([a-zA-Z0-9]([a-zA-Z0-9-]*[a-zA-Z0-9])?))*$"#
        )
    }

    fn try_from_string<S>(value: S) -> std::result::Result<Self, S>
    where
        S: Into<String> + AsRef<str>,
    {
        let rx = Self::valid_regex();
        if rx.is_match(value.as_ref()) {
            std::result::Result::Ok(Self(value.into()))
        } else {
            std::result::Result::Err(value)
        }
    }

    #[cfg(test)]
    fn new_unchecked(value: String) -> Self {
        Self(value)
    }
}

impl TryFrom<&str> for Tag {
    type Error = ValueError<String>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from_string(value).map_err(|value| ValueError {
            value: value.to_string(),
            expected: Self::EXPECTED,
        })
    }
}

impl TryFrom<String> for Tag {
    type Error = ValueError<String>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from_string(value).map_err(|value| ValueError {
            value,
            expected: Self::EXPECTED,
        })
    }
}

impl<'de> Deserialize<'de> for Tag {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        Self::try_from_string(s).map_err(|s| {
            serde::de::Error::invalid_value(serde::de::Unexpected::Str(&s), &Self::EXPECTED)
        })
    }
}
