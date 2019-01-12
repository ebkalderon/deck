use std::path::{Path, PathBuf};

use super::{Directory, IdFuture, ReadFuture, WriteFuture};

use crate::id::OutputId;

#[derive(Debug)]
pub struct OutputsDir;

impl Directory for OutputsDir {
    type Id = OutputId;
    type Input = String;
    type Output = PathBuf;

    const NAME: &'static str = "outputs";

    fn precompute_id(&self, _input: &Self::Input) -> IdFuture<Self::Id> {
        unimplemented!()
    }

    fn compute_id(&self, _target: &Path) -> IdFuture<Self::Id> {
        unimplemented!()
    }

    fn read(&self, _target: &Path, _id: &Self::Id) -> ReadFuture<Self::Output> {
        unimplemented!()
    }

    fn write(&self, _target: &Path, _input: Self::Input) -> WriteFuture<Self::Id, Self::Output> {
        unimplemented!()
    }
}
