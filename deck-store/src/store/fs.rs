use std::fs;
use std::path::PathBuf;

use futures::Stream;
use hyper::{client::HttpConnector, Client};
use hyper_tls::HttpsConnector;

use self::dir::{ManifestsDir, OutputsDir, ReadFuture, SourcesDir, WriteFuture};
// use self::fetcher::{FetchGit, Fetchable};
use self::state::State;
use super::closure::Closure;
use crate::id::{ManifestId, OutputId, SourceId};
use crate::package::{Manifest, Source};

mod dir;
mod fetcher;
mod file;
mod state;

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

    pub fn compute_closure(&self, _package_id: String) -> ReadFuture<Closure> {
        unimplemented!()
    }

    pub fn has_manifest(&self, id: &ManifestId) -> bool {
        let prefix = &self.prefix;
        self.manifests.contains(prefix, id)
    }

    pub fn has_output(&self, id: &OutputId) -> bool {
        let prefix = &self.prefix;
        self.outputs.contains(prefix, id)
    }

    pub fn create_output_dir(&self, package_id: String) -> WriteFuture<OutputId, PathBuf> {
        let prefix = &self.prefix;
        self.outputs.write(prefix, package_id)
    }

    pub fn write_manifest(&self, manifest: Manifest) -> WriteFuture<ManifestId, Manifest> {
        use self::dir::ManifestInput;
        let prefix = &self.prefix;
        let input = ManifestInput::Constructed(manifest);
        self.manifests.write(prefix, input)
    }
}

#[cfg(test)]
mod tests {
    use super::dir::{ManifestInput, ManifestsDir};
    use super::State;
    use super::*;

    use futures::{future, Future};
    use tokio::runtime::Runtime;

    #[test]
    fn create_manifest() {
        let mut runtime = Runtime::new().unwrap();
        let path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/store"));
        println!("{:?}", path);

        let manifests = State::new(ManifestsDir);

        let manifest = Manifest::build("hello", "1.0.0", "fc3j3vub6kodu4jtfoakfs5xhumqi62m", None)
            .finish()
            .expect("failed to create manifest");
        let id = manifest.compute_id();

        let _read1 = manifests.read(&path, &id);
        let _read2 = manifests.read(&path, &id);
        let _read3 = manifests.read(&path, &id);
        let write1 = manifests.write(&path, ManifestInput::Constructed(manifest.clone()));
        let write2 = manifests.write(&path, ManifestInput::Constructed(manifest.clone()));
        let write3 = manifests.write(&path, ManifestInput::Constructed(manifest.clone()));

        let ret = runtime
            .block_on(future::join_all(vec![
                Box::new(write2.map_err(|_| ()).map(|_| ()))
                    as Box<dyn Future<Item = _, Error = _> + Send>,
                Box::new(write3.map_err(|_| ()).map(|_| ()))
                    as Box<dyn Future<Item = _, Error = _> + Send>,
                // Box::new(read3.map_err(|_| ()).map(|_| ()))
                //     as Box<dyn Future<Item = _, Error = _> + Send>,
                Box::new(write1.map_err(|_| ()).map(|_| ()))
                    as Box<dyn Future<Item = _, Error = _> + Send>,
                // Box::new(read1.map_err(|_| ()).map(|_| ()))
                //     as Box<dyn Future<Item = _, Error = _> + Send>,
                // Box::new(read2.map_err(|_| ()).map(|_| ()))
                //     as Box<dyn Future<Item = _, Error = _> + Send>,
            ]))
            .unwrap();

        println!("finished: {:?}", ret);
        runtime.shutdown_on_idle().wait().unwrap();
    }
}
