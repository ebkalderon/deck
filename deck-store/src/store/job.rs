pub use self::build_manifest::BuildManifest;
pub use self::fetch_output::FetchOutput;
pub use self::fetch_source::FetchSource;

use std::fmt::{Debug, Formatter, Result as FmtResult};

use futures::{Future, Poll, Sink, Stream};

use super::progress::{Progress, ProgressSender};

mod build_manifest;
mod fetch_output;
mod fetch_source;

#[must_use = "futures do nothing unless polled"]
pub struct JobFuture(Box<dyn Future<Item = (), Error = ()> + Send>);

impl JobFuture {
    fn from_stream<S>(inner: S, progress: ProgressSender) -> Self
    where
        S: Stream<Item = Progress, Error = ()> + Send + 'static,
    {
        let future = inner
            .then(|result| Ok(result))
            .forward(progress.sink_map_err(|_| ()))
            .map(|_| ());

        JobFuture(Box::new(future))
    }
}

impl Debug for JobFuture {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.debug_tuple(stringify!(JobFuture))
            .field(&"Box<dyn Future<Item = (), Error = () + Send>")
            .finish()
    }
}

impl Future for JobFuture {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.0.poll()
    }
}

pub trait IntoJob: Stream<Item = Progress, Error = ()> + Sized + Send + 'static {
    fn into_job(self, progress: ProgressSender) -> JobFuture {
        JobFuture::from_stream(self, progress)
    }
}
