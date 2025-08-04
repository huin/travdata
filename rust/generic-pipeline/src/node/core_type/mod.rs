//! Core types used within an extraction configuration [crate::node::GenericNode].
//!
//! Many of these have value validation, so their inner value is private.

#[cfg(test)]
mod tests;

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

/// Unique identifier of an extraction configuration [crate::node::GenericNode] within a
/// [crate::pipeline::GenericPipeline].
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
pub struct NodeId(String);

impl NodeId {
    const EXPECTED: &str = r#"a string matching ^[a-zA-Z0-9]([a-zA-Z0-9-]*[a-zA-Z0-9])?$"#;

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

    #[cfg(test)]
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
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from_string(value)
            .map_err(|value| anyhow!("NodeId: got {value:?} which is not {}", Self::EXPECTED))
    }
}

impl TryFrom<String> for NodeId {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from_string(value)
            .map_err(|value| anyhow!("NodeId: got {value:?} which is not {}", Self::EXPECTED))
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

/// Tag value that non-uniquely identifies a set of extraction configuration
/// [crate::node::GenericNode]s.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
pub struct Tag(String);

impl Tag {
    const EXPECTED: &str = r#"a string containing one or more slash delimited components, each matching ^[a-zA-Z0-9]([a-zA-Z0-9-]*[a-zA-Z0-9])?$"#;

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
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from_string(value)
            .map_err(|value| anyhow!("Tag: got {value:?} which is not {}", Self::EXPECTED))
    }
}

impl TryFrom<String> for Tag {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from_string(value)
            .map_err(|value| anyhow!("Tag: got {value:?} which is not {}", Self::EXPECTED))
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
