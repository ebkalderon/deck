#![forbid(unsafe_code)]

#[macro_use]
extern crate prost_derive;

/// Protocol for communicating with a daemon managing a central store in multi-user mode.
pub mod daemon {
    include!(concat!(env!("OUT_DIR"), "/deck.daemon.v1alpha1.rs"));
}
