use std::collections::BTreeSet;
use std::fmt::{Formatter, Result as FmtResult};

use serde::de::{Deserialize, Deserializer, SeqAccess, Visitor};
use serde::ser::{Serialize, SerializeSeq, Serializer};

use super::outputs::{Output, Outputs};

/// TODO: Change to `Uri` once https://github.com/hyperium/http/pull/274 gets merged.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Source {
    Uri { uri: String, hash: String },
    Git,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
struct Entry {
    #[serde(flatten)]
    source: Source,
    target_outputs: BTreeSet<Output>,
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Sources(BTreeSet<Entry>);

impl Sources {
    pub fn new() -> Self {
        Sources(BTreeSet::new())
    }

    #[inline]
    pub fn insert(&mut self, source: Source, target_outputs: BTreeSet<Output>) {
        self.0.insert(Entry {
            source,
            target_outputs,
        });
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &Source> {
        self.0.iter().map(|s| &s.source)
    }

    pub fn all_valid_outputs(&self, outputs: &Outputs) -> bool {
        self.0
            .iter()
            .flat_map(|s| s.target_outputs.iter())
            .all(|out| outputs.contains(out))
    }
}

impl<'de> Deserialize<'de> for Sources {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SourcesVisitor;

        impl<'de> Visitor<'de> for SourcesVisitor {
            type Value = Sources;

            fn expecting(&self, fmt: &mut Formatter) -> FmtResult {
                fmt.write_str("a 'source' table entry with output and an optional name")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut set = BTreeSet::<Entry>::new();
                while let Some(output) = seq.next_element()? {
                    set.insert(output);
                }

                Ok(Sources(set))
            }
        }

        deserializer.deserialize_seq(SourcesVisitor)
    }
}

impl Serialize for Sources {
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
