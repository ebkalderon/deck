#![deny(missing_debug_implementations)]
#![feature(async_await, await_macro, futures_api)]
#![forbid(unsafe_code)]

pub extern crate deck_core as core;

#[cfg(feature = "local")]
pub use self::local::LocalCache;
#[cfg(feature = "s3")]
pub use self::s3::S3Cache;

use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;

use deck_core::OutputId;
use futures::stream::Stream;

mod https;
#[cfg(feature = "local")]
mod local;
#[cfg(feature = "s3")]
mod s3;

// NOTE: All this noise has been to work fine with a simple `async fn`, with no need for associated
// types, this type alias, or `Pin<Box<_>>`. Replace _immediately_ once `async fn` in traits is
// stabilized in Rust.

pub type BinaryCacheFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, ()>> + Send + 'a>>;
pub type OutputStream<'a> = Pin<Box<dyn Stream<Item = Result<Vec<u8>, ()>> + Send + 'a>>;

pub trait BinaryCache: Debug {
    fn query_outputs<'a>(&'a mut self, id: &'a OutputId) -> BinaryCacheFuture<'a, ()>;
    fn fetch_output<'a>(&'a mut self, id: &'a OutputId) -> OutputStream<'a>;
}
