//! Deck daemon implementation.

#![forbid(unsafe_code)]
#![deny(missing_debug_implementations)]

#[macro_use]
extern crate serde;

use crate::config::Config;

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
