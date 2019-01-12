use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

use crate::id::ManifestId;
use crate::package::Manifest;

#[derive(Clone, Debug)]
pub struct Closure {
    target: ManifestId,
    packages: Arc<BTreeMap<ManifestId, Manifest>>,
}

impl Closure {
    /// Creates a new `Closure` for the given target `ManifestId` with the specified `packages`.
    pub fn new(target: ManifestId, packages: HashSet<Manifest>) -> Result<Self, ()> {
        let with_ids: BTreeMap<ManifestId, Manifest> = packages
            .into_iter()
            .map(|manifest| (manifest.compute_id(), manifest))
            .collect();

        validate_graph(target.clone(), &with_ids)?;

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
    pub fn dependent_closures<'a>(&'a self) -> impl Iterator<Item = Closure> + 'a {
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

fn validate_graph(target: ManifestId, packages: &BTreeMap<ManifestId, Manifest>) -> Result<(), ()> {
    let deps: Vec<_> = match packages.get(&target) {
        Some(ref pkg) => pkg.dependencies().cloned().collect(),
        None => return Err(()),
    };

    for dep in deps {
        if dep == target {
            return Err(());
        }

        if packages.contains_key(&dep) {
            return validate_graph(dep.clone(), packages);
        } else {
            return Err(());
        }
    }

    Ok(())
}
