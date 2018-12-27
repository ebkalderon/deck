pub use self::manifest::ManifestId;
pub use self::output::OutputId;
pub use self::source::SourceId;

use std::path::{Path, PathBuf};

mod manifest;
mod name;
mod output;
mod source;

pub trait FilesystemId: Sized {
    fn from_path(path: &Path) -> Result<Self, ()>;
    fn to_path(&self) -> PathBuf;
}
