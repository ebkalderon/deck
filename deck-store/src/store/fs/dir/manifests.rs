use std::path::Path;

use futures::{future, Future};
use tokio::fs::File;
use tokio::io::{ErrorKind, Read, Write};

use super::super::file::FileFutureExt;
use super::{Directory, DirectoryFuture, IdFuture, ReadFuture, WriteFuture};
use crate::id::ManifestId;
use crate::package::Manifest;

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum ManifestInput {
    Constructed(Manifest),
    Text(String),
}

#[derive(Debug)]
pub struct ManifestsDir;

impl Directory for ManifestsDir {
    type Id = ManifestId;
    type Input = ManifestInput;
    type Output = Manifest;

    const NAME: &'static str = "manifests";

    fn precompute_id(&self, input: &Self::Input) -> IdFuture<Self::Id> {
        match *input {
            ManifestInput::Constructed(ref manifest) => {
                let id = manifest.compute_id();
                Box::new(future::ok(id))
            }
            ManifestInput::Text(ref text) => {
                let text = text.clone();
                Box::new(future::lazy(move || {
                    let manifest: Manifest = text.parse().map_err(|_| ())?;
                    Ok(manifest.compute_id())
                }))
            }
        }
    }

    fn compute_id(&self, target: &Path) -> IdFuture<Self::Id> {
        let computing = File::open(target.to_owned())
            .lock_shared()
            .map_err(|_| ())
            .and_then(|mut file| {
                let mut s = String::new();
                file.read_to_string(&mut s).map_err(|_| ())?;
                let manifest: Manifest = s.parse().map_err(|_| ())?;
                Ok(manifest.compute_id())
            });

        Box::new(computing)
    }

    fn read(&self, target: &Path, _id: &Self::Id) -> ReadFuture<Self::Output> {
        let opening = File::open(target.to_owned())
            .lock_shared()
            .then(|result| match result {
                Ok(manifest) => Ok(Some(manifest)),
                Err(ref e) if e.kind() == ErrorKind::NotFound => Ok(None),
                Err(e) => Err(e),
            });

        let reading = opening.map_err(|_| ()).and_then(|file| {
            if let Some(mut file) = file {
                let mut s = String::new();
                file.read_to_string(&mut s).map_err(|_| ())?;
                let manifest = s.parse().map_err(|_| ())?;
                Ok(Some(manifest))
            } else {
                Ok(None)
            }
        });

        DirectoryFuture::new(reading)
    }

    fn write(&self, target: &Path, input: Self::Input) -> WriteFuture<Self::Id, Self::Output> {
        let creating = File::create(target.to_owned())
            .lock_exclusive()
            .map_err(|_| ());

        let writing = creating.and_then(|mut file| {
            let manifest = match input {
                ManifestInput::Constructed(manifest) => {
                    let toml = manifest.to_string();
                    write!(file, "{}", toml).map_err(|_| ())?;
                    manifest
                }
                ManifestInput::Text(text) => {
                    let manifest = text.parse().map_err(|_| ())?;
                    write!(file, "{}", text).map_err(|_| ())?;
                    manifest
                }
            };

            Ok((manifest.compute_id(), manifest))
        });

        DirectoryFuture::new(writing)
    }
}
