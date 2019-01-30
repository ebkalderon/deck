use std::pin::Pin;
use std::task::{LocalWaker, Poll};

use futures_preview::stream::Stream;

use crate::id::ManifestId;
use crate::store::context::Context;
use crate::store::progress::Progress;

#[must_use = "streams do nothing unless polled"]
pub struct FetchOutput(Pin<Box<dyn Stream<Item = Result<Progress, ()>> + Send>>);

impl FetchOutput {
    pub fn new(ctx: Context, id: ManifestId) -> Self {
        unimplemented!()
    }
}

impl Stream for FetchOutput {
    type Item = Result<Progress, ()>;

    fn poll_next(mut self: Pin<&mut Self>, lw: &LocalWaker) -> Poll<Option<Self::Item>> {
        self.0.as_mut().poll_next(lw)
    }
}
