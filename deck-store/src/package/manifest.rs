use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Error as FmtError, Formatter, Result as FmtResult};
use std::str::FromStr;

use toml::de::Error as DeserializeError;

use super::outputs::Outputs;
use crate::hash::Hash;
use crate::id::{ManifestId, Name, OutputId};

/// TODO: Change to `Uri` once https://github.com/hyperium/http/pull/274 gets merged.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Source {
    Uri { uri: String, hash: String },
    Git,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
struct Package {
    name: Name,
    version: String,
    dependencies: BTreeSet<ManifestId>,
    build_dependencies: BTreeSet<ManifestId>,
    dev_dependencies: BTreeSet<ManifestId>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct Manifest {
    package: Package,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    env: BTreeMap<String, String>,
    #[serde(default, rename = "source", skip_serializing_if = "BTreeSet::is_empty")]
    sources: BTreeSet<Source>,
    #[serde(rename = "output")]
    outputs: Outputs,
}

impl Manifest {
    pub fn build<T: Into<String>>(name: T, version: T, main_output_hash: T) -> ManifestBuilder {
        ManifestBuilder::new(name, version, main_output_hash)
    }

    #[inline]
    pub fn compute_id(&self) -> ManifestId {
        let name = self.package.name.clone();
        let version = self.package.version.clone();
        let hash = Hash::compute().input(&self.to_string()).finish();
        ManifestId::new(name, version, hash)
    }

    #[inline]
    pub fn name(&self) -> &str {
        self.package.name.as_str()
    }

    #[inline]
    pub fn version(&self) -> &str {
        &self.package.version
    }

    #[inline]
    pub fn sources(&self) -> impl Iterator<Item = &Source> {
        self.sources.iter()
    }

    #[inline]
    pub fn dependencies(&self) -> impl Iterator<Item = &ManifestId> {
        self.package.dependencies.iter()
    }

    #[inline]
    pub fn build_dependencies(&self) -> impl Iterator<Item = &ManifestId> {
        self.package.build_dependencies.iter()
    }

    #[inline]
    pub fn dev_dependencies(&self) -> impl Iterator<Item = &ManifestId> {
        self.package.dev_dependencies.iter()
    }

    #[inline]
    pub fn env(&self) -> impl Iterator<Item = (&String, &String)> {
        self.env.iter()
    }

    #[inline]
    pub fn outputs(&self) -> impl Iterator<Item = OutputId> + '_ {
        let name = self.package.name.clone();
        let ver = self.package.version.clone();
        self.outputs.iter_with(name, ver)
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

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s)
    }
}

#[derive(Clone, Debug)]
pub struct ManifestBuilder {
    package: Result<Package, ()>,
    env: BTreeMap<String, String>,
    sources: BTreeSet<Source>,
    outputs: Result<Outputs, ()>,
}

impl ManifestBuilder {
    pub fn new<T: Into<String>>(name: T, version: T, main_output_hash: T) -> Self {
        let package = name.into().parse().map(|name| Package {
            name,
            version: version.into(),
            dependencies: BTreeSet::new(),
            build_dependencies: BTreeSet::new(),
            dev_dependencies: BTreeSet::new(),
        });

        let outputs = main_output_hash
            .into()
            .parse()
            .map(|hash| Outputs::new(hash));

        ManifestBuilder {
            package,
            env: BTreeMap::new(),
            sources: BTreeSet::new(),
            outputs,
        }
    }

    pub fn dependency(mut self, id: ManifestId) -> Self {
        if let Ok(ref mut p) = self.package {
            p.dependencies.insert(id);
        }
        self
    }

    pub fn build_dependency(mut self, id: ManifestId) -> Self {
        if let Ok(ref mut p) = self.package {
            p.build_dependencies.insert(id);
        }
        self
    }

    pub fn dev_dependency(mut self, id: ManifestId) -> Self {
        if let Ok(ref mut p) = self.package {
            p.dev_dependencies.insert(id);
        }
        self
    }

    pub fn source(mut self, source: Source) -> Self {
        self.sources.insert(source);
        self
    }

    pub fn output(mut self, name: Name, hash: Hash) -> Self {
        if let Ok(ref mut out) = self.outputs {
            out.append(name, hash);
        }
        self
    }

    pub fn finish(self) -> Result<Manifest, ()> {
        Ok(Manifest {
            package: self.package?,
            env: self.env,
            sources: self.sources,
            outputs: self.outputs?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MANIFEST: &'static str = r#"
        [package]
        name = "hello"
        version = "1.2.3"
        dependencies = ["thing@1.2.3-fc3j3vub6kodu4jtfoakfs5xhumqi62m"]
        build-dependencies = []
        dev-dependencies = []

        [[source]]
        uri = "https://www.example.com/hello.tar.gz"
        hash = "1234567890abcdef"

        [env]
        LANG = "C_ALL"

        [[output]]
        precomputed-hash = "fc3j3vub6kodu4jtfoakfs5xhumqi62m"

        [[output]]
        name = "doc"
        precomputed-hash = "fc3j3vub6kodu4jtfoakfs5xhumqi62m"

        [[output]]
        name = "man"
        precomputed-hash = "fc3j3vub6kodu4jtfoakfs5xhumqi62m"
    "#;

    #[test]
    fn example_deserialize() {
        let example: Manifest = MANIFEST.parse().expect("Failed to parse manifest");
        println!("{}", example);
    }
}
