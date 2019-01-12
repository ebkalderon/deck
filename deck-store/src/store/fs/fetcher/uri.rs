use std::path::PathBuf;
use std::sync::Arc;

use futures::Stream;
use hyper::{Chunk, Uri};

use super::super::HttpsClient;
use super::Fetcher;
use crate::store::progress::Progress;

#[derive(Debug)]
pub struct FetchUri {
    client: Arc<HttpsClient>,
    uri: Uri,
    hash: String,
}

impl FetchUri {
    pub fn new(uri: Uri, hash: String, client: Arc<HttpsClient>) -> Self {
        FetchUri { client, uri, hash }
    }
}

impl Fetcher for FetchUri {
    type Args = ();
    type Progress = Box<dyn Stream<Item = Progress, Error = ()> + Send>;

    fn fetch(&self, args: Self::Args, target: PathBuf) -> Self::Progress {
        unimplemented!()
    }
}
