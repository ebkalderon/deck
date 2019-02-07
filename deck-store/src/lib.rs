#![deny(missing_debug_implementations)]
#![feature(async_await, await_macro, futures_api)]
#![feature(const_str_as_bytes)]
#![forbid(unsafe_code)]

pub extern crate deck_core as core;

pub use self::closure::Closure;
pub use self::id::StoreId;

use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::future::Future;
use std::pin::Pin;
use std::task::{LocalWaker, Poll};

use deck_binary_cache::BinaryCache;
use deck_core::{Manifest, ManifestId, Platform};
use deck_repository::Repository;
use futures_preview::stream::{Stream, StreamExt};

use self::progress::Progress;

#[cfg(feature = "local")]
pub mod local;
pub mod progress;
pub mod remote;

mod closure;
mod id;

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

/// Represents a content-addressable store of packages.
pub trait Store: BinaryCache + Debug {
    fn supported_platforms<'a>(&'a self) -> StoreFuture<'a, Vec<Platform>>;
    fn build_manifest(&mut self, manifest: Manifest) -> BuildStream;
    fn get_build_log<'a>(&'a mut self, id: &'a ManifestId) -> StoreFuture<'a, Option<String>>;
    fn verify<'a>(&'a mut self, check: CheckContents, repair: Repair) -> StoreFuture<'a, ()>;
}

/// Stream which reports the current progress of a builder.
///
/// Created from the `Store::build_manifest()` method.
#[must_use = "streams do nothing unless polled"]
pub struct BuildStream(Pin<Box<dyn Stream<Item = Result<Progress, ()>> + Send>>);

impl BuildStream {
    /// Creates a new `BuildStream` from the given progress stream.
    pub fn new<S>(stream: S) -> Self
    where
        S: Stream<Item = Result<Progress, ()>> + Send + 'static,
    {
        BuildStream(stream.boxed())
    }
}

impl Debug for BuildStream {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.debug_tuple(stringify!(BuildStream))
            .field(&"Pin<Box<dyn Stream<Item = Result<Progress, Error>> + Send>>")
            .finish()
    }
}

impl Stream for BuildStream {
    type Item = Result<Progress, ()>;

    fn poll_next(mut self: Pin<&mut Self>, lw: &LocalWaker) -> Poll<Option<Self::Item>> {
        self.0.as_mut().poll_next(lw)
    }
}
