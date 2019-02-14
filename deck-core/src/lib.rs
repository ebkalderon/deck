#![deny(missing_debug_implementations)]
#![forbid(unsafe_code)]

pub use self::hash::{Hash, HashBuilder};
pub use self::id::{FilesystemId, ManifestId, OutputId, SourceId};
pub use self::manifest::{Manifest, ManifestBuilder, Source};
pub use self::name::Name;
pub use self::platform::Platform;

mod hash;
mod id;
mod manifest;
mod name;
mod platform;
