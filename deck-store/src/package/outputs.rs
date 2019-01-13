//! Represents the `output` array table in the package manifest.

use std::collections::BTreeSet;
use std::fmt::{Display, Formatter, Result as FmtResult};

use serde::de::{self, Deserialize, Deserializer, Error as DeError, SeqAccess, Visitor};
use serde::ser::{Serialize, SerializeSeq, Serializer};

use crate::hash::Hash;
use crate::id::{Name, OutputId};

/// Represents the `output` array table in the package manifest.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Outputs(BTreeSet<Entry>);

impl Outputs {
    /// Creates a new `Outputs` table with the default output set to the given precomputed hash and
    /// references.
    ///
    /// # What is meant by a "precomputed hash"
    ///
    /// Since the store follows an intensional model (see Eelco Dolstra's writings on the theorized
    /// "intensional model" within Nix and NixOS), this precomputed hash is used only to identify
    /// compatible trusted substitutes for safe sharing between untrusted users. It may be
    /// rewritten to something else after the builder has been run.
    pub fn new<T>(precomputed_hash: Hash, refs: T) -> Self
    where
        T: IntoIterator<Item = OutputId>,
    {
        let refs = refs.into_iter().collect();
        let mut set = BTreeSet::new();
        set.insert(Entry::new(Output::Default, precomputed_hash, refs));
        Outputs(set)
    }

    /// Appends a new named output with the given name, precomputed hash [`Hash`], and references.
    ///
    /// # What is meant by a "precomputed hash"
    ///
    /// Since the store follows an intensional model (see Eelco Dolstra's writings on the theorized
    /// "intensional model" within Nix and NixOS), this precomputed hash is used only to identify
    /// compatible trusted substitutes for safe sharing between untrusted users. It may be
    /// rewritten to something else after the builder has been run.
    #[inline]
    pub fn append<T>(&mut self, name: Name, precomputed_hash: Hash, refs: T)
    where
        T: IntoIterator<Item = OutputId>,
    {
        let refs = refs.into_iter().collect();
        let output = Entry::new(Output::Named(name), precomputed_hash, refs);
        self.0.insert(output);
    }

    /// Renders the given output entries as a set of [`OutputId`]s with `name` and `version`.
    ///
    /// [`OutputId`]: ../struct.OutputId.html
    pub fn iter_with(&self, name: Name, version: String) -> impl Iterator<Item = OutputId> + '_ {
        self.0
            .iter()
            .map(move |out| out.to_output_id(name.clone(), version.clone()))
    }
}

impl<'de> Deserialize<'de> for Outputs {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let set: BTreeSet<Entry> = Deserialize::deserialize(deserializer)?;
        let num_default_outputs = set.iter().filter(|out| out.is_default_output()).count();
        if num_default_outputs == 1 {
            Ok(Outputs(set))
        } else if num_default_outputs > 1 {
            Err(de::Error::custom("cannot have multiple default outputs"))
        } else {
            Err(de::Error::custom("missing default output"))
        }
    }
}

impl Serialize for Outputs {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

/// Types of build outputs that a manifest can have.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Output {
    /// The unnamed default output.
    ///
    /// Contains the actual application/library being installed. When a package is installed by a
    /// user, this is likely what they are looking for. Manifests are required to have one and only
    /// one default output.
    Default,
    /// An optional "named" output.
    ///
    /// Contains extra components or artifacts not typically included in a base installation. For
    /// example, the `doc` named output could contain rendered HTML documentation, the `man` named
    /// output could contain man pages, the `debug` named output could contain debugging symbols,
    /// etc. Users can request to install these add-on outputs on top of the default output.
    Named(Name),
}

impl Output {
    /// Returns whether this output is an [`Output::Default`].
    ///
    /// [`Output::Default`]: ./struct.Output.html#variant.Default
    pub fn is_default_output(&self) -> bool {
        *self == Output::Default
    }
}

impl Default for Output {
    fn default() -> Self {
        Output::Default
    }
}

impl Display for Output {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match *self {
            Output::Default => write!(fmt, ""),
            Output::Named(ref name) => write!(fmt, "{}", name),
        }
    }
}

impl Into<Option<Name>> for Output {
    fn into(self) -> Option<Name> {
        match self {
            Output::Default => None,
            Output::Named(name) => Some(name),
        }
    }
}

impl<'de> Deserialize<'de> for Output {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;

        if s.is_empty() {
            Ok(Output::Default)
        } else {
            let out = s
                .parse()
                .map_err(|_err| de::Error::custom("failed to deserialize"))?;
            Ok(Output::Named(out))
        }
    }
}

impl Serialize for Output {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Output::Default => None::<&str>.serialize(serializer),
            Output::Named(_) => self.to_string().serialize(serializer),
        }
    }
}

/// A single serializable entry in the `Outputs` table.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
struct Entry {
    #[serde(default, rename = "name")]
    output_name: Output,
    precomputed_hash: Hash,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    references: BTreeSet<OutputId>,
}

impl Entry {
    /// Creates a new `Entry` with the given name, precomputed hash, and references.
    #[inline]
    pub fn new(output_name: Output, precomputed_hash: Hash, refs: BTreeSet<OutputId>) -> Self {
        Entry {
            output_name,
            precomputed_hash,
            references: refs,
        }
    }

    /// Returns whether this output entry is an [`Output::Default`].
    ///
    /// [`Output::Default`]: ./struct.Output.html#variant.Default
    #[inline]
    fn is_default_output(&self) -> bool {
        self.output_name.is_default_output()
    }

    /// Renders this entry as an `OutputId` using the given name and version information.
    #[inline]
    fn to_output_id(&self, name: Name, version: String) -> OutputId {
        let output_name = self.output_name.clone();
        let precomputed_hash = self.precomputed_hash.clone();
        OutputId::new(name, version, output_name.into(), precomputed_hash)
    }
}

#[cfg(test)]
mod tests {
    use toml::de;

    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    struct Container {
        #[serde(rename = "output")]
        pub outputs: Outputs,
    }

    const EXAMPLE_OUTPUTS: &'static str = r#"
        [[output]]
        precomputed-hash = "fc3j3vub6kodu4jtfoakfs5xhumqi62m"
        references = ["foo@1.2.3-fc3j3vub6kodu4jtfoakfs5xhumqi62m"]

        [[output]]
        name = "docs"
        precomputed-hash = "fc3j3vub6kodu4jtfoakfs5xhumqi62m"

        [[output]]
        name = "man"
        precomputed-hash = "fc3j3vub6kodu4jtfoakfs5xhumqi62m"
    "#;

    const MISSING_DEFAULT_OUTPUT: &'static str = r#"
        [[output]]
        name = "foo"
        precomputed-hash = "fc3j3vub6kodu4jtfoakfs5xhumqi62m"
    "#;

    const MULTIPLE_DEFAULT_OUTPUTS: &'static str = r#"
        [[output]]
        precomputed-hash = "fc3j3vub6kodu4jtfoakfs5xhumqi62m"
        references = ["foo@1.2.3-fc3j3vub6kodu4jtfoakfs5xhumqi62m"]

        [[output]]
        precomputed-hash = "xpyrto6ighxc4gfhxrexzcrlcdaipars"

        [[output]]
        name = "docs"
        precomputed-hash = "4gw3yobvb2q3uwyu7i4qri3o5bvs2mrt"
    "#;

    #[test]
    fn parse_from_string() {
        let toml: Container = de::from_str(EXAMPLE_OUTPUTS).expect("Failed to deserialize");
        let actual = toml.outputs;
        let dummy_hash: Hash = "fc3j3vub6kodu4jtfoakfs5xhumqi62m"
            .parse()
            .expect("Failed to parse hash from text");
        let dummy_ref = "foo@1.2.3-fc3j3vub6kodu4jtfoakfs5xhumqi62m"
            .parse()
            .expect("Failed to parse ID");

        let mut expected = Outputs::new(dummy_hash.clone(), vec![dummy_ref]);
        let docs_name = "docs".parse().expect("Failed to parse name 'docs'");
        expected.append(docs_name, dummy_hash.clone(), None);
        let man_name = "man".parse().expect("Failed to parse name 'man'");
        expected.append(man_name, dummy_hash.clone(), None);

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_missing_default_output() {
        de::from_str::<Container>(MISSING_DEFAULT_OUTPUT)
            .expect_err("Failed to reject `Outputs` missing default outputs");
    }

    #[test]
    fn rejects_multiple_default_outputs() {
        de::from_str::<Container>(MULTIPLE_DEFAULT_OUTPUTS)
            .expect_err("Failed to reject `Outputs` with multiple default outputs");
    }
}
