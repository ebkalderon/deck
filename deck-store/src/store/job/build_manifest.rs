use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::time::{Duration, Instant};

use futures::{future, stream, Future, Poll, Stream};

use super::super::context::Context;
use super::progress::{Building, BuildingStatus, Finished, FinishedStatus, Progress};
use super::IntoJob;
use chrono::Utc;
use package::Manifest;

#[must_use = "streams do nothing unless polled"]
pub struct BuildManifest(Box<dyn Stream<Item = Progress, Error = ()> + Send>);

impl BuildManifest {
    pub fn new(_ctx: Context, manifest: Manifest) -> Self {
        let building = Progress::Building(Building {
            package_id: manifest.id().clone(),
            current_task: 3,
            total_tasks: 5,
            status: BuildingStatus::Compiling,
            description: "make all".to_string(),
            stdout: Vec::new(),
            stderr: Vec::new(),
        });

        let finished = Progress::Finished(Finished {
            package_id: manifest.id().clone(),
            status: FinishedStatus::Built,
            timestamp: Utc::now(),
        });

        let when = Instant::now() + Duration::from_millis(1000);
        let delay = tokio::timer::Delay::new(when);

        let stream = stream::futures_ordered(vec![
            Box::new(future::ok(building)) as Box<dyn Future<Item = _, Error = _> + Send>,
            Box::new(delay.map(move |_| finished).map_err(|_| ()))
                as Box<dyn Future<Item = _, Error = _> + Send>,
        ]);

        BuildManifest(Box::new(stream))
    }
}

impl Debug for BuildManifest {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.debug_tuple(stringify!(BuildManifest))
            .field(&"Box<dyn Stream<Item = Progress, Error = ()> + Send>")
            .finish()
    }
}

impl IntoJob for BuildManifest {}

impl Stream for BuildManifest {
    type Item = Progress;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.0.poll()
    }
}
