//! Deck daemon implementation.

#![forbid(unsafe_code)]
#![deny(missing_debug_implementations)]

extern crate fnv;
extern crate futures;
#[macro_use]
extern crate serde;
extern crate tokio;
extern crate toml;

use config::Config;

mod config;

#[derive(Debug)]
pub struct Daemon {
    cfg: Config,
}

impl Daemon {
    pub fn new(cfg: Config) -> Result<Self, ()> {
        Ok(Daemon { cfg })
    }
}
