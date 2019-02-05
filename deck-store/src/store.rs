pub use self::builder::BuildStream;
pub use self::closure::Closure;

use std::fmt::Debug;

use futures::Future;

use crate::binary_cache::BinaryCache;
use crate::package::Manifest;
use crate::platform::Platform;

pub mod builder;
pub mod progress;
pub mod remote;

mod closure;
mod context;
mod fs;

pub type PlatformFuture = Box<dyn Future<Item = Vec<Platform>, Error = ()>>;

pub type AddedFuture = Box<dyn Future<Item = (), Error = ()>>;

pub type VerifyFuture = Box<dyn Future<Item = (), Error = ()>>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckContents {
    Enabled,
    Disabled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Repair {
    Enabled,
    Disabled,
}

pub trait Store: Debug {
    fn supported_platforms(&self) -> PlatformFuture;

    fn build_package(&mut self, manifest: &Manifest) -> BuildStream;

    fn add_binary_cache<B: BinaryCache>(&mut self, cache: B) -> AddedFuture;

    fn add_remote_store<S: Store>(&mut self, store: S) -> AddedFuture;

    fn verify(&mut self, check: CheckContents, repair: Repair) -> VerifyFuture;
}
