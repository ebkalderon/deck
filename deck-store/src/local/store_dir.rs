use std::fs;
use std::path::PathBuf;

use deck_core::{Manifest, ManifestId, OutputId, Source, SourceId};

use self::manifests::{ManifestsDir, ManifestsInput};
use self::outputs::OutputsDir;
use self::sources::SourcesDir;
use super::dir::State;
use crate::closure::Closure;

mod manifests;
mod outputs;
mod sources;

#[derive(Debug)]
pub struct StoreDir {
    prefix: PathBuf,
    manifests: State<ManifestsDir>,
    outputs: State<OutputsDir>,
    sources: State<SourcesDir>,
}

impl StoreDir {
    pub fn open(path: PathBuf) -> Result<Self, ()> {
        let prefix = fs::read_dir(&path)
            .map_err(|_| ())
            .and_then(|_| fs::canonicalize(path).map_err(|_| ()))?;

        Ok(StoreDir {
            prefix,
            manifests: State::new(ManifestsDir),
            outputs: State::new(OutputsDir),
            sources: State::new(SourcesDir),
        })
    }

    pub async fn compute_closure(&self, _id: ManifestId) -> Option<Closure> {
        unimplemented!()
    }

    pub fn contains_output(&self, id: &OutputId) -> bool {
        let prefix = &self.prefix;
        self.outputs.contains(prefix, id)
    }

    pub async fn write_manifest(&self, manifest: Manifest) -> Result<Manifest, ()> {
        let prefix = &self.prefix;
        let input = ManifestsInput::Manifest(manifest);
        let (_, out) = await!(self.manifests.write(prefix, input))?;
        Ok(out)
    }
}
