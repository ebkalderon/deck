#![deny(missing_debug_implementations)]
#![feature(async_await, await_macro, futures_api)]
#![forbid(unsafe_code)]

pub extern crate deck_core as core;

#[cfg(feature = "s3")]
pub use self::s3::S3Cache;

use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;

use deck_core::OutputId;

mod https;
#[cfg(feature = "local")]
mod local;
#[cfg(feature = "s3")]
mod s3;

// NOTE: All this noise has been to work fine with a simple `async fn`, with no need for associated
// types, this type alias, or `Pin<Box<_>>`. Replace _immediately_ once `async fn` in traits is
// stabilized in Rust.

pub type BinaryCacheFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, ()>> + Send + 'a>>;

pub trait BinaryCache: Debug {
    fn query<'a>(&'a mut self, id: &'a OutputId) -> BinaryCacheFuture<'a, ()>;
}
