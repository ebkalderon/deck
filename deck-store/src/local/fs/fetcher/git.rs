use std::path::PathBuf;

use futures::Stream;

use super::Fetcher;
use crate::progress::Progress;

#[derive(Debug)]
pub struct FetchGit;

impl Fetcher for FetchGit {
    type Args = ();
    type Progress = Box<dyn Stream<Item = Progress, Error = ()> + Send>;

    fn fetch(&self, args: Self::Args, target: PathBuf) -> Self::Progress {
        unimplemented!()
    }
}
