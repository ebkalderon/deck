//! Partial package ID, usually inputted by a user.

use std::borrow::Cow;
use std::str::FromStr;

use semver::Version;
use url::Url;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PackageIdSpec<'p> {
    name: Cow<'p, str>,
    version: Option<Version>,
    source: Option<Url>,
}

impl<'p> PackageIdSpec<'p> {
    pub fn parse<S: AsRef<str>>(spec: S) -> Result<Self, ParseError> {
        unimplemented!()
    }
}

impl<'p> FromStr for PackageIdSpec<'p> {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[derive(Debug, Fail)]
pub enum ParseError {
    #[fail(display = "invalid package ID: {}", _0)]
    InvalidId(String),
}
