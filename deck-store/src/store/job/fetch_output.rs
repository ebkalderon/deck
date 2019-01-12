use std::fmt::{Debug, Formatter, Result as FmtResult};

use futures::{Poll, Stream};

use super::super::context::Context;
use super::super::progress::Progress;
use super::IntoJob;
use crate::id::ManifestId;

#[must_use = "streams do nothing unless polled"]
pub struct FetchOutput(Box<dyn Stream<Item = Progress, Error = ()> + Send>);

impl FetchOutput {
    pub fn new(ctx: Context, id: ManifestId) -> Self {
        unimplemented!()
    }
}

impl Debug for FetchOutput {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.debug_tuple(stringify!(FetchSource))
            .field(&"Box<dyn Stream<Item = Progress, Error = ()> + Send>")
            .finish()
    }
}

impl IntoJob for FetchOutput {}

impl Stream for FetchOutput {
    type Item = Progress;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.0.poll()
    }
}
