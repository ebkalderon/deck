pub use self::manifests::{ManifestInput, ManifestsDir};
pub use self::outputs::OutputsDir;
pub use self::sources::SourcesDir;

use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::path::Path;

use futures::{Future, Poll, Stream};

use super::super::progress::Progress;
// use super::fetcher::Fetcher;
use crate::id::FilesystemId;

mod manifests;
mod outputs;
mod sources;

pub type IdFuture<I> = Box<dyn Future<Item = I, Error = ()> + Send>;

pub type ReadFuture<O> = DirectoryFuture<Option<O>>;

pub type WriteFuture<I, O> = DirectoryFuture<(I, O)>;

pub type FetchStream<I> = Box<dyn Stream<Item = FetchState<I>, Error = ()> + Send>;

pub trait Directory: Debug + Send + Sync {
    type Id: FilesystemId + 'static;
    type Input: Send;
    type Output: Send + 'static;

    const NAME: &'static str;

    fn precompute_id(&self, input: &Self::Input) -> IdFuture<Self::Id>;
    fn compute_id(&self, target: &Path) -> IdFuture<Self::Id>;
    fn read(&self, target: &Path, id: &Self::Id) -> ReadFuture<Self::Output>;
    fn write(&self, target: &Path, input: Self::Input) -> WriteFuture<Self::Id, Self::Output>;
}

#[must_use = "futures do nothing unless polled"]
pub struct DirectoryFuture<T>(Box<dyn Future<Item = T, Error = ()> + Send>);

impl<T> DirectoryFuture<T> {
    pub(super) fn new<F>(inner: F) -> Self
    where
        F: Future<Item = T, Error = ()> + Send + 'static,
    {
        DirectoryFuture(Box::new(inner))
    }
}

impl<T> Debug for DirectoryFuture<T> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.debug_tuple(stringify!(DirectoryFuture<T>))
            .field(&"Box<dyn Future<Item = T, Error = ()> + Send>")
            .finish()
    }
}

impl<T> Future for DirectoryFuture<T> {
    type Item = T;
    type Error = ();

    #[inline]
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.0.poll()
    }
}

#[derive(Debug)]
pub enum FetchState<I> {
    Fetching(Progress),
    Finished(I),
}
