use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::future::Future;
use std::pin::Pin;
use std::task::{Poll, Waker};

use deck_core::{Manifest, ManifestId};
use futures_preview::future::{self, FutureExt, TryFutureExt};
use futures_preview::sink::SinkExt;
use futures_preview::stream::{self, Stream, StreamExt};

use super::BuildGraph;
use crate::local::context::Context;
use crate::progress::{Progress, ProgressReceiver, ProgressSender};
use crate::BuildStream;

/// Executes a discrete unit of work during the build process.
///
/// Some examples of discrete units of work might include: fetching a package source, fetching a
/// package output, and building a package.
#[must_use = "futures do nothing unless polled"]
pub struct JobFuture(Pin<Box<dyn Future<Output = ()> + Send>>);

impl JobFuture {
    /// Creates a new `JobFuture` that forwards the `progress` stream to the given `ProgressSender`.
    pub fn new<S>(progress: S, tx: ProgressSender) -> Self
    where
        S: Stream<Item = Result<Progress, ()>> + Send + Unpin + 'static,
    {
        let future = progress
            .map(Ok)
            .forward(tx.sink_map_err(|_| ()))
            .map(|_| ())
            .boxed();

        JobFuture(future)
    }
}

impl Future for JobFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, waker: &Waker) -> Poll<Self::Output> {
        self.0.as_mut().poll(waker)
    }
}

/// A self-contained node in a build graph.
///
/// This future drives the execution of one or more `JobFuture`s.
///
/// `BuildFuture`s do not resolve to any particular output values on their own. They are only used
/// to execute work on the threadpool and enforce the ordering of build tasks. The current progress
/// of all `BuildFuture`s in a build graph is aggregated in a single `BuildStream`, which is
/// returned by a `Builder`.
///
/// This future is intentionally made `Clone` and is safe to poll from multiple threads.
#[derive(Clone)]
#[must_use = "futures do nothing unless polled"]
pub struct BuildFuture(future::Shared<Pin<Box<dyn Future<Output = ()> + Send>>>);

impl BuildFuture {
    /// Creates a new `BuildFuture` which executes a single one-off job.
    pub fn new(job: JobFuture) -> Self {
        let future: Box<dyn Future<Output = _> + Send> = Box::new(job);
        BuildFuture(Pin::from(future).shared())
    }

    /// Creates a new `BuildFuture` which executes the given jobs concurrently, resolving only once
    /// all of them have completed.
    pub fn join_all<I: IntoIterator<Item = JobFuture>>(jobs: I) -> Self {
        let joined = future::join_all(jobs).map(|_| ());
        let future: Box<dyn Future<Output = _> + Send> = Box::new(joined);
        BuildFuture(Pin::from(future).shared())
    }

    /// Creates a new `BuildFuture` which waits for `deps` to complete before executing `next`.
    pub fn join_all_and_then<I: IntoIterator<Item = BuildFuture>>(
        deps: I,
        next: JobFuture,
    ) -> Self {
        let joined = future::join_all(deps).then(|_| next);
        let future: Box<dyn Future<Output = _> + Send> = Box::new(joined);
        BuildFuture(Pin::from(future).shared())
    }
}

impl Debug for BuildFuture {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.debug_tuple(stringify!(BuildFuture))
            .field(&"future::Shared<Pin<Box<dyn Future<Output = ()> + Send>>>")
            .finish()
    }
}

impl Future for BuildFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, waker: &Waker) -> Poll<Self::Output> {
        Future::poll(Pin::new(&mut self.0), waker)
    }
}

/// Internal state of the builder.
pub struct BuilderState {
    /// Shared context with access to the store and fetchers.
    pub context: Context,
    /// Package manifest to build.
    pub manifest: Manifest,
    /// Precomputed ID of the package manifest to build.
    pub manifest_id: ManifestId,
    /// Cache of processed nodes in the build graph.
    pub graph: BuildGraph,
    /// Sink used to send progress info to the `BuildStream`.
    pub progress: ProgressSender,
    /// List of dependent `BuildFuture`s to join on later.
    pub dependencies: Vec<BuildFuture>,
}

/// Future which asynchronously constructs a `BuildGraph`, exiting early if any error occurs.
#[must_use = "futures do nothing unless polled"]
pub struct InnerFuture(Pin<Box<dyn Future<Output = Result<BuilderState, ()>> + Send>>);

impl InnerFuture {
    /// Creates a new `InnerFuture` which represents the intermediate state of the builder.
    pub fn new<F: Future<Output = Result<BuilderState, ()>> + Send + 'static>(f: F) -> Self {
        InnerFuture(f.boxed())
    }
}

impl Debug for InnerFuture {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.debug_tuple(stringify!(InnerFuture))
            .field(&"Pin<Box<dyn Future<Output = Result<BuilderState, Error>> + Send>>")
            .finish()
    }
}

impl Future for InnerFuture {
    type Output = Result<BuilderState, ()>;

    fn poll(mut self: Pin<&mut Self>, waker: &Waker) -> Poll<Self::Output> {
        self.0.as_mut().poll(waker)
    }
}

impl BuildStream {
    /// Creates a new `BuildStream` which drives the builder to completion.
    ///
    /// Requires a `BuildFuture` which represents the entire build graph and the receiving half of
    /// the `ProgressReceiver` used to report progress.
    pub(super) fn from_future<F>(future: F, rx: ProgressReceiver) -> Self
    where
        F: Future<Output = Result<BuildFuture, ()>> + Send + 'static,
    {
        let build_started = async move {
            match await!(future) {
                Err(err) => vec![Err(err)],
                Ok(build) => {
                    tokio::spawn(build.map(Ok).compat());
                    Vec::new()
                }
            }
        };

        let progress = build_started
            .map(|res| stream::iter(res))
            .flatten_stream()
            .select(rx)
            .boxed();

        BuildStream(progress)
    }
}
