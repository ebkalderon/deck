use std::fmt::Debug;

use futures::Future;

use binary_cache::BinaryCache;
use error::BuildError;
use manifest::Manifest;
use platform::Platform;

#[cfg(feature = "local")]
pub mod local;
#[cfg(feature = "ssh")]
pub mod ssh;

mod id;

#[derive(Debug)]
pub struct Package;

pub type PlatformFuture = Box<dyn Future<Item = Vec<Platform>, Error = ()>>;

pub type PackageFuture = Box<dyn Future<Item = Package, Error = BuildError>>;

pub type AddedFuture = Box<dyn Future<Item = (), Error = ()>>;

pub type VerifyFuture = Box<dyn Future<Item = (), Error = ()>>;

pub trait Store: Debug {
    fn supported_platforms(&self) -> PlatformFuture;

    fn build_package(&mut self, manifest: &Manifest) -> PackageFuture;

    fn add_binary_cache<B: BinaryCache>(&mut self, cache: B) -> AddedFuture;

    fn add_remote_store<S: Store>(&mut self, store: S) -> AddedFuture;

    fn verify(&mut self, repair: bool) -> VerifyFuture;
}
