use std::path::PathBuf;

use deck_core::{Hash, Source, SourceId};
use futures_preview::future::{self, FutureExt};

use crate::local::dir::{DirFuture, Directory, ReadPath, WritePath};

#[derive(Clone, Debug)]
pub enum SourceInput {
    Path(Source, PathBuf),
    Text(String, String),
}

#[derive(Debug)]
pub struct SourcesDir;

impl Directory for SourcesDir {
    type Id = SourceId;
    type Input = SourceInput;
    type Output = PathBuf;

    const NAME: &'static str = "sources";

    fn precompute_id<'a>(&'a self, input: &'a Self::Input) -> DirFuture<'a, Self::Id> {
        let future = async move {
            match input {
                SourceInput::Path(_, _) => unimplemented!(),
                SourceInput::Text(ref name, ref text) => {
                    let hash = Hash::compute().input(&text).finish();
                    let id = SourceId::new(name.clone(), hash)?;
                    Ok(id)
                }
            }
        };

        future.boxed()
    }

    fn compute_id<'a>(&'a self, _path: &'a ReadPath) -> DirFuture<'a, Self::Id> {
        unimplemented!()
    }

    fn read<'a>(&'a self, path: &'a ReadPath) -> DirFuture<'a, Option<Self::Output>> {
        if path.exists() {
            future::ok(Some(path.as_path().to_owned())).boxed()
        } else {
            future::ok(None).boxed()
        }
    }

    fn write<'a>(
        &'a self,
        _path: &'a mut WritePath,
        _input: Self::Input,
    ) -> DirFuture<'a, Self::Output> {
        unimplemented!()
    }
}
