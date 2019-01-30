use std::fs;
use std::path::PathBuf;

use futures::Stream;
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

#[cfg(test)]
mod tests {
    use super::*;

    use futures_preview::future::{FutureExt, TryFutureExt};
    use futures::Future;
    use tokio::runtime::current_thread::block_on_all;

    #[ignore]
    #[test]
    fn create_manifest() {
        let path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/store"));
        println!("{:?}", path);

        let store = StoreDir::open(path).unwrap();
        let manifest = Manifest::build("hello", "1.0.0", "fc3j3vub6kodu4jtfoakfs5xhumqi62m", None)
            .finish()
            .expect("failed to create manifest");

        // FIXME: `tokio::fs` requires a `tokio` executor, but `StoreDir` produces a non-`'static`
        // future which `tokio` cannot execute. `tokio::current_thread::block_on_all()` can execute
        // them, but it panics on `tokio::fs::File` because the entire `tokio-fs` crate doesn't
        // work with `current_thread` and requires a `tokio`-specific threadpool with
        // `::blocking()`. This test will be ignored for now. See this for details:
        // https://github.com/tokio-rs/tokio/issues/386
        let write1 = store.write_manifest(manifest.clone()).boxed().compat();
        block_on_all(write1).unwrap();
    }
}
