//! Represents the `source` array table in the package manifest.

use std::collections::BTreeSet;

/// External fetchable source that can be cached in the store.
///
/// TODO: Change to `Uri` once https://github.com/hyperium/http/pull/274 gets merged.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Source {
    Uri { uri: String, hash: String },
    Git,
}

/// Represents the `source` array table in the package manifest.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct Sources(BTreeSet<Source>);

impl Sources {
    /// Creates a new empty `Sources` table.
    pub fn new() -> Self {
        Sources(BTreeSet::new())
    }

    /// Inserts a new [`Source`] into the table.
    ///
    /// [`Source`]: ./enum.Source.html
    #[inline]
    pub fn insert(&mut self, source: Source) {
        self.0.insert(source);
    }

    /// Returns whether the table is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Iterates over each [`Source`] in the table.
    ///
    /// [`Source`]: ./enum.Source.html
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &Source> {
        self.0.iter()
    }
}
