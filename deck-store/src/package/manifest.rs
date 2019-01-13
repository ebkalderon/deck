use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::fmt::{Display, Error as FmtError, Formatter, Result as FmtResult};
use std::str::FromStr;

use toml::de::Error as DeserializeError;

use super::outputs::{Output, Outputs};
use super::sources::{Source, Sources};
use crate::hash::Hash;
use crate::id::{ManifestId, Name, OutputId};

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
    #[serde(rename = "output")]
    outputs: Outputs,
    #[serde(default, rename = "source", skip_serializing_if = "Sources::is_empty")]
    sources: Sources,
}

impl Manifest {
    /// Builds a new `Manifest` with the given name, version, and main output [`Hash`].
    ///
    /// [`Hash`]: ../struct.Hash.html
    pub fn build<T: Into<String>>(name: T, version: T, main_output_hash: T) -> ManifestBuilder {
        ManifestBuilder::new(name, version, main_output_hash)
    }

    /// Computes the corresponding content-addressable ID of this manifest.
    #[inline]
    pub fn compute_id(&self) -> ManifestId {
        let name = self.package.name.clone();
        let version = self.package.version.clone();
        let hash = Hash::compute().input(&self.to_string()).finish();
        ManifestId::new(name, version, hash)
    }

    /// Returns the name of the package.
    ///
    /// This string is guaranteed not to be empty.
    #[inline]
    pub fn name(&self) -> &str {
        self.package.name.as_str()
    }

    /// Returns the semantic version of the package.
    #[inline]
    pub fn version(&self) -> &str {
        &self.package.version
    }

    /// Iterates over the package's runtime dependencies.
    #[inline]
    pub fn dependencies(&self) -> impl Iterator<Item = &ManifestId> {
        self.package.dependencies.iter()
    }

    /// Iterates over the package's build-time dependencies.
    #[inline]
    pub fn build_dependencies(&self) -> impl Iterator<Item = &ManifestId> {
        self.package.build_dependencies.iter()
    }

    /// Iterates over the package's optional testing dependencies.
    #[inline]
    pub fn dev_dependencies(&self) -> impl Iterator<Item = &ManifestId> {
        self.package.dev_dependencies.iter()
    }

    /// Iterates over the package builder's environment variables as key-value pairs.
    #[inline]
    pub fn env(&self) -> impl Iterator<Item = (OsString, OsString)> + '_ {
        self.env
            .iter()
            .map(|(k, v)| (OsString::from(k), OsString::from(v)))
    }

    /// Iterates over the package's build outputs.
    ///
    /// # Note
    ///
    /// Every package is guaranteed to produce at least one [`Output::Main`] output and zero or more
    /// [`Output::Named`] outputs.
    #[inline]
    pub fn outputs(&self) -> impl Iterator<Item = OutputId> + '_ {
        let name = self.package.name.clone();
        let ver = self.package.version.clone();
        self.outputs.iter_with(name, ver)
    }

    /// Iterates over the package's sources.
    #[inline]
    pub fn sources(&self) -> impl Iterator<Item = &Source> {
        self.sources.iter()
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
    sources: Sources,
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
            sources: Sources::new(),
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

    pub fn output(mut self, name: Name, hash: Hash) -> Self {
        if let Ok(ref mut out) = self.outputs {
            out.append(name, hash);
        }
        self
    }

    pub fn source<T>(mut self, source: Source, target_outputs: T) -> Self
    where
        T: IntoIterator<Item = Output>,
    {
        let outputs = target_outputs.into_iter().collect();
        self.sources.insert(source, outputs);
        self
    }

    pub fn finish(self) -> Result<Manifest, ()> {
        let outputs = self.outputs?;

        if !self.sources.all_valid_outputs(&outputs) {
            return Err(());
        }

        Ok(Manifest {
            package: self.package?,
            env: self.env,
            outputs,
            sources: self.sources,
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
        # FIXME: According to Nix, the package deps should be `ManifestId`s, and each `output`
        # should have its own inputs (?) which are `OutputId`s. How this interacts with `output`s,
        # I'm not sure yet.
        dependencies = ["thing@1.2.3-fc3j3vub6kodu4jtfoakfs5xhumqi62m"]
        build-dependencies = []
        dev-dependencies = []

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

        [[source]]
        uri = "https://www.example.com/hello.tar.gz"
        hash = "1234567890abcdef"
        target-outputs = ["", "doc", "man"]
    "#;

    #[test]
    fn example_deserialize() {
        let example: Manifest = MANIFEST.parse().expect("Failed to parse manifest");
        println!("{}", example);
    }
}
