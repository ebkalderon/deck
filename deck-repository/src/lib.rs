#![deny(missing_debug_implementations)]
#![feature(async_await, await_macro, futures_api)]
#![forbid(unsafe_code)]

pub extern crate deck_core as core;

use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;

use deck_core::{Manifest, ManifestId};

// NOTE: All this noise has been to work fine with a simple `async fn`, with no need for associated
// types, this type alias, or `Pin<Box<_>>`. Replace _immediately_ once `async fn` in traits is
// stabilized in Rust.

pub type RepositoryFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, ()>> + Send + 'a>>;

pub trait Repository: Debug {
    fn query<'a>(&'a mut self, id: &'a ManifestId) -> RepositoryFuture<'a, Manifest>;
}
