use std::path::PathBuf;

use futures_preview::compat::{Compat01As03, Future01CompatExt};
use futures_preview::future::FutureExt;
use futures_preview::io::{AsyncReadExt, AsyncWriteExt};
use tokio::fs::File;
use tokio::io::{ErrorKind, Read, Write};

use super::super::file::FileFutureExt;
use super::{DirFuture, Directory, ReadPath, WritePath};
use crate::id::ManifestId;
use crate::package::Manifest;

#[derive(Clone, Debug)]
pub enum ManifestsInput {
    Manifest(Manifest),
    Path(PathBuf),
    Text(String),
}

#[derive(Debug)]
pub struct ManifestsDir;

impl Directory for ManifestsDir {
    type Id = ManifestId;
    type Input = ManifestsInput;
    type Output = Manifest;

    const NAME: &'static str = "manifests";

    fn precompute_id<'a>(&'a self, input: &'a Self::Input) -> DirFuture<'a, Self::Id> {
        let future = async move {
            match input {
                ManifestsInput::Manifest(ref manifest) => Ok(manifest.compute_id()),
                ManifestsInput::Path(ref path) => {
                    let p = path.to_owned();
                    let mut file = await!(File::open(p).lock_shared().compat()).map_err(|_| ())?;
                    let mut text = String::new();
                    file.read_to_string(&mut text).map_err(|_| ())?;
                    let manifest: Manifest = text.parse().map_err(|_| ())?;
                    Ok(manifest.compute_id())
                }
                ManifestsInput::Text(ref text) => {
                    let manifest: Manifest = text.parse().map_err(|_| ())?;
                    Ok(manifest.compute_id())
                }
            }
        };

        future.boxed()
    }

    fn compute_id<'a>(&'a self, path: &'a ReadPath) -> DirFuture<'a, Self::Id> {
        let future = async move {
            let mut file = await!(path.open_file())?;
            let mut s = String::new();
            file.read_to_string(&mut s).map_err(|_| ())?;
            let manifest: Manifest = s.parse().map_err(|_| ())?;
            Ok(manifest.compute_id())
        };

        future.boxed()
    }

    fn read<'a>(&'a self, path: &'a ReadPath) -> DirFuture<'a, Option<Self::Output>> {
        let future = async move {
            match await!(path.open_file()) {
                // Err(ref e) if e.kind() == ErrorKind::NotFound => Ok(None),
                Err(_) => Err(()),
                Ok(mut file) => {
                    let mut s = String::new();
                    file.read_to_string(&mut s).map_err(|_| ())?;
                    let manifest = s.parse().map_err(|_| ())?;
                    Ok(Some(manifest))
                }
            }
        };

        future.boxed()
    }

    fn write<'a>(
        &'a self,
        path: &'a mut WritePath,
        input: Self::Input,
    ) -> DirFuture<'a, Self::Output> {
        let future = async move {
            println!("doing the thing... {}", path.display());
            let mut file = Compat01As03::new(await!(path.create_file())?);
            println!("succeeded");
            match input {
                ManifestsInput::Manifest(manifest) => {
                    let toml = manifest.to_string();
                    await!(file.write_all(toml.as_bytes())).map_err(|_| ())?;
                    Ok(manifest)
                }
                ManifestsInput::Path(p) => {
                    let toml = {
                        let mut src =
                            await!(File::open(p).lock_shared().compat()).map_err(|_| ())?;
                        let mut toml = String::new();
                        src.read_to_string(&mut toml).map_err(|_| ())?;
                        toml
                    };
                    let manifest = toml.parse().map_err(|_| ())?;
                    await!(file.write_all(toml.as_bytes())).map_err(|_| ())?;
                    Ok(manifest)
                }
                ManifestsInput::Text(text) => {
                    let manifest = text.parse().map_err(|_| ())?;
                    await!(file.write_all(text.as_bytes())).map_err(|_| ())?;
                    Ok(manifest)
                }
            }
        };

        future.boxed()
    }
}
