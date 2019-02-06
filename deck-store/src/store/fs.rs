use std::fs;
use std::path::PathBuf;

use hyper::{client::HttpConnector, Client};
use hyper_tls::HttpsConnector;

use self::dir::{ManifestsDir, OutputsDir, SourcesDir};
use self::state::State;
use super::closure::Closure;
use crate::id::{ManifestId, OutputId, SourceId};
use crate::package::{Manifest, Source};

mod dir;
mod fetcher;
mod file;
mod state;

const TEMP_DIR_NAME: &str = "tmp";
const VAR_DIR_NAME: &str = "var";

pub(crate) type HttpsClient = Client<HttpsConnector<HttpConnector>>;

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
        use self::dir::ManifestsInput;
        let prefix = &self.prefix;
        let input = ManifestsInput::Manifest(manifest);
        let (_, out) = await!(self.manifests.write(prefix, input))?;
        Ok(out)
    }
}
