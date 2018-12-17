pub use self::closure::Closure;

use std::fs;
use std::path::PathBuf;

use futures::Stream;
use hyper::Chunk;

use self::dir::{ManifestsDir, OutputsDir, ReadFuture, SourcesDir, WriteFuture};
use self::state::State;
use package::{Manifest, Source};

mod closure;
mod dir;
mod file;
mod state;

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

    pub fn compute_closure(&self, package_id: String) -> ReadFuture<Closure> {
        unimplemented!()
    }

    pub fn create_output_dir(&self, package_id: String) -> WriteFuture<String, PathBuf> {
        let prefix = &self.prefix;
        self.outputs.write(prefix, package_id)
    }

    pub fn write_source_http<S>(&self, source: Source, stream: S) -> WriteFuture<String, PathBuf>
    where
        S: Stream<Item = Chunk, Error = ()> + Send + 'static,
    {
        let prefix = &self.prefix;
        self.sources.write(prefix, (source, Box::new(stream)))
    }

    pub fn write_manifest(&self, manifest: String) -> WriteFuture<String, Manifest> {
        use self::dir::ManifestInput;
        let prefix = &self.prefix;
        let input = ManifestInput::Text(manifest);
        self.manifests.write(prefix, input)
    }
}

#[cfg(test)]
mod tests {
    use super::dir::{Directory, ManifestInput, ManifestsDir};
    use super::State;
    use super::*;

    use futures::{future, Future};
    use tokio::fs::{self, File};
    use tokio::runtime::Runtime;

    #[test]
    fn create_manifest() {
        let mut runtime = Runtime::new().unwrap();
        let path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/store"));
        println!("{:?}", path);

        let manifests = State::new(ManifestsDir);

        let read1 = manifests.read(&path, &"hello".into());
        let read2 = manifests.read(&path, &"hello".into());
        let read3 = manifests.read(&path, &"hello".into());
        let write1 = manifests.write(
            &path,
            ManifestInput::Constructed(Manifest::build("hello".into()).finish()),
        );
        let write2 = manifests.write(
            &path,
            ManifestInput::Constructed(Manifest::build("hello".into()).finish()),
        );
        let write3 = manifests.write(
            &path,
            ManifestInput::Constructed(Manifest::build("hello".into()).finish()),
        );

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
            ])).unwrap();

        println!("finished: {:?}", ret);
        runtime.shutdown_on_idle().wait().unwrap();
    }
}
