#![feature(async_await, await_macro, futures_api)]
#![feature(const_str_as_bytes)]
#![forbid(unsafe_code)]

#[macro_use]
extern crate futures;
#[macro_use]
extern crate serde;

#[cfg(feature = "local")]
#[macro_use]
extern crate diesel;
#[cfg(feature = "local")]
#[macro_use]
extern crate diesel_migrations;

#[cfg(feature = "s3")]
extern crate rusoto_s3;

pub use crate::hash::Hash;
pub use crate::id::{ManifestId, OutputId, SourceId};

pub mod binary_cache;
pub mod package;
pub mod platform;
pub mod store;

mod hash;
mod id;
