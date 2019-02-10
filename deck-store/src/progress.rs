use chrono::{offset::Utc, DateTime};
use deck_core::ManifestId;
use futures_preview::channel::mpsc::{self, Receiver, Sender};

pub(crate) type ProgressSender = Sender<Result<Progress, ()>>;
pub(crate) type ProgressReceiver = Receiver<Result<Progress, ()>>;

pub(crate) fn progress_channel(buffer: usize) -> (ProgressSender, ProgressReceiver) {
    mpsc::channel(buffer)
}

#[derive(Clone, Debug)]
pub enum Progress {
    Downloading(Downloading),
    Blocked { package_id: ManifestId },
    Building(Building),
    Finished(Finished),
}

#[derive(Clone, Debug)]
pub struct Downloading {
    pub package_id: ManifestId,
    pub source: String,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
}

#[derive(Clone, Debug)]
pub enum BuildingStatus {
    Waiting,
    Preparing,
    Configuring,
    Compiling,
    Testing,
    Finalizing,
}

#[derive(Clone, Debug)]
pub struct Building {
    pub package_id: ManifestId,
    pub status: BuildingStatus,
    pub current_task: u32,
    pub total_tasks: u32,
    pub description: String,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

#[derive(Clone, Debug)]
pub enum FinishedStatus {
    Built,
    Memoized,
    Downloaded,
}

#[derive(Clone, Debug)]
pub struct Finished {
    pub package_id: ManifestId,
    pub status: FinishedStatus,
    pub timestamp: DateTime<Utc>,
}
