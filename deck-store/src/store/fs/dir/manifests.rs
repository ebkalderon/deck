use std::path::Path;
use std::str::FromStr;

use futures::{future, Future};
use tokio::fs::File;
use tokio::io::{ErrorKind, Read, Write};

use super::super::file::FileFutureExt;
use super::{Directory, DirectoryFuture, IdFuture, ReadFuture, WriteFuture};
use package::Manifest;

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum ManifestInput {
    Constructed(Manifest),
    Text(String),
}

#[derive(Debug)]
pub struct ManifestsDir;

impl Directory for ManifestsDir {
    type Id = String;
    type Input = ManifestInput;
    type Output = Manifest;

    const NAME: &'static str = "manifests";

    fn compute_id(&self, input: &Self::Input) -> IdFuture<Self::Id> {
        match *input {
            ManifestInput::Constructed(ref m) => {
                let _output = m.to_string();
                // TODO: Compute hash here.
                Box::new(future::lazy(|| Ok("hello".to_string())))
            }
            ManifestInput::Text(ref _s) => {
                // TODO: Compute hash here.
                Box::new(future::lazy(|| Ok("hello".to_string())))
            }
        }
    }

    fn read(&self, target: &Path, _id: &Self::Id) -> ReadFuture<Self::Output> {
        let opening = File::open(target.to_owned())
            .lock_shared()
            .then(|result| match result {
                Ok(manifest) => Ok(Some(manifest)),
                Err(ref e) if e.kind() == ErrorKind::NotFound => Ok(None),
                Err(e) => Err(e),
            });

        let reading = opening.map_err(|_| ()).map(|file| {
            if let Some(mut file) = file {
                let mut s = String::new();
                file.read_to_string(&mut s).expect("failed to read");
                Some(Manifest::from_str(&s).expect("failed to deserialize"))
            } else {
                None
            }
        });

        DirectoryFuture::new(reading)
    }

    fn write(&self, target: &Path, input: Self::Input) -> WriteFuture<Self::Id, Self::Output> {
        let creating = File::create(target.to_owned())
            .lock_exclusive()
            .map_err(|_| ());

        let writing = creating.map_err(|_| ()).map(|mut file| {
            let manifest = match input {
                ManifestInput::Constructed(manifest) => {
                    let toml = manifest.to_string();
                    write!(file, "{}", toml).expect("failed to write file");
                    manifest
                }
                ManifestInput::Text(text) => {
                    let manifest = Manifest::from_str(&text).expect("failed to deserialize text");
                    write!(file, "{}", text).expect("failed to write file");
                    manifest
                }
            };

            (manifest.id().clone(), manifest)
        });

        DirectoryFuture::new(writing)
    }
}
