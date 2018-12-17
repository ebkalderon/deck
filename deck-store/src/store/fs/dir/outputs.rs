use std::path::{Path, PathBuf};

use super::{Directory, IdFuture, ReadFuture, WriteFuture};

#[derive(Debug)]
pub struct OutputsDir;

impl Directory for OutputsDir {
    type Id = String;
    type Input = String;
    type Output = PathBuf;

    const NAME: &'static str = "outputs";

    fn compute_id(&self, input: &Self::Input) -> IdFuture<Self::Id> {
        unimplemented!()
    }

    fn read(&self, target: &Path, id: &Self::Id) -> ReadFuture<Self::Output> {
        unimplemented!()
    }

    fn write(&self, target: &Path, input: Self::Input) -> WriteFuture<Self::Id, Self::Output> {
        unimplemented!()
    }
}
