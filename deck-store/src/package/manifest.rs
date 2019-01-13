use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::fmt::{Display, Error as FmtError, Formatter, Result as FmtResult};
use std::str::FromStr;

use toml::de::Error as DeserializeError;

use super::outputs::Outputs;
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
    /// Constructs a `Manifest` with the given name, version, main output [`Hash`], and inputs.
    ///
    /// [`Hash`]: ../struct.Hash.html
    pub fn build<T, U>(name: T, version: T, main_output_hash: T, inputs: U) -> ManifestBuilder
    where
        T: Into<String>,
        U: IntoIterator<Item = OutputId>,
    {
        ManifestBuilder::new(name, version, main_output_hash, inputs)
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
    ///
    /// # Example
    ///
    /// ```
    /// # use deck_store::package::Manifest;
    /// #
    /// let manifest = Manifest::build("foo", "1.0.0", "fc3j3vub6kodu4jtfoakfs5xhumqi62m")
    ///      .finish()
    ///      .unwrap();
    ///
    /// let name = manifest.name();
    /// assert_eq!(name, "foo");
    /// ```
    #[inline]
    pub fn name(&self) -> &str {
        self.package.name.as_str()
    }

    /// Returns the semantic version of the package.
    ///
    /// # Example
    ///
    /// ```
    /// # use deck_store::package::Manifest;
    /// #
    /// let manifest = Manifest::build("foo", "1.0.0", "fc3j3vub6kodu4jtfoakfs5xhumqi62m")
    ///      .finish()
    ///      .unwrap();
    ///
    /// let version = manifest.version();
    /// assert_eq!(version, "1.0.0");
    /// ```
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
    /// Every package is guaranteed to produce at least one main output and zero or more additional
    /// outputs. When a manifest is built from source, all outputs are built together.
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
    /// Constructs a `Manifest` with the given name, version, main output [`Hash`], and inputs.
    ///
    /// [`Hash`]: ../struct.Hash.html
    pub fn new<T, U>(name: T, version: T, main_output_hash: T, inputs: U) -> Self
    where
        T: Into<String>,
        U: IntoIterator<Item = OutputId>,
    {
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
            .map(|hash| Outputs::new(hash, inputs));

        ManifestBuilder {
            package,
            env: BTreeMap::new(),
            sources: Sources::new(),
            outputs,
        }
    }

    /// Adds a runtime dependency on `id`.
    pub fn dependency(mut self, id: ManifestId) -> Self {
        if let Ok(ref mut p) = self.package {
            p.dependencies.insert(id);
        }
        self
    }

    /// Adds a build dependency on `id`.
    ///
    /// # Availability
    ///
    /// This kind of dependency is only downloaded when the package is being built from source.
    /// Otherwise, the dependency is ignored. Artifacts from build dependencies cannot be linked to
    /// at runtime.
    pub fn build_dependency(mut self, id: ManifestId) -> Self {
        if let Ok(ref mut p) = self.package {
            p.build_dependencies.insert(id);
        }
        self
    }

    /// Adds a test-only dependency on `id`.
    ///
    /// # Availability
    ///
    /// This kind of dependency is only downloaded when the package is being built from source and
    /// running tests is enabled. Otherwise, the dependency is ignored. Artifacts from dev
    /// dependencies cannot be linked to at runtime, and they are never included in the final
    /// output.
    pub fn dev_dependency(mut self, id: ManifestId) -> Self {
        if let Ok(ref mut p) = self.package {
            p.dev_dependencies.insert(id);
        }
        self
    }

    /// Declares an additional build output directory produced by this manifest.
    ///
    /// Build output directories can accept other build outputs as inputs, allowing them to be
    /// symlinked into the directory structure for runtime dependencies.
    ///
    /// By default, all manifests produce a single [`Output::Main`]. This method allows for
    /// secondary outputs to be added (known as [`Output::Named`]) with supplementary content, e.g.
    /// documentation, man pages, debug info, etc.
    ///
    /// [`Output::Main`]: ./enum.Output.html
    /// [`Output::Named`]: ./enum.Output.html
    pub fn output<T>(mut self, name: Name, precomputed_hash: Hash, inputs: T) -> Self
    where
        T: IntoIterator<Item = OutputId>,
    {
        if let Ok(ref mut out) = self.outputs {
            out.append(name, precomputed_hash, inputs);
        }
        self
    }

    /// Adds an external fetchable source to this manifest.
    pub fn source<T>(mut self, source: Source) -> Self {
        self.sources.insert(source);
        self
    }

    pub fn finish(self) -> Result<Manifest, ()> {
        Ok(Manifest {
            package: self.package?,
            env: self.env,
            outputs: self.outputs?,
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
        inputs = ["thing@1.2.3:bin-fc3j3vub6kodu4jtfoakfs5xhumqi62m"]

        [[output]]
        name = "doc"
        precomputed-hash = "fc3j3vub6kodu4jtfoakfs5xhumqi62m"
        inputs = ["thing@1.2.3-fc3j3vub6kodu4jtfoakfs5xhumqi62m"]

        [[output]]
        name = "man"
        precomputed-hash = "fc3j3vub6kodu4jtfoakfs5xhumqi62m"

        [[source]]
        uri = "https://www.example.com/hello.tar.gz"
        hash = "1234567890abcdef"
    "#;

    #[test]
    fn example_deserialize() {
        let example: Manifest = MANIFEST.parse().expect("Failed to parse manifest");
        println!("{}", example);
    }
}
