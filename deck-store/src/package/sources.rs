use std::collections::BTreeSet;

/// TODO: Change to `Uri` once https://github.com/hyperium/http/pull/274 gets merged.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Source {
    Uri { uri: String, hash: String },
    Git,
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct Sources(BTreeSet<Source>);

impl Sources {
    pub fn new() -> Self {
        Sources(BTreeSet::new())
    }

    #[inline]
    pub fn insert(&mut self, source: Source) {
        self.0.insert(source);
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &Source> {
        self.0.iter()
    }
}
