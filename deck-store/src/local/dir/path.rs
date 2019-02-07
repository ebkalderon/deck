use std::io::Write;
use std::path::{Display, Path, PathBuf};

use deck_core::FilesystemId;
use futures::future::poll_fn;
use futures_preview::compat::Future01CompatExt;
use futures_preview::future::{FutureExt, TryFutureExt};
use tokio::fs::{self, File, OpenOptions};

use crate::local::{TEMP_DIR_NAME, VAR_DIR_NAME};
use crate::local::file::{FileFutureExt, LockedFile};

const LOCK_FILE_EXT: &str = "lock";
const MARK_LOCK_AS_STALE: &[u8] = "stale".as_bytes();

#[derive(Debug, Eq, PartialEq)]
pub enum LockedPath {
    WriteNew(WritePath),
    ReadExisting(ReadPath),
}

#[derive(Debug)]
pub struct DirectoryPath<I> {
    root: PathBuf,
    temp_path: PathBuf,
    lock_path: PathBuf,
    id: I,
}

impl<I: FilesystemId> DirectoryPath<I> {
    pub fn new<P: AsRef<Path>, S: AsRef<str>>(prefix: P, directory: S, id: I) -> Self {
        let mut lock_path = prefix.as_ref().join(VAR_DIR_NAME).join(id.to_path());
        lock_path.set_extension(LOCK_FILE_EXT);

        DirectoryPath {
            root: prefix.as_ref().join(directory.as_ref()).join(id.to_path()),
            temp_path: prefix.as_ref().join(TEMP_DIR_NAME).join(id.to_path()),
            lock_path,
            id,
        }
    }

    pub async fn lock_reading(self) -> Result<Option<ReadPath>, ()> {
        if self.root.exists() {
            Ok(Some(ReadPath::new(self.root, self.id, None)))
        } else {
            let guard = await!(LockFileGuard::new(self.lock_path))?;
            if self.root.exists() {
                Ok(Some(ReadPath::new(self.root, self.id, Some(guard))))
            } else {
                Ok(None)
            }
        }
    }

    pub async fn lock_writing(self) -> Result<LockedPath, ()> {
        if self.root.exists() {
            let should_read = ReadPath::new(self.root, self.id, None);
            Ok(LockedPath::ReadExisting(should_read))
        } else {
            let guard = await!(LockFileGuard::new(self.lock_path))?;
            if self.root.exists() {
                let should_read = ReadPath::new(self.root, self.id, None);
                Ok(LockedPath::ReadExisting(should_read))
            } else {
                let should_write = WritePath::new(self.root, self.temp_path, self.id, guard);
                Ok(LockedPath::WriteNew(should_write))
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct WritePath {
    final_path: PathBuf,
    temp_path: PathBuf,
    id: String,
    guard: LockFileGuard,
}

impl WritePath {
    fn new<I: ToString>(out: PathBuf, temp: PathBuf, id: I, guard: LockFileGuard) -> Self {
        WritePath {
            final_path: out,
            temp_path: temp,
            id: id.to_string(),
            guard,
        }
    }

    pub fn as_id(&self) -> &str {
        &self.id
    }

    pub fn as_path(&self) -> &Path {
        &self.temp_path
    }

    pub fn display(&self) -> Display {
        self.temp_path.display()
    }

    pub async fn create_file(&mut self) -> Result<LockedFile, ()> {
        await!(File::create(self.temp_path.clone())
            .lock_exclusive()
            .compat()
            .boxed()
            .map_err(|_| ()))
    }

    pub fn copy_from<P: AsRef<Path>>(&mut self, source: P) -> Result<u64, ()> {
        std::fs::copy(source, self.temp_path.clone()).map_err(|_| ())
    }

    pub fn to_read_only(&self) -> ReadPath {
        ReadPath {
            path: self.temp_path.clone(),
            id: self.id.clone(),
            guard: None,
        }
    }

    pub async fn normalize_and_rename(self) -> Result<(), ()> {
        if self.temp_path.exists() {
            // TODO: Need to normalize permissions here.
            await!(fs::rename(self.temp_path, self.final_path)
                .compat()
                .map_err(|_| ()))?;
        }

        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ReadPath {
    path: PathBuf,
    id: String,
    guard: Option<LockFileGuard>,
}

impl ReadPath {
    fn new<I: ToString>(path: PathBuf, id: I, guard: Option<LockFileGuard>) -> Self {
        ReadPath {
            path,
            id: id.to_string(),
            guard,
        }
    }

    pub fn as_id(&self) -> &str {
        &self.id
    }

    pub fn as_path(&self) -> &Path {
        &self.path
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    pub async fn open_file(&self) -> Result<LockedFile, ()> {
        await!(File::open(self.path.clone())
            .lock_shared()
            .compat()
            .boxed()
            .map_err(|_| ()))
    }
}

impl Drop for ReadPath {
    fn drop(&mut self) {
        self.guard.take();
    }
}

#[derive(Debug)]
struct LockFileGuard {
    file: LockedFile,
    path: PathBuf,
}

impl LockFileGuard {
    async fn new(path: PathBuf) -> Result<Self, ()> {
        let opening = OpenOptions::new()
            .write(true)
            .create(true)
            .open(path.clone())
            .lock_exclusive()
            .compat()
            .boxed()
            .map_err(|_| ());

        let file = await!(opening)?;
        let (mut file, metadata) = await!(file.metadata().compat().map_err(|_| ()))?;

        if !metadata.len() == 0 {
            await!(poll_fn(|| file.poll_set_len(0)).compat()).map_err(|_| ())?;
        }

        Ok(LockFileGuard { file, path })
    }
}

impl Drop for LockFileGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
        let _ = self.file.write_all(MARK_LOCK_AS_STALE);
    }
}

impl Eq for LockFileGuard {}

impl PartialEq for LockFileGuard {
    fn eq(&self, other: &LockFileGuard) -> bool {
        self.path == other.path
    }
}
