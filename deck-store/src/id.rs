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

pub trait FilesystemId: Clone + Debug + Display + Eq + Hash + Send + Sized + Sync {
    fn from_path(path: &Path) -> Result<Self, ()>;
    fn to_path(&self) -> PathBuf;
}
