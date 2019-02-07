use std::future::Future;
use std::pin::Pin;
use std::task::{LocalWaker, Poll};
use std::time::{Duration, Instant};

use chrono::Utc;
use deck_core::Manifest;
use futures_preview::compat::Future01CompatExt;
use futures_preview::future::{self, FutureExt, TryFutureExt};
use futures_preview::stream::{self, Stream};

use crate::local::context::Context;
use crate::progress::{Building, BuildingStatus, Finished, FinishedStatus, Progress};

#[must_use = "streams do nothing unless polled"]
pub struct BuildManifest(Pin<Box<dyn Stream<Item = Result<Progress, ()>> + Send>>);

impl BuildManifest {
    pub fn new(_ctx: Context, manifest: Manifest) -> Self {
        let id = manifest.compute_id();

        let building = Progress::Building(Building {
            package_id: id.clone(),
            current_task: 3,
            total_tasks: 5,
            status: BuildingStatus::Compiling,
            description: "make all".to_string(),
            stdout: Vec::new(),
            stderr: Vec::new(),
        });

        let finished = Progress::Finished(Finished {
            package_id: id,
            status: FinishedStatus::Built,
            timestamp: Utc::now(),
        });

        let when = Instant::now() + Duration::from_millis(1000);
        let delay = tokio::timer::Delay::new(when);

        let stream = stream::futures_ordered(vec![
            Box::pin(future::ok(building)) as Pin<Box<dyn Future<Output = _> + Send>>,
            Box::pin(delay.compat().then(|_| future::ok(finished)))
                as Pin<Box<dyn Future<Output = _> + Send>>,
        ]);

        BuildManifest(Box::pin(stream))
    }
}

impl Stream for BuildManifest {
    type Item = Result<Progress, ()>;

    fn poll_next(mut self: Pin<&mut Self>, lw: &LocalWaker) -> Poll<Option<Self::Item>> {
        self.0.as_mut().poll_next(lw)
    }
}
