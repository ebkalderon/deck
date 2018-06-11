//! Partial package ID, usually inputted by a user.

use std::borrow::Cow;
use std::str::FromStr;

use semver::{SemVerError, Version};
use url::Url;

/// Partial package ID.
///
/// # Examples
///
/// * `foo`
/// * `foo:1.0.0`
/// * `deck.io/foo`
/// * `deck.io/foo#1.0.0`
/// * `deck.io/foo/bar:1.0.0`
/// * `https://deck.io/foo#1.0.0`
/// * `https://github.com/path-to-repo.git/foo#1.0.0`
/// * `git://github.com/path-to-repo.git/foo#1.0.0`
/// * `git://github.com/path-to-repo.git/foo/bar:1.0.0`
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PackageIdSpec<'a> {
    name: Cow<'a, str>,
    version: Option<Version>,
    source: Option<Url>,
}

impl FromStr for PackageIdSpec<'static> {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        unimplemented!()
    }
}

#[derive(Debug, Fail)]
pub enum ParseError {
    #[fail(display = "invalid package ID: {}", _0)]
    InvalidId(String),
    #[fail(display = "invalid package version: {}", _0)]
    InvalidVersion(#[cause] SemVerError),
}

impl From<SemVerError> for ParseError {
    fn from(err: SemVerError) -> Self {
        ParseError::InvalidVersion(err)
    }
}
