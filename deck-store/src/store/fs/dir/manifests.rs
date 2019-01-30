use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;

use futures_preview::future::FutureExt;
use futures_preview::compat::Future01CompatExt;
use tokio::fs::File;
use tokio::io::{ErrorKind, Read, Write};

use super::super::file::FileFutureExt;
use super::Directory;
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

    type IdFuture = Pin<Box<dyn Future<Output = Result<Self::Id, ()>> + Send>>;
    type ReadFuture = Pin<Box<dyn Future<Output = Result<Option<Self::Output>, ()>> + Send>>;
    type WriteFuture = Pin<Box<dyn Future<Output = Result<Self::Output, ()>> + Send>>;

    const NAME: &'static str = "manifests";

    fn precompute_id(&self, input: &Self::Input) -> Self::IdFuture {
        let input = input.clone();
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

    fn compute_id(&self, target: &Path) -> Self::IdFuture {
        let p = target.to_owned();
        let future = async {
            let mut file = await!(File::open(p).lock_shared().compat()).map_err(|_| ())?;
            let mut s = String::new();
            file.read_to_string(&mut s).map_err(|_| ())?;
            let manifest: Manifest = s.parse().map_err(|_| ())?;
            Ok(manifest.compute_id())
        };

        future.boxed()
    }

    fn read(&self, target: &Path, _id: &Self::Id) -> Self::ReadFuture {
        let p = target.to_owned();
        let future = async {
            match await!(File::open(p).lock_shared().compat()) {
                Err(ref e) if e.kind() == ErrorKind::NotFound => Ok(None),
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

    fn write(&self, target: &Path, input: Self::Input) -> Self::WriteFuture {
        let p = target.to_owned();
        let future = async {
            println!("doing the thing... {}", p.display());
            let mut file = await!(File::create(p).lock_exclusive().compat()).map_err(|e| println!("{}", e))?;
            println!("succeeded");
            match input {
                ManifestsInput::Manifest(manifest) => {
                    let toml = manifest.to_string();
                    write!(file, "{}", toml).map_err(|_| ())?;
                    Ok(manifest)
                }
                ManifestsInput::Path(p) => {
                    let toml = {
                        let mut src = await!(File::open(p).lock_shared().compat()).map_err(|_| ())?;
                        let mut toml = String::new();
                        src.read_to_string(&mut toml).map_err(|_| ())?;
                        toml
                    };
                    let manifest = toml.parse().map_err(|_| ())?;
                    write!(file, "{}", toml).map_err(|_| ())?;
                    Ok(manifest)
                }
                ManifestsInput::Text(text) => {
                    let manifest = text.parse().map_err(|_| ())?;
                    write!(file, "{}", text).map_err(|_| ())?;
                    Ok(manifest)
                }
            }
        };

        future.boxed()
    }
}
