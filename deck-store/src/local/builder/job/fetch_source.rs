use std::pin::Pin;
use std::task::{Poll, Waker};

use deck_core::{ManifestId, Source};
use futures_preview::compat::{Future01CompatExt, Stream01CompatExt};
use futures_preview::future::{self, FutureExt, TryFutureExt};
use futures_preview::stream::{self, Stream, StreamExt, TryStreamExt};
use hyper::header::CONTENT_LENGTH;

use crate::local::context::Context;
use crate::progress::{Blocked, Downloading, Progress};

#[must_use = "streams do nothing unless polled"]
pub struct FetchSource(Pin<Box<dyn Stream<Item = Result<Progress, ()>> + Send>>);

impl FetchSource {
    pub fn new(ctx: Context, id: ManifestId, source: Source) -> Self {
        match source {
            Source::Git => fetch_git(ctx, id),
            Source::Path { ref path, ref hash } => unimplemented!(),
            Source::Uri { uri, hash } => fetch_uri(ctx, id, uri, hash),
        }
    }

    fn from_stream<S: Stream<Item = Result<Progress, ()>> + Send + 'static>(inner: S) -> Self {
        FetchSource(Box::pin(inner))
    }
}

impl Stream for FetchSource {
    type Item = Result<Progress, ()>;

    fn poll_next(mut self: Pin<&mut Self>, waker: &Waker) -> Poll<Option<Self::Item>> {
        self.0.as_mut().poll_next(waker)
    }
}

fn fetch_uri(ctx: Context, id: ManifestId, uri: String, _hash: String) -> FetchSource {
    let future = async move {
        let get = ctx.client.get(uri.parse().unwrap()).compat();
        let response = await!(get).map_err(|e| eprintln!("failed to connect to URI: {}", e))?;

        let len = response
            .headers()
            .get(CONTENT_LENGTH)
            .and_then(|len| len.to_str().ok())
            .and_then(|len| len.parse::<u64>().ok());

        let mut progress = Downloading {
            package_id: id.clone(),
            downloaded_bytes: 0,
            total_bytes: len,
            source: uri.clone(),
        };

        let downloading = response
            .into_body()
            .compat()
            .map_err(|_| ())
            .map_ok(move |chunk| {
                progress.downloaded_bytes += chunk.len() as u64;
                Progress::Downloading(progress.clone())
            });

        let progress = Progress::Blocked(Blocked {
            package_id: id,
            description: format!("fetched source from `{}`", uri),
        });

        let done = downloading.chain(stream::once(future::ok(progress)));
        Ok(done)
    };

    let stream = future
        .map_ok(|stream| Box::pin(stream) as Pin<Box<dyn Stream<Item = _> + Send>>)
        .unwrap_or_else(|err| Box::pin(stream::once(future::err(err))))
        .flatten_stream();

    FetchSource::from_stream(stream)
}

fn fetch_git(_ctx: Context, id: ManifestId) -> FetchSource {
    FetchSource::from_stream(stream::once(future::ok(Progress::Blocked(Blocked {
        package_id: id,
        description: "checked out repository".to_string(),
    }))))
}
