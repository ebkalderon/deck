use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Error as FmtError, Formatter, Result as FmtResult};
use std::str::FromStr;

use hyper::Uri;
use toml::de::Error as DeserializeError;

/// TODO: Add `Serialize`/`Deserialize` once https://github.com/hyperium/http/pull/274 gets merged.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Source {
    Uri { uri: Uri, hash: String },
    Git,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
struct Package {
    id: String,
    dependencies: BTreeSet<String>,
    build_dependencies: BTreeSet<String>,
    dev_dependencies: BTreeSet<String>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct Manifest {
    package: Package,
    #[serde(skip)]
    sources: Option<Vec<Source>>,
    env: Option<BTreeMap<String, String>>,
}

impl Manifest {
    pub fn build(id: String) -> ManifestBuilder {
        ManifestBuilder::new(id)
    }

    pub fn id(&self) -> &String {
        &self.package.id
    }

    pub fn sources(&self) -> impl Iterator<Item = &Source> {
        self.sources.iter().flat_map(|src| src)
    }

    pub fn dependencies(&self) -> impl Iterator<Item = &String> {
        self.package.dependencies.iter()
    }

    pub fn build_dependencies(&self) -> impl Iterator<Item = &String> {
        self.package.build_dependencies.iter()
    }

    pub fn dev_dependencies(&self) -> impl Iterator<Item = &String> {
        self.package.dev_dependencies.iter()
    }
}

impl Display for Manifest {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        toml::to_string(self)
            .map_err(|e| {
                println!("couldn't display self: {}", e);
                FmtError::default()
            })
            .and_then(|s| write!(fmt, "{}", s))
    }
}

impl FromStr for Manifest {
    type Err = DeserializeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s)
    }
}

#[derive(Clone, Debug)]
pub struct ManifestBuilder {
    package: Package,
    sources: Option<Vec<Source>>,
    env: Option<BTreeMap<String, String>>,
}

impl ManifestBuilder {
    pub fn new(id: String) -> Self {
        ManifestBuilder {
            package: Package {
                id,
                dependencies: BTreeSet::new(),
                build_dependencies: BTreeSet::new(),
                dev_dependencies: BTreeSet::new(),
            },
            sources: None,
            env: None,
        }
    }

    pub fn source(mut self, source: Source) -> Self {
        {
            let sources = self.sources.get_or_insert_with(|| Vec::new());
            sources.push(source);
        }
        self
    }

    pub fn dependency(mut self, package_id: String) -> Self {
        self.package.dependencies.insert(package_id);
        self
    }

    pub fn build_dependency(mut self, package_id: String) -> Self {
        self.package.dependencies.insert(package_id);
        self
    }

    pub fn dev_dependency(mut self, package_id: String) -> Self {
        self.package.dependencies.insert(package_id);
        self
    }

    pub fn finish(self) -> Manifest {
        Manifest {
            package: self.package,
            sources: self.sources,
            env: self.env,
        }
    }
}
