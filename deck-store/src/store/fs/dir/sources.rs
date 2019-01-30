use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;

use futures_preview::StreamExt;
use futures_preview::future::{self, FutureExt, Ready};

use super::Directory;
use crate::hash::Hash;
use crate::id::SourceId;
use crate::package::Source;

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

    type IdFuture = Pin<Box<dyn Future<Output = Result<Self::Id, ()>> + Send>>;
    type ReadFuture = Ready<Result<Option<Self::Output>, ()>>;
    type WriteFuture = Pin<Box<dyn Future<Output = Result<Self::Output, ()>> + Send>>;

    const NAME: &'static str = "sources";

    fn precompute_id(&self, input: &Self::Input) -> Self::IdFuture {
        let input = input.clone();
        let future = async move {
            match input {
                SourceInput::Path(ref src, _) => unimplemented!(),
                SourceInput::Text(ref name, ref text) => {
                    let hash = Hash::compute().input(&text).finish();
                    let id = SourceId::new(name.clone(), hash)?;
                    Ok(id)
                }
            }
        };

        future.boxed()
    }

    fn compute_id(&self, _target: &Path) -> Self::IdFuture {
        unimplemented!()
    }

    fn read(&self, target: &Path, _: &Self::Id) -> Self::ReadFuture {
        if target.exists() {
            future::ok(Some(target.to_owned()))
        } else {
            future::ok(None)
        }
    }

    fn write(&self, target: &Path, input: Self::Input) -> Self::WriteFuture {
        unimplemented!()
    }
}
