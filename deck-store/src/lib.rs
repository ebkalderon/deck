#![forbid(unsafe_code)]

extern crate blake2;
extern crate chrono;
extern crate data_encoding;
extern crate filetime;
extern crate fs2;
#[macro_use]
extern crate futures;
extern crate futures_locks;
extern crate hyper;
extern crate hyper_tls;
extern crate ignore;
extern crate rand;
#[macro_use]
extern crate serde;
extern crate sha2;
extern crate sha3;
extern crate tokio;
extern crate toml;
extern crate url;

#[cfg(feature = "local")]
#[macro_use]
extern crate diesel;
#[cfg(feature = "local")]
#[macro_use]
extern crate diesel_migrations;

#[cfg(feature = "s3")]
extern crate rusoto_s3;

pub use hash::Hash;
pub use id::{ManifestId, OutputId, SourceId};

pub mod binary_cache;
pub mod package;
pub mod platform;
pub mod store;

mod hash;
mod id;
