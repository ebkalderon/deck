use std::fmt::{Debug, Formatter, Result as FmtResult};

use futures::{stream, Future, Poll, Stream};
use hyper::header::CONTENT_LENGTH;
use hyper::Uri;

use super::super::context::Context;
use super::super::progress::{Downloading, Progress};
use super::IntoJob;
use crate::id::ManifestId;
use crate::package::Source;

#[must_use = "streams do nothing unless polled"]
pub struct FetchSource(Box<dyn Stream<Item = Progress, Error = ()> + Send>);

impl FetchSource {
    pub fn new(ctx: Context, id: ManifestId, source: Source) -> Self {
        match source {
            Source::Git => fetch_git(ctx, id),
            Source::Path { ref path, ref hash } => unimplemented!(),
            Source::Uri { uri, hash } => fetch_uri(ctx, id, uri, hash),
        }
    }

    fn from_stream<S: Stream<Item = Progress, Error = ()> + Send + 'static>(inner: S) -> Self {
        FetchSource(Box::new(inner))
    }
}

impl Debug for FetchSource {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.debug_tuple(stringify!(FetchSource))
            .field(&"Box<dyn Stream<Item = Progress, Error = ()> + Send>")
            .finish()
    }
}

impl IntoJob for FetchSource {}

impl Stream for FetchSource {
    type Item = Progress;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.0.poll()
    }
}

fn fetch_uri(ctx: Context, id: ManifestId, uri: String, _hash: String) -> FetchSource {
    let client = ctx.client.clone();
    let _store = ctx.store.clone();

    let downloading = client
        .get(uri.parse().unwrap())
        .map_err(|e| eprintln!("failed to connect to URI: {}", e))
        .map(move |resp| {
            let len = resp
                .headers()
                .get(CONTENT_LENGTH)
                .and_then(|len| len.to_str().ok())
                .and_then(|len| len.parse::<u64>().ok());

            let mut progress = Downloading {
                package_id: id.clone(),
                downloaded_bytes: 0,
                total_bytes: len,
                source: uri,
            };

            let stream = resp.into_body().map_err(|_| ()).map(move |chunk| {
                progress.downloaded_bytes += chunk.len() as u64;
                Progress::Downloading(progress.clone())
            });

            (id, stream)
        });

    let completing = downloading.map(move |(id, stream)| {
        let progress = Progress::Blocked { package_id: id };
        stream.chain(stream::iter_ok(vec![progress]))
    });

    FetchSource::from_stream(completing.flatten_stream())
}

fn fetch_git(_ctx: Context, id: ManifestId) -> FetchSource {
    FetchSource::from_stream(stream::iter_ok(vec![Progress::Blocked { package_id: id }]))
}
