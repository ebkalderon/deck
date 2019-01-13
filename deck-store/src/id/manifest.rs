use std::fmt::{Display, Formatter, Result as FmtResult};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde::ser::{Serialize, Serializer};

use super::{name::Name, FilesystemId, OutputId};
use crate::hash::Hash;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ManifestId {
    name: Name,
    version: String,
    hash: Hash,
}

impl ManifestId {
    pub fn new(name: Name, version: String, hash: Hash) -> Self {
        ManifestId {
            name,
            version,
            hash,
        }
    }

    pub fn parse<T: AsRef<str>>(name: T, version: T, hash: T) -> Result<Self, ()> {
        Ok(ManifestId {
            name: name.as_ref().parse()?,
            version: version.as_ref().into(),
            hash: hash.as_ref().parse()?,
        })
    }

    #[inline]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    #[inline]
    pub fn version(&self) -> &str {
        self.version.as_str()
    }

    #[inline]
    pub fn hash(&self) -> &Hash {
        &self.hash
    }

    pub fn is_same_package(&self, output_id: &OutputId) -> bool {
        let name_matches = self.name.as_str() == output_id.name();
        let version_matches = self.version.as_str() == output_id.version();
        name_matches && version_matches
    }
}

impl Display for ManifestId {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "{}@{}-{}", self.name, self.version, self.hash)
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
        let mut tokens = s.rsplitn(2, '-');
        let hash = tokens.next().ok_or(())?;
        let remainder = tokens.next().ok_or(())?;

        let mut tokens = remainder.rsplitn(2, '@');
        let version = tokens.next().ok_or(())?;
        let name = tokens.next().ok_or(())?;

        ManifestId::parse(name, version, hash)
    }
}

impl PartialEq<str> for ManifestId {
    fn eq(&self, other: &str) -> bool {
        let s = self.to_string();
        s.as_str() == other
    }
}

impl PartialEq<&'_ str> for ManifestId {
    fn eq(&self, other: &&str) -> bool {
        self == *other
    }
}

impl PartialEq<ManifestId> for str {
    fn eq(&self, other: &ManifestId) -> bool {
        other.to_string().as_str() == self
    }
}

impl PartialEq<ManifestId> for &'_ str {
    fn eq(&self, other: &ManifestId) -> bool {
        other == self
    }
}

impl<'de> Deserialize<'de> for ManifestId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ManifestIdVisitor;

        impl<'de> Visitor<'de> for ManifestIdVisitor {
            type Value = ManifestId;

            fn expecting(&self, fmt: &mut Formatter) -> FmtResult {
                fmt.write_str("a manifest ID with the form `name-version-hash`")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                ManifestId::from_str(value).map_err(|_err| E::custom("failed to deserialize"))
            }
        }

        deserializer.deserialize_str(ManifestIdVisitor)
    }
}

impl Serialize for ManifestId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HASH: &'static str = "fc3j3vub6kodu4jtfoakfs5xhumqi62m";
    const EXAMPLE_ID: &'static str = "foobar@1.0.0-fc3j3vub6kodu4jtfoakfs5xhumqi62m";

    #[test]
    fn is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ManifestId>();
    }

    #[test]
    fn path_ends_with_toml() {
        let id: ManifestId = EXAMPLE_ID.parse().expect("Failed to parse ID");
        let path = id.to_path();
        assert_eq!(path.extension().and_then(|s| s.to_str()), Some("toml"));
    }

    #[test]
    fn parse_from_string() {
        let expected = ManifestId::parse("foobar", "1.0.0", HASH).expect("Failed to init ID");
        let actual: ManifestId = EXAMPLE_ID.parse().expect("Failed to parse ID");
        assert_eq!(expected, actual);
        assert_eq!(expected.name(), actual.name());
        assert_eq!(expected.version(), actual.version());
        assert_eq!(expected.hash(), actual.hash());
    }

    #[test]
    fn parse_roundtrip() {
        let original: ManifestId = EXAMPLE_ID.parse().expect("Failed to parse ID");
        let text_form = original.to_string();

        let parsed: ManifestId = text_form.parse().expect("Failed to parse ID from text");
        assert_eq!(original, parsed);
    }
}
