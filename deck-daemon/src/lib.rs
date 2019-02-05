//! Deck daemon implementation.

#![deny(missing_debug_implementations)]
#![forbid(unsafe_code)]

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
