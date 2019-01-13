use std::path::{Path, PathBuf};

use futures::{future, Future, Stream};
use hyper::Chunk;
use tokio::fs::File;
use tokio::io::Write;

use super::super::file::FileFutureExt;
use super::{Directory, DirectoryFuture, IdFuture, ReadFuture, WriteFuture};
use crate::id::SourceId;
use crate::package::Source;

#[derive(Debug)]
pub struct SourcesDir;

impl Directory for SourcesDir {
    type Id = SourceId;
    type Input = (Source, Box<dyn Stream<Item = Chunk, Error = ()> + Send>);
    type Output = PathBuf;

    const NAME: &'static str = "sources";

    fn precompute_id(&self, input: &Self::Input) -> IdFuture<Self::Id> {
        let id = match input.0 {
            Source::Git => unimplemented!(),
            Source::Path { ref path, ref hash } => unimplemented!(),
            Source::Uri { ref uri, ref hash } => {
                let file = Path::new(uri)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("index.html");
                format!("{}-{}", file, hash).parse().unwrap()
            }
        };

        Box::new(future::ok(id))
    }

    fn compute_id(&self, _target: &Path) -> IdFuture<Self::Id> {
        unimplemented!()
    }

    // If the `target` ID exists, we get the path to the source. Otherwise, nothing.
    fn read(&self, target: &Path, _: &Self::Id) -> ReadFuture<Self::Output> {
        if target.exists() {
            DirectoryFuture::new(future::ok(Some(target.to_owned())))
        } else {
            DirectoryFuture::new(future::ok(None))
        }
    }

    // Take a `Source` or a `Path` and write it to the directory, computing a new ID if needed.
    fn write(&self, target: &Path, input: Self::Input) -> WriteFuture<Self::Id, Self::Output> {
        let output_path = target.to_owned();
        let id = target
            .file_name()
            .and_then(|id| id.to_str())
            .unwrap_or("index.html")
            .to_string();

        let (_, stream) = input;
        let write_file = File::create(output_path.clone())
            .lock_exclusive()
            .map_err(|_| ())
            .and_then(|mut file| {
                stream.for_each(move |chunk| file.write_all(&chunk).map_err(|_| ()))
            })
            .and_then(|_| /* TODO: validate file's hash here. */ Ok(()))
            .map(move |_| (id.parse().unwrap(), output_path));

        DirectoryFuture::new(write_file)
    }
}
