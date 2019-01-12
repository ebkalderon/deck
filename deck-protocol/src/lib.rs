#![forbid(unsafe_code)]

#[macro_use]
extern crate prost_derive;

/// Common data types used across the Deck protocols.
pub mod core {
    include!(concat!(env!("OUT_DIR"), "/deck.core.rs"));
}

/// Protocol for communicating with a remote binary cache.
pub mod binary_cache {
    include!(concat!(env!("OUT_DIR"), "/deck.binary_cache.rs"));
}

/// Protocol for communicating with a sandboxed builder.
pub mod builder {
    include!(concat!(env!("OUT_DIR"), "/deck.builder.rs"));
}

/// Protocol for communicating with a daemon controlling a central store.
#[cfg(feature = "daemon")]
pub mod daemon {
    include!(concat!(env!("OUT_DIR"), "/deck.daemon.rs"));
}
