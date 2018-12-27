use std::path::PathBuf;

use futures::{future, stream, Async, Future, IntoFuture, Poll, Stream};

use id::FilesystemId;

pub trait Fetchable {
    type Progress: Send;

    fn fetch(&self) -> Box<dyn Stream<Item = Self::Progress, Error = ()> + Send>;
}

#[derive(Debug)]
pub struct Fetcher<F>(F);

impl<F> Fetcher<F>
where
    F: Fetchable,
    F::Progress: 'static,
{
    pub fn new(fetchable: F) -> Self {
        Fetcher(fetchable)
    }

    pub fn fetch<I>(&self, output: PathBuf) -> FetchStream<F::Progress, I>
    where
        I: FilesystemId + Send + 'static,
    {
        let stream = self.0.fetch();
        FetchStream::new(stream, output, |path| I::from_path(&path))
    }
}

#[derive(Debug)]
pub enum State<P, I> {
    Fetching(P),
    Finished(I),
}

#[must_use = "streams do nothing unless polled"]
pub struct FetchStream<P, I> {
    inner: Option<Box<dyn Stream<Item = State<P, I>, Error = ()> + Send>>,
}

impl<P, I> FetchStream<P, I>
where
    P: Send + 'static,
    I: Send + 'static,
{
    pub fn new<S, F, U>(stream: S, output: PathBuf, compute_id: F) -> Self
    where
        S: Stream<Item = P, Error = ()> + Send + 'static,
        F: Fn(PathBuf) -> U + Send + 'static,
        U: IntoFuture<Item = I, Error = ()> + Send + 'static,
        U::Future: Send + 'static,
    {
        let computed = future::lazy(move || compute_id(output).into_future().map(State::Finished));
        let fetching = stream
            .map(State::Fetching)
            .chain(stream::futures_unordered(vec![computed]))
            .fuse();

        FetchStream {
            inner: Some(Box::new(fetching)),
        }
    }
}

impl<P, I> Stream for FetchStream<P, I> {
    type Item = State<P, I>;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self.inner.take() {
            None => Ok(Async::Ready(None)),
            Some(mut inner) => {
                let result = try_ready!(inner.poll());
                match result {
                    finished @ Some(State::Finished(_)) => {
                        self.inner = Some(inner);
                        Ok(Async::Ready(finished))
                    }
                    ongoing => Ok(Async::Ready(ongoing)),
                }
            }
        }
    }
}
