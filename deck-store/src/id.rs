//! Content-addressable identifiers for store objects.

pub use self::manifest::ManifestId;
pub use self::name::Name;
pub use self::output::OutputId;
pub use self::source::SourceId;

use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::path::{Path, PathBuf};

mod manifest;
mod name;
mod output;
mod source;

/// Trait for store IDs which have an on-disk representation.
pub trait FilesystemId: Clone + Debug + Display + Eq + Hash + Send + Sync {
    /// Attempts to parse the filesystem-agnostic ID from the given path.
    fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, ()>;
    /// Returns the `PathBuf` representation of this ID.
    fn to_path(&self) -> PathBuf;
}
