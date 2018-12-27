use std::fmt::{Display, Formatter, Result as FmtResult};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use super::{name::Name, FilesystemId};
use hash::Hash;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ManifestId {
    name: Name,
    version: String,
    hash: Hash,
}

impl ManifestId {
    pub fn new(name: String, version: String, hash: Hash) -> Result<Self, ()> {
        Ok(ManifestId {
            name: Name::new(name)?,
            version,
            hash,
        })
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn version(&self) -> &str {
        self.version.as_str()
    }

    pub fn hash(&self) -> &Hash {
        &self.hash
    }
}

impl Display for ManifestId {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "{}-{}-{}", self.name, self.version, self.hash)
    }
}

impl FilesystemId for ManifestId {
    fn from_path(path: &Path) -> Result<Self, ()> {
        let raw_stem = path.file_stem().ok_or(())?;
        let stem = raw_stem.to_str().ok_or(())?;
        ManifestId::from_str(stem)
    }

    fn to_path(&self) -> PathBuf {
        let path = format!("{}.toml", self);
        PathBuf::from(path)
    }
}

impl FromStr for ManifestId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens = s.rsplitn(3, '-');
        let hash = tokens.next().ok_or(()).and_then(|s| s.parse())?;
        let version = tokens.next().map(|s| s.to_string()).ok_or(())?;
        let name = tokens.next().map(|s| s.to_string()).ok_or(())?;

        if tokens.count() != 0 {
            return Err(());
        }

        ManifestId::new(name, version, hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HASH: &'static str = "fc3j3vub6kodu4jtfoakfs5xhumqi62m";
    const EXAMPLE_ID: &'static str = "foobar-1.0.0-fc3j3vub6kodu4jtfoakfs5xhumqi62m";

    #[test]
    fn is_send_and_sync() {
        fn verify<T: Send + Sync>() {}
        verify::<ManifestId>();
    }

    #[test]
    fn path_ends_with_toml() {
        let id: ManifestId = EXAMPLE_ID.parse().expect("Failed to parse ID!");
        let path = id.to_path();
        assert_eq!(path.extension().and_then(|s| s.to_str()), Some("toml"));
    }

    #[test]
    fn parse_from_string() {
        let hash = HASH.parse().expect("Failed to parse hash from constant!");
        let expected = ManifestId::new("foobar".to_string(), "1.0.0".to_string(), hash)
            .expect("Failed to init ID!");
        let actual: ManifestId = EXAMPLE_ID.parse().expect("Failed to parse ID!");
        assert_eq!(expected, actual);
        assert_eq!(expected.name(), actual.name());
        assert_eq!(expected.version(), actual.version());
        assert_eq!(expected.hash(), actual.hash());
    }

    #[test]
    fn parse_roundtrip() {
        let original: ManifestId = EXAMPLE_ID.parse().expect("Failed to parse ID!");
        let text_form = original.to_string();

        let parsed: ManifestId = text_form.parse().expect("Failed to parse ID from text!");
        assert_eq!(original, parsed);
    }
}
