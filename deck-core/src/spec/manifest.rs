use super::Specifier;
use crate::hash::Hash;
use crate::id::ManifestId;
use crate::name::Name;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ManifestSpec {
    name: Name,
    version: Option<String>,
    hash: Option<Hash>,
}

impl ManifestSpec {
    pub const fn new(name: Name, version: Option<String>, hash: Option<Hash>) -> Self {
        ManifestSpec {
            name,
            version,
            hash,
        }
    }

    #[inline]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    #[inline]
    pub fn version(&self) -> Option<&String> {
        self.version.as_ref()
    }

    #[inline]
    pub fn hash(&self) -> Option<&Hash> {
        self.hash.as_ref()
    }
}

impl Specifier for ManifestSpec {
    type Id = ManifestId;

    fn matches(&self, id: &Self::Id) -> bool {
        let name_matches = self.name == id.name();
        let version_matches = self
            .version
            .as_ref()
            .map(|ver| ver == id.version())
            .unwrap_or(true);
        let hash_matches = self
            .hash
            .as_ref()
            .map(|hash| hash == id.hash())
            .unwrap_or(true);

        name_matches && version_matches && hash_matches
    }
}
