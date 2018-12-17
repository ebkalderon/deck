use std::path::{Path, PathBuf};

use futures::{future, Future, Stream};
use hyper::Chunk;
use tokio::fs::File;
use tokio::io::Write;

use super::super::file::FileFutureExt;
use super::{Directory, DirectoryFuture, IdFuture, ReadFuture, WriteFuture};
use package::Source;

#[derive(Debug)]
pub struct SourcesDir;

impl Directory for SourcesDir {
    type Id = String;
    type Input = (Source, Box<dyn Stream<Item = Chunk, Error = ()> + Send>);
    type Output = PathBuf;

    const NAME: &'static str = "sources";

    // Compute temporary ID from the `Source`.
    fn compute_id(&self, input: &Self::Input) -> IdFuture<Self::Id> {
        let id = match input.0 {
            Source::Uri { ref uri, ref hash } => {
                let file = Path::new(uri.path())
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("index.html");
                format!("{}-{}", file, hash)
            }
            Source::Git => unimplemented!(),
        };

        Box::new(future::ok(id))
    }

    // If the `target` ID exists, we get the path to the source. Otherwise, nothing.
    fn read(&self, target: &Path, _: &Self::Id) -> ReadFuture<Self::Output> {
        if target.exists() {
            DirectoryFuture::new(future::ok(Some(target.to_owned())))
        } else {
            DirectoryFuture::new(future::ok(None))
        }
    }

    // Take a `Source` and write it to disk, reporting the progress as it goes.
    //
    // # Issues
    //
    // * If this is a `Source::Uri`, we use `hyper::Client` to download the URI. The problem is, we
    //   need to report the progress somehow. We can't return a stream here, because `Self::Output`
    //   is also used in `read()`, and also we can't use the `Progress` type because it's meant to
    //   be used by the `FetchSource` job, not here.
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
            }).and_then(|_| /* TODO: validate file's hash here. */ Ok(()))
            .map(move |_| (id, output_path));

        DirectoryFuture::new(write_file)
    }
}
