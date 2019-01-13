//! Reproducible package data.

pub use self::manifest::{Manifest, ManifestBuilder};
pub use self::sources::Source;

mod manifest;
mod outputs;
mod sources;
