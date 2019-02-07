use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;

use serde::de::{self, Deserialize, Deserializer};
use serde::Serialize;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Name(String);

impl Name {
    pub fn new<S: Into<String>>(name: S) -> Result<Name, ()> {
        let s = name.into();
        if s.is_empty() {
            return Err(());
        }

        let allowed_chars = s
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.');

        let reserved_names = match s.as_str() {
            "." | ".." | "/" => true,
            _ => false,
        };

        if !allowed_chars || reserved_names {
            return Err(());
        }

        Ok(Name(s))
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Name {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        Name::from_str(&s).map_err(|_err| de::Error::custom("failed to deserialize"))
    }
}

impl Display for Name {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "{}", self.0)
    }
}

impl FromStr for Name {
    type Err = ();

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Name::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Name>();
    }

    #[test]
    fn parse_valid_names() {
        Name::new("foo-bar").expect("Failed to parse valid name with hyphen");
        Name::new("foo_bar").expect("Failed to parse valid name with underscore");
        Name::new("f0-o_B4.r").expect("Failed to parse valid name with mixed chars");
    }

    #[test]
    fn reject_invalid_names() {
        Name::new("foo bar").expect_err("Failed to reject name with space");
        Name::new("/foo/bar").expect_err("Failed to reject name with path-like slashes");
        Name::new("foo!@#$%^&*(){}+?<>'\"").expect_err("Failed to reject name with special chars");
    }

    #[test]
    fn reject_empty_name() {
        Name::new("").expect_err("Failed to reject empty name");
    }
}
