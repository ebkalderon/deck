use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;

use super::Directory;

use crate::id::OutputId;

#[derive(Debug)]
pub struct OutputsDir;

impl Directory for OutputsDir {
    type Id = OutputId;
    type Input = PathBuf;
    type Output = PathBuf;

    type IdFuture = Pin<Box<dyn Future<Output = Result<Self::Id, ()>> + Send>>;
    type ReadFuture = Pin<Box<dyn Future<Output = Result<Option<Self::Output>, ()>> + Send>>;
    type WriteFuture = Pin<Box<dyn Future<Output = Result<Self::Output, ()>> + Send>>;

    const NAME: &'static str = "outputs";

    fn precompute_id(&self, _input: &Self::Input) -> Self::IdFuture {
        unimplemented!()
    }

    fn compute_id(&self, _target: &Path) -> Self::IdFuture {
        unimplemented!()
    }

    fn read(&self, _target: &Path, _id: &Self::Id) -> Self::ReadFuture {
        unimplemented!()
    }

    fn write(&self, _target: &Path, _input: Self::Input) -> Self::WriteFuture {
        unimplemented!()
    }
}
