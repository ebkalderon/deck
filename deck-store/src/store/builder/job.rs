pub use self::build_manifest::BuildManifest;
pub use self::fetch_output::FetchOutput;
pub use self::fetch_source::FetchSource;

use std::future::Future;
use std::pin::Pin;

use futures_preview::future::{self, FutureExt, TryFutureExt};
use futures_preview::stream::{self, Stream};

use super::futures::JobFuture;
use crate::id::ManifestId;
use crate::store::context::Context;
use crate::store::progress::{Progress, ProgressSender};

mod build_manifest;
mod fetch_output;
mod fetch_source;

pub trait IntoJob {
    fn into_job(self, tx: ProgressSender) -> JobFuture;
}

impl<S, F> IntoJob for F
where
    S: Stream<Item = Result<Progress, ()>> + Send + 'static,
    F: Future<Output = Result<S, ()>> + Send + Unpin + 'static,
{
    fn into_job(self, tx: ProgressSender) -> JobFuture {
        let stream = self
            .map_ok(|stream| Box::pin(stream) as Pin<Box<dyn Stream<Item = _> + Send>>)
            .unwrap_or_else(|err| {
                Box::pin(stream::once(future::err(err))) as Pin<Box<dyn Stream<Item = _> + Send>>
            })
            .flatten_stream();

        JobFuture::new(stream, tx)
    }
}

trait Job {
    type Args;
    type Stream: Stream<Item = Result<Progress, ()>> + Send;
    type Future: Future<Output = Result<Self::Stream, ()>> + Send;

    fn run(context: Context, id: ManifestId, args: Self::Args) -> Self::Future;
}
