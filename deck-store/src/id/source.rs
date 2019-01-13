use std::fmt::{Display, Formatter, Result as FmtResult};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::de::{self, Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};

use super::FilesystemId;
use crate::hash::Hash;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceId {
    name: String,
    hash: Hash,
}

impl SourceId {
    pub fn new(name: String, hash: Hash) -> Result<Self, ()> {
        if name.is_empty() {
            return Err(());
        }

        Ok(SourceId { name, hash })
    }

    #[inline]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    #[inline]
    pub fn hash(&self) -> &Hash {
        &self.hash
    }
}

impl Display for SourceId {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "{}-{}", self.name, self.hash)
    }
}

impl FilesystemId for SourceId {
    fn from_path(path: &Path) -> Result<Self, ()> {
        let raw_name = path.file_name().ok_or(())?;
        let name = raw_name.to_str().ok_or(())?;
        SourceId::from_str(name)
    }

    fn to_path(&self) -> PathBuf {
        PathBuf::from(self.to_string())
    }
}

impl FromStr for SourceId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens = s.rsplitn(2, '-');
        let hash = tokens.next().ok_or(()).and_then(|s| s.parse())?;
        let name = tokens.next().map(|s| s.to_string()).ok_or(())?;

        if tokens.count() != 0 {
            return Err(());
        }

        SourceId::new(name, hash)
    }
}

impl<'de> Deserialize<'de> for SourceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        SourceId::from_str(&s).map_err(|_err| de::Error::custom("failed to deserialize"))
    }
}

impl Serialize for SourceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HASH: &'static str = "fc3j3vub6kodu4jtfoakfs5xhumqi62m";
    const EXAMPLE_ID: &'static str = "foobar.json-fc3j3vub6kodu4jtfoakfs5xhumqi62m";

    #[test]
    fn is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SourceId>();
    }

    #[test]
    fn parse_from_string() {
        let hash = HASH.parse().expect("Failed to parse hash from constant");
        let expected = SourceId::new("foobar.json".to_string(), hash).expect("Failed to init ID");
        let actual: SourceId = EXAMPLE_ID.parse().expect("Failed to parse ID!");
        assert_eq!(expected, actual);
        assert_eq!(expected.name(), actual.name());
        assert_eq!(expected.hash(), actual.hash());
    }

    #[test]
    fn parse_roundtrip() {
        let original: SourceId = EXAMPLE_ID.parse().expect("Failed to parse ID");
        let text_form = original.to_string();

        let parsed: SourceId = text_form.parse().expect("Failed to parse ID from text");
        assert_eq!(original, parsed);
    }

    #[test]
    fn reject_empty_name() {
        let hash = HASH.parse().expect("Failed to parse hash from constant");
        SourceId::new("".to_string(), hash).expect_err("Failed to reject empty name");
    }
}
