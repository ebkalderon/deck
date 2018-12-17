pub use self::manifests::{ManifestInput, ManifestsDir};
pub use self::outputs::OutputsDir;
pub use self::sources::SourcesDir;

use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::hash::Hash;
use std::path::Path;

use futures::{Future, Poll};

mod manifests;
mod outputs;
mod sources;

pub type IdFuture<I> = Box<dyn Future<Item = I, Error = ()> + Send>;

pub type ReadFuture<O> = DirectoryFuture<Option<O>>;

pub type WriteFuture<I, O> = DirectoryFuture<(I, O)>;

pub trait Directory: Debug + Send + Sync {
    type Id: Clone + Eq + Hash + Send + Sync + ToString;
    type Input: Send;
    type Output: Send;

    const NAME: &'static str;

    fn compute_id(&self, input: &Self::Input) -> IdFuture<Self::Id>;
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

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.0.poll()
    }
}
