#![forbid(unsafe_code)]

extern crate chrono;
extern crate filetime;
extern crate fs2;
#[macro_use]
extern crate futures;
extern crate futures_locks;
extern crate hyper;
extern crate hyper_tls;
extern crate ignore;
#[macro_use]
extern crate serde;
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

pub mod binary_cache;
pub mod package;
pub mod platform;
pub mod store;
