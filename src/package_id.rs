//! Concrete package ID.

use std::borrow::Cow;
use std::str::FromStr;

use semver::Version;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceId;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PackageId<'p> {
    name: Cow<'p, str>,
    version: Version,
    source: SourceId,
}

impl<'p> PackageId<'p> {
    pub fn new<S: AsRef<str>>(spec: S) -> Result<Self, ParseError> {
        unimplemented!()
    }
}

impl<'p> FromStr for PackageId<'p> {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

#[derive(Debug, Fail)]
pub enum ParseError {
    #[fail(display = "invalid package ID: {}", _0)]
    InvalidId(String),
}
