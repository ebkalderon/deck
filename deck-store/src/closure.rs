//! Self-contained dependency graph for a set of packages.

use std::collections::{BTreeMap, HashSet};
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::sync::Arc;

use deck_core::{Manifest, ManifestId, OutputId};

type Result<T> = std::result::Result<T, ClosureError>;

/// Self-contained dependency graph for a set of packages.
#[derive(Clone, Debug)]
pub struct Closure {
    target: ManifestId,
    packages: Arc<BTreeMap<ManifestId, Manifest>>,
}

impl Closure {
    /// Creates a new `Closure` for the given target `ManifestId` with the specified `packages`.
    pub fn new(target: ManifestId, packages: HashSet<Manifest>) -> Result<Self> {
        let with_ids: BTreeMap<ManifestId, Manifest> = packages
            .into_iter()
            .map(|manifest| (manifest.compute_id(), manifest))
            .collect();

        validate_closure(target.clone(), &with_ids)?;

        Ok(Closure {
            target,
            packages: Arc::new(with_ids),
        })
    }

    /// Returns the target `ManifestId` represented by this closure.
    #[inline]
    pub fn target(&self) -> &ManifestId {
        &self.target
    }

    /// Returns the target `Manifest` represented by this closure.
    #[inline]
    pub fn target_manifest(&self) -> &Manifest {
        &self.packages[&self.target]
    }

    /// Returns a set of sub-closures for each dependency of the target.
    #[inline]
    pub fn dependent_closures(&self) -> impl Iterator<Item = Closure> + '_ {
        let packages = self.packages.clone();
        self.target_manifest()
            .dependencies()
            .cloned()
            .map(move |dep| Closure {
                target: dep,
                packages: packages.clone(),
            })
    }
}

/// Checks the given set of packages against the target `ManifestId` and checks whether the
/// essential properties hold, namely:
///
/// 1. For this closure and all dependent closures, `target` must be contained within `packages`.
/// 2. For all dependencies in this closure, there must be no direct cycles (however, note that
///    filesystem-level self-references within an output are allowed).
/// 3. For all outputs specified in `target`, each set of references must correspond to exactly one
///    declared dependency. Undeclared references and references to build/dev dependencies are
///    disallowed.
fn validate_closure(target: ManifestId, packages: &BTreeMap<ManifestId, Manifest>) -> Result<()> {
    let manifest = packages
        .get(&target)
        .ok_or_else(|| ClosureError::MissingTarget(target.clone()))?;

    for dep in manifest.dependencies() {
        if *dep == target {
            return Err(ClosureError::CycleDetected(target));
        }

        if packages.contains_key(&dep) {
            return validate_closure(dep.clone(), packages);
        } else {
            return Err(ClosureError::MissingDependency {
                package: target,
                dependency: dep.clone(),
            });
        }
    }

    for out in manifest.outputs() {
        if !manifest.dependencies().any(|dep| dep.is_same_package(&out)) {
            return Err(ClosureError::InvalidInput {
                package: target,
                input: out.clone(),
            });
        }
    }

    Ok(())
}

/// Types of errors that can occur while constructing and validating closures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClosureError {
    /// A package contained a dependency on itself.
    CycleDetected(ManifestId),
    /// A package references an output that is not declared in the package dependencies.
    InvalidInput {
        /// Package which contained the invalid reference.
        package: ManifestId,
        /// The invalid reference in question.
        input: OutputId,
    },
    /// Closure for `package` lacks the manifest information for a required dependency.
    MissingDependency {
        /// Package's closure being evaluated.
        package: ManifestId,
        /// The missing dependency in question.
        dependency: ManifestId,
    },
    /// Closure lacks the manifest information for its own target.
    MissingTarget(ManifestId),
}

impl Display for ClosureError {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        use self::ClosureError::*;
        match *self {
            CycleDetected(ref pkg) => {
                write!(fmt, "manifest {} contains a dependency on itself", pkg)
            }
            InvalidInput {
                ref package,
                ref input,
            } => write!(
                fmt,
                "manifest {} references output {}, but its parent package is not in `dependencies`",
                package, input
            ),
            MissingDependency {
                ref package,
                ref dependency,
            } => write!(
                fmt,
                "closure for {} is missing manifest information for dependency {}",
                package, dependency
            ),
            MissingTarget(ref pkg) => write!(
                fmt,
                "closure for {} is missing manifest information of its target",
                pkg
            ),
        }
    }
}

impl Error for ClosureError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
