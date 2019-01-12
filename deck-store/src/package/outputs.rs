use std::collections::BTreeSet;
use std::fmt::{Error as FmtError, Formatter, Result as FmtResult};

use serde::de::{Deserialize, Deserializer, Error as DeError, SeqAccess, Visitor};
use serde::ser::{Serialize, SerializeSeq, Serializer};

use crate::hash::Hash;
use crate::id::{Name, OutputId};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
struct Output {
    #[serde(rename = "name")]
    output_name: Option<Name>,
    precomputed_hash: Hash,
}

impl Output {
    #[inline]
    pub fn new(name: Option<Name>, precomputed_hash: Hash) -> Self {
        Output {
            output_name: name,
            precomputed_hash,
        }
    }

    #[inline]
    fn is_main_output(&self) -> bool {
        self.output_name.is_none()
    }

    fn to_output_id(&self, name: Name, version: String) -> OutputId {
        let output_name = self.output_name.clone();
        let precomputed_hash = self.precomputed_hash.clone();
        OutputId::new(name, output_name, version, precomputed_hash)
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Outputs(BTreeSet<Output>);

impl Outputs {
    pub fn new(precomputed_hash: Hash) -> Self {
        let mut set = BTreeSet::new();
        set.insert(Output::new(None, precomputed_hash));
        Outputs(set)
    }

    #[inline]
    pub fn append(&mut self, name: Name, precomputed_hash: Hash) {
        let output = Output::new(Some(name), precomputed_hash);
        self.0.insert(output);
    }

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
                let mut set = BTreeSet::<Output>::new();

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

        [[output]]
        precomputed-hash = "xpyrto6ighxc4gfhxrexzcrlcdaipars"

        [[output]]
        name = "docs"
        precomputed-hash = "4gw3yobvb2q3uwyu7i4qri3o5bvs2mrt"
    "#;

    #[test]
    fn deserialize() {
        let toml: Container = de::from_str(EXAMPLE_OUTPUTS).expect("Failed to deserialize");
        let actual = toml.outputs;
        let dummy_hash: Hash = "fc3j3vub6kodu4jtfoakfs5xhumqi62m"
            .parse()
            .expect("Failed to parse hash from text");

        let mut expected = Outputs::new(dummy_hash.clone());
        let docs_name = "docs".parse().expect("Failed to parse name 'docs'");
        expected.append(docs_name, dummy_hash.clone());
        let man_name = "man".parse().expect("Failed to parse name 'man'");
        expected.append(man_name, dummy_hash.clone());

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
