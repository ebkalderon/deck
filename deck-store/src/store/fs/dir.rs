pub use self::manifests::{ManifestsDir, ManifestsInput};
pub use self::outputs::OutputsDir;
pub use self::sources::SourcesDir;

use std::fmt::Debug;
use std::path::Path;
use std::future::Future;

use crate::id::FilesystemId;

mod manifests;
mod outputs;
mod sources;

pub trait Directory: Debug + Send + Sync {
    type Id: FilesystemId + 'static;
    type Input: Send;
    type Output: Send + 'static;

    type IdFuture: Future<Output = Result<Self::Id, ()>> + Send;
    type ReadFuture: Future<Output = Result<Option<Self::Output>, ()>> + Send;
    type WriteFuture: Future<Output = Result<Self::Output, ()>> + Send;

    const NAME: &'static str;

    fn precompute_id(&self, input: &Self::Input) -> Self::IdFuture;
    fn compute_id(&self, target: &Path) -> Self::IdFuture;
    fn read(&self, target: &Path, id: &Self::Id) -> Self::ReadFuture;
    fn write(&self, target: &Path, input: Self::Input) -> Self::WriteFuture;
}
