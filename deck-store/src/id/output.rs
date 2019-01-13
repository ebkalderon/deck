use std::fmt::{Display, Formatter, Result as FmtResult};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde::ser::{Serialize, Serializer};

use super::{name::Name, FilesystemId};
use crate::hash::Hash;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OutputId {
    name: Name,
    version: String,
    output: Option<Name>,
    hash: Hash,
}

impl OutputId {
    #[inline]
    pub fn new(name: Name, version: String, output: Option<Name>, hash: Hash) -> Self {
        OutputId {
            name,
            version,
            output,
            hash,
        }
    }

    pub fn parse<T>(name: T, version: T, output: Option<T>, hash: T) -> Result<Self, ()>
    where
        T: AsRef<str>,
    {
        let output = match output {
            Some(s) => Some(s.as_ref().parse()?),
            None => None,
        };

        Ok(OutputId {
            name: name.as_ref().parse()?,
            version: version.as_ref().to_string(),
            output,
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
    pub fn output(&self) -> Option<&str> {
        self.output.as_ref().map(|out| out.as_str())
    }

    #[inline]
    pub fn hash(&self) -> &Hash {
        &self.hash
    }
}

impl Display for OutputId {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        let out = self
            .output
            .as_ref()
            .map(|out| format!(":{}", out))
            .unwrap_or_default();

        write!(fmt, "{}@{}{}-{}", self.name, self.version, out, self.hash)
    }
}

impl FilesystemId for OutputId {
    fn from_path(path: &Path) -> Result<Self, ()> {
        let raw_name = path.file_name().ok_or(())?;
        let name = raw_name.to_str().ok_or(())?;
        OutputId::from_str(name)
    }

    fn to_path(&self) -> PathBuf {
        PathBuf::from(self.to_string())
    }
}

impl FromStr for OutputId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens = s.rsplitn(2, '-');
        let hash = tokens.next().ok_or(())?;
        let remainder = tokens.next().ok_or(())?;

        let mut tokens = remainder.rsplitn(2, '@');
        let identifier = tokens.next().ok_or(())?;
        let name = tokens.next().ok_or(())?;

        let mut tokens = identifier.splitn(2, ':');
        let version = tokens.next().ok_or(())?;
        let output = tokens.next();

        OutputId::parse(name, version, output, hash)
    }
}

impl PartialEq<str> for OutputId {
    fn eq(&self, other: &str) -> bool {
        let s = self.to_string();
        s.as_str() == other
    }
}

impl PartialEq<&'_ str> for OutputId {
    fn eq(&self, other: &&str) -> bool {
        self == *other
    }
}

impl PartialEq<OutputId> for str {
    fn eq(&self, other: &OutputId) -> bool {
        other.to_string().as_str() == self
    }
}

impl PartialEq<OutputId> for &'_ str {
    fn eq(&self, other: &OutputId) -> bool {
        other == self
    }
}

impl<'de> Deserialize<'de> for OutputId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct OutputIdVisitor;

        impl<'de> Visitor<'de> for OutputIdVisitor {
            type Value = OutputId;

            fn expecting(&self, fmt: &mut Formatter) -> FmtResult {
                fmt.write_str("a build output ID with the form `name@version[:output]-hash`")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                OutputId::from_str(value).map_err(|_err| E::custom("failed to deserialize"))
            }
        }

        deserializer.deserialize_str(OutputIdVisitor)
    }
}

impl Serialize for OutputId {
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
    const SIMPLE_ID: &'static str = "foobar@1.0.0-fc3j3vub6kodu4jtfoakfs5xhumqi62m";
    const WITH_OUTPUT_NAME: &'static str = "foobar@1.0.0:man-fc3j3vub6kodu4jtfoakfs5xhumqi62m";

    #[test]
    fn is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<OutputId>();
    }

    #[test]
    fn parse_simple_id_from_string() {
        let expected = OutputId::parse("foobar", "1.0.0", None, HASH).expect("Failed to init ID");
        let actual: OutputId = SIMPLE_ID.parse().expect("Failed to parse ID");
        assert_eq!(expected, actual);
        assert_eq!(expected.name(), actual.name());
        assert_eq!(expected.version(), actual.version());
        assert_eq!(expected.output(), actual.output());
        assert_eq!(expected.hash(), actual.hash());
    }

    #[test]
    fn parse_simple_roundtrip() {
        let original: OutputId = SIMPLE_ID.parse().expect("Failed to parse ID");
        let text_form = original.to_string();

        let parsed: OutputId = text_form.parse().expect("Failed to parse ID from text");
        assert_eq!(original, parsed);
    }

    #[test]
    fn parse_id_with_name_from_string() {
        let expected =
            OutputId::parse("foobar", "1.0.0", Some("man"), HASH).expect("Failed to init ID");
        let actual: OutputId = WITH_OUTPUT_NAME.parse().expect("Failed to parse ID");
        assert_eq!(expected, actual);
        assert_eq!(expected.name(), actual.name());
        assert_eq!(expected.version(), actual.version());
        assert_eq!(expected.output(), actual.output());
        assert_eq!(expected.hash(), actual.hash());
    }

    #[test]
    fn parse_id_with_name_roundtrip() {
        let original: OutputId = WITH_OUTPUT_NAME.parse().expect("Failed to parse ID");
        let text_form = original.to_string();

        let parsed: OutputId = text_form.parse().expect("Failed to parse ID from text");
        assert_eq!(original, parsed);
    }
}
