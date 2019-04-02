use std::pin::Pin;
use std::task::{Poll, Waker};

use deck_core::ManifestId;
use futures_preview::stream::Stream;

use crate::local::context::Context;
use crate::progress::Progress;

#[must_use = "streams do nothing unless polled"]
pub struct FetchOutput(Pin<Box<dyn Stream<Item = Result<Progress, ()>> + Send>>);

impl FetchOutput {
    pub fn new(ctx: Context, id: ManifestId) -> Self {
        unimplemented!()
    }
}

impl Stream for FetchOutput {
    type Item = Result<Progress, ()>;

    fn poll_next(mut self: Pin<&mut Self>, waker: &Waker) -> Poll<Option<Self::Item>> {
        self.0.as_mut().poll_next(waker)
    }
}
