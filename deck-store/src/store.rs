pub use self::builder::BuildStream;
pub use self::closure::Closure;

use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;

use crate::binary_cache::BinaryCache;
use crate::id::ManifestId;
use crate::package::Manifest;
use crate::platform::Platform;
use crate::repo::Repository;

pub mod builder;
pub mod fs;
pub mod progress;
pub mod remote;

mod closure;
mod context;

// NOTE: All this noise has been to work fine with a simple `async fn`, with no need for associated
// types, this type alias, or `Pin<Box<_>>`. Replace _immediately_ once `async fn` in traits is
// stabilized in Rust.

pub type StoreFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, ()>> + Send + 'a>>;

/// Sets whether the hashes of the store contents should be recomputed and verified.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CheckContents {
    /// Each item in the store should have its hash recomputed and verified.
    Enabled,
    /// Only check whether the paths are registered, do not validate the hashes.
    Disabled,
}

/// Sets whether store inconsistencies should be repaired.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Repair {
    /// Missing paths should be registered and inconsistent hashes should be recomputed.
    Enabled,
    /// Nothing should be repaired, report errors without modifying the store.
    Disabled,
}

pub trait Store: Debug {
    fn supported_platforms<'a>(&'a self) -> StoreFuture<'a, Vec<Platform>>;
    fn add_binary_cache<'a, B: BinaryCache>(&'a mut self, cache: B) -> StoreFuture<'a, ()>;
    fn add_remote_store<'a, S: Store>(&'a mut self, store: S) -> StoreFuture<'a, ()>;
    fn add_repository<'a, R: Repository>(&'a mut self, repo: R) -> StoreFuture<'a, ()>;
    fn build_manifest(&mut self, manifest: Manifest) -> BuildStream;
    fn get_build_log<'a>(&'a mut self, id: &'a ManifestId) -> StoreFuture<'a, Option<String>>;
    fn verify<'a>(&'a mut self, check: CheckContents, repair: Repair) -> StoreFuture<'a, ()>;
}
