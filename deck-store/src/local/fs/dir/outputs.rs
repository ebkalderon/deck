use std::path::PathBuf;

use deck_core::OutputId;

use super::{DirFuture, Directory, ReadPath, WritePath};

#[derive(Debug)]
pub struct OutputsDir;

impl Directory for OutputsDir {
    type Id = OutputId;
    type Input = PathBuf;
    type Output = PathBuf;

    const NAME: &'static str = "outputs";

    fn precompute_id<'a>(&'a self, _input: &'a Self::Input) -> DirFuture<'a, Self::Id> {
        unimplemented!()
    }

    fn compute_id<'a>(&'a self, _path: &'a ReadPath) -> DirFuture<'a, Self::Id> {
        unimplemented!()
    }

    fn read<'a>(&'a self, _path: &'a ReadPath) -> DirFuture<'a, Option<Self::Output>> {
        unimplemented!()
    }

    fn write<'a>(
        &'a self,
        _path: &'a mut WritePath,
        _input: Self::Input,
    ) -> DirFuture<'a, Self::Output> {
        unimplemented!()
    }
}
