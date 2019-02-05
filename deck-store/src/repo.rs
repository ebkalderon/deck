use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;

use crate::id::ManifestId;
use crate::package::Manifest;

pub type RepositoryFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, ()>> + Send + 'a>>;

pub trait Repository: Debug {
    fn query<'a>(&'a mut self, id: &'a ManifestId) -> RepositoryFuture<'a, Manifest>;
}
