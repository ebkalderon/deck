use std::fmt::Debug;
use std::path::PathBuf;

use futures::Stream;

use self::git::FetchGit;
use self::uri::FetchUri;
use crate::store::progress::Progress;

mod git;
mod uri;

trait Fetcher: Debug + Send {
    type Args;
    type Progress: Stream<Item = Progress, Error = ()> + Send;

    fn fetch(&self, args: Self::Args, target: PathBuf) -> Self::Progress;
}

#[derive(Debug)]
pub struct Fetchers {
    git: FetchGit,
    uri: FetchUri,
}

// API examples:
//
// context
//     .fetch_source(source)
//     .map(|state| {
//         match state {
//             FetchState::Fetching(prog) => prog,
//             FetchState::Finished(id) => {
//                 // perform build
//                 // return progress of build
//             }
//         }
//     })
