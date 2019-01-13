//! Represents the `output` array table in the package manifest.

use std::collections::BTreeSet;
use std::fmt::{Display, Formatter, Result as FmtResult};

use serde::de::{Deserialize, Deserializer, Error as DeError, SeqAccess, Visitor};
use serde::ser::{Serialize, SerializeSeq, Serializer};

use crate::hash::Hash;
use crate::id::{Name, OutputId};

/// Represents the `output` array table in the package manifest.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Outputs(BTreeSet<Entry>);

impl Outputs {
    /// Creates a new `Outputs` table with the main output set to the precomputed hash and inputs.
    ///
    /// # What is meant by a "precomputed hash"
    ///
    /// Since the store follows an intensional model (see Eelco Dolstra's writings on the theorized
    /// "intensional model" within Nix and NixOS), this precomputed hash is used only to identify
    /// compatible trusted substitutes for safe sharing between untrusted users. It may be
    /// rewritten to something else after the builder has been run.
    pub fn new<T>(precomputed_hash: Hash, inputs: T) -> Self
    where
        T: IntoIterator<Item = OutputId>,
    {
        let mut set = BTreeSet::new();
        let inputs = inputs.into_iter().collect();
        set.insert(Entry::new(Output::Main, precomputed_hash, inputs));
        Outputs(set)
    }

    /// Appends a new named output with the given name, precomputed hash [`Hash`], and inputs.
    ///
    /// # What is meant by a "precomputed hash"
    ///
    /// Since the store follows an intensional model (see Eelco Dolstra's writings on the theorized
    /// "intensional model" within Nix and NixOS), this precomputed hash is used only to identify
    /// compatible trusted substitutes for safe sharing between untrusted users. It may be
    /// rewritten to something else after the builder has been run.
    #[inline]
    pub fn append<T>(&mut self, name: Name, precomputed_hash: Hash, inputs: T)
    where
        T: IntoIterator<Item = OutputId>,
    {
        let inputs = inputs.into_iter().collect();
        let output = Entry::new(Output::Named(name), precomputed_hash, inputs);
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
        struct OutputsVisitor;

        impl<'de> Visitor<'de> for OutputsVisitor {
            type Value = Outputs;

            fn expecting(&self, fmt: &mut Formatter) -> FmtResult {
                fmt.write_str("an 'output' table entry with a precomputed hash and optional name")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut set = BTreeSet::<Entry>::new();
                while let Some(output) = seq.next_element()? {
                    set.insert(output);
                }

                let num_main_outputs = set.iter().filter(|out| out.is_main_output()).count();
                if num_main_outputs == 1 {
                    Ok(Outputs(set))
                } else if num_main_outputs > 1 {
                    Err(A::Error::custom("cannot have multiple main outputs"))
                } else {
                    Err(A::Error::custom("missing main output"))
                }
            }
        }

        deserializer.deserialize_seq(OutputsVisitor)
    }
}

impl Serialize for Outputs {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for element in &self.0 {
            seq.serialize_element(element)?;
        }
        seq.end()
    }
}

/// Types of build outputs that a manifest can have.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Output {
    /// The unnamed main output.
    ///
    /// Contains the actual application/library being installed. When a package is installed by a
    /// user, this is likely what they are looking for. Manifests are required to have one and only
    /// one main output.
    Main,
    /// An optional "named" output.
    ///
    /// Contains extra components or artifacts not typically included in a base installation. For
    /// example, the `doc` named output could contain rendered HTML documentation, the `man` named
    /// output could contain man pages, the `debug` named output could contain debugging symbols,
    /// etc. Users can request to install these add-on outputs on top of the main output.
    Named(Name),
}

impl Output {
    /// Returns whether this output is an [`Output::Main`].
    ///
    /// [`Output::Main`]: ./struct.Output.html#variant.Main
    pub fn is_main_output(&self) -> bool {
        *self == Output::Main
    }
}

impl Default for Output {
    fn default() -> Self {
        Output::Main
    }
}

impl Display for Output {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match *self {
            Output::Main => write!(fmt, ""),
            Output::Named(ref name) => write!(fmt, "{}", name),
        }
    }
}

impl Into<Option<Name>> for Output {
    fn into(self) -> Option<Name> {
        match self {
            Output::Main => None,
            Output::Named(name) => Some(name),
        }
    }
}

impl<'de> Deserialize<'de> for Output {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct OutputVisitor;

        impl<'de> Visitor<'de> for OutputVisitor {
            type Value = Output;

            fn expecting(&self, fmt: &mut Formatter) -> FmtResult {
                fmt.write_str("an output name string, e.g. \"\", \"doc\", \"lib\", or \"man\"")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                if value.is_empty() {
                    Ok(Output::Main)
                } else {
                    value
                        .parse()
                        .map_err(|_err| E::custom("failed to deserialize"))
                        .map(|name| Output::Named(name))
                }
            }
        }

        deserializer.deserialize_str(OutputVisitor)
    }
}

impl Serialize for Output {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
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
    inputs: BTreeSet<OutputId>,
}

impl Entry {
    /// Creates a new `Entry` with the given name, precomputed hash, and inputs.
    #[inline]
    pub fn new(output_name: Output, precomputed_hash: Hash, inputs: BTreeSet<OutputId>) -> Self {
        Entry {
            output_name,
            precomputed_hash,
            inputs,
        }
    }

    /// Returns whether this output entry is an [`Output::Main`].
    ///
    /// [`Output::Main`]: ./struct.Output.html#variant.Main
    #[inline]
    fn is_main_output(&self) -> bool {
        self.output_name.is_main_output()
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
        inputs = ["foo@1.2.3-fc3j3vub6kodu4jtfoakfs5xhumqi62m"]

        [[output]]
        name = "docs"
        precomputed-hash = "fc3j3vub6kodu4jtfoakfs5xhumqi62m"

        [[output]]
        name = "man"
        precomputed-hash = "fc3j3vub6kodu4jtfoakfs5xhumqi62m"
    "#;

    const MISSING_MAIN_OUTPUT: &'static str = r#"
        [[output]]
        name = "foo"
        precomputed-hash = "fc3j3vub6kodu4jtfoakfs5xhumqi62m"
    "#;

    const MULTIPLE_MAIN_OUTPUTS: &'static str = r#"
        [[output]]
        precomputed-hash = "fc3j3vub6kodu4jtfoakfs5xhumqi62m"
        inputs = ["foo@1.2.3-fc3j3vub6kodu4jtfoakfs5xhumqi62m"]

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
        let dummy_input = "foo@1.2.3-fc3j3vub6kodu4jtfoakfs5xhumqi62m"
            .parse()
            .expect("Failed to parse ID");

        let mut expected = Outputs::new(dummy_hash.clone(), vec![dummy_input]);
        let docs_name = "docs".parse().expect("Failed to parse name 'docs'");
        expected.append(docs_name, dummy_hash.clone(), None);
        let man_name = "man".parse().expect("Failed to parse name 'man'");
        expected.append(man_name, dummy_hash.clone(), None);

        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_missing_main_output() {
        de::from_str::<Container>(MISSING_MAIN_OUTPUT)
            .expect_err("Failed to reject `Outputs` missing main outputs");
    }

    #[test]
    fn rejects_multiple_main_outputs() {
        de::from_str::<Container>(MULTIPLE_MAIN_OUTPUTS)
            .expect_err("Failed to reject `Outputs` with multiple main outputs");
    }
}
