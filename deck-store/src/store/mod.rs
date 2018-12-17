pub use self::builder::BuildStream;

use std::fmt::Debug;

use futures::Future;

use binary_cache::BinaryCache;
use package::Manifest;
use platform::Platform;

pub mod builder;
pub mod remote;

mod context;
mod fs;
mod job;

pub type PlatformFuture = Box<dyn Future<Item = Vec<Platform>, Error = ()>>;

pub type AddedFuture = Box<dyn Future<Item = (), Error = ()>>;

pub type VerifyFuture = Box<dyn Future<Item = (), Error = ()>>;

pub trait Store: Debug {
    fn supported_platforms(&self) -> PlatformFuture;

    fn build_package(&mut self, manifest: &Manifest) -> BuildStream;

    fn add_binary_cache<B: BinaryCache>(&mut self, cache: B) -> AddedFuture;

    fn add_remote_store<S: Store>(&mut self, store: S) -> AddedFuture;

    fn verify(&mut self, repair: bool) -> VerifyFuture;
}
