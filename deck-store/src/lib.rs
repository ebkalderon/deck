#![deny(missing_debug_implementations)]
#![feature(async_await, await_macro, futures_api)]
#![feature(const_str_as_bytes)]
#![forbid(unsafe_code)]

#[cfg(feature = "local")]
extern crate diesel;
#[cfg(feature = "local")]
extern crate diesel_migrations;

#[cfg(feature = "s3")]
extern crate rusoto_s3;

pub use crate::hash::Hash;
pub use crate::id::{ManifestId, OutputId, SourceId};

pub mod binary_cache;
pub mod package;
pub mod platform;
pub mod repo;
pub mod store;

mod hash;
mod id;
