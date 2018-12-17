use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

use package::Manifest;

#[derive(Clone, Debug)]
pub struct Closure {
    pub target: String,
    pub packages: Arc<BTreeMap<String, Manifest>>,
}

impl Closure {
    pub fn new(target: String, packages: HashSet<Manifest>) -> Result<Self, ()> {
        let with_ids: BTreeMap<String, Manifest> = packages
            .into_iter()
            .map(|manifest| (manifest.id().clone(), manifest))
            .collect();

        if !with_ids.contains_key(&target) {
            return Err(());
        }

        Ok(Closure {
            target,
            packages: Arc::new(with_ids),
        })
    }

    pub fn contains(&self, package: &String) -> bool {
        self.packages.contains_key(package)
    }
}
