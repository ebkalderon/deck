//! Filesystem-level file locking for Tokio.

use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::fs::{File as StdFile, Metadata};
use std::ops::{Deref, DerefMut};

use fs2::FileExt;
use futures::{try_ready, Async, Future, Poll};
use tokio::fs::{file, File};
use tokio::io::Error as IoError;

/// Trait adding file locking functionality to `tokio::File`.
///
/// This trait is named `FileFutureExt` instead of `FileExt` to avoid conflicting with
/// `fs2::FileExt`, which provides the underlying implementation.
pub trait FileFutureExt {
    /// Locks the file for exclusive usage, blocking if the file is currently locked.
    ///
    /// Unlike `fs2::FileExt::lock_exclusive()`, this method will not stall the underlying futures
    /// threadpool.
    fn lock_exclusive(self) -> LockedFuture;
    /// Locks the file for shared usage, blocking if the file is currently locked exclusively.
    ///
    /// Unlike `fs2::FileExt::lock_exclusive()`, this method will not stall the underlying futures
    /// threadpool.
    fn lock_shared(self) -> LockedFuture;
}

impl<F> FileFutureExt for F
where
    F: Future<Item = File, Error = IoError> + Send + 'static,
{
    #[inline]
    fn lock_exclusive(self) -> LockedFuture {
        LockedFuture::new(self, Kind::Exclusive)
    }

    #[inline]
    fn lock_shared(self) -> LockedFuture {
        LockedFuture::new(self, Kind::Shared)
    }
}

/// Wrapper type for `std::fs::File` which automatically unlocks the file when dropped.
#[derive(Debug)]
pub struct LockedFile(Option<File>);

impl LockedFile {
    #[inline]
    fn new(file: File) -> Self {
        LockedFile(Some(file))
    }

    /// Queries metadata about the underlying file.
    #[inline]
    pub fn metadata(self) -> MetadataFuture {
        MetadataFuture::new(self)
    }
}

impl Deref for LockedFile {
    type Target = File;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().expect("inner `tokio::fs::File` is empty!")
    }
}

impl DerefMut for LockedFile {
    fn deref_mut(&mut self) -> &mut File {
        self.0.as_mut().expect("inner `tokio::fs::File` is empty!")
    }
}

impl Drop for LockedFile {
    fn drop(&mut self) {
        let file = self.0.take().expect("inner `tokio::fs::File` is empty!");
        let std_file = file.into_std();
        std_file.unlock().expect("failed to unlock file!");
    }
}

/// A `Future` which attempts to acquire a file lock and will resolve to a [`LockedFile`].
///
/// [`LockedFile`]: ./struct.LockedFile.html
#[derive(Debug)]
#[must_use = "futures do nothing unless polled"]
pub struct LockedFuture {
    kind: Kind,
    state: State,
}

impl LockedFuture {
    fn new<F>(inner: F, kind: Kind) -> Self
    where
        F: Future<Item = File, Error = IoError> + Send + 'static,
    {
        LockedFuture {
            kind,
            state: State::Pending(Box::new(inner)),
        }
    }
}

impl Future for LockedFuture {
    type Item = LockedFile;
    type Error = IoError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let std_file = match self.state {
            State::Pending(ref mut inner) => {
                let file = try_ready!(inner.poll());
                file.into_std()
            }
            State::Blocking(ref mut inner) => {
                inner.take().expect("inner `std::fs::File` was empty")
            }
            State::Locked => {
                panic!("File already locked and returned from future! Cannot lock again.");
            }
        };

        let lock_attempt = match self.kind {
            Kind::Exclusive => std_file.try_lock_exclusive(),
            Kind::Shared => std_file.try_lock_shared(),
        };

        match lock_attempt {
            Ok(()) => {
                let file = LockedFile::new(File::from_std(std_file));
                self.state = State::Locked;
                Ok(Async::Ready(file))
            }
            Err(ref e) if e.kind() == fs2::lock_contended_error().kind() => {
                self.state = State::Blocking(Some(std_file));
                Ok(Async::NotReady)
            }
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug)]
enum Kind {
    Exclusive,
    Shared,
}

enum State {
    Pending(Box<dyn Future<Item = File, Error = IoError> + Send>),
    Blocking(Option<StdFile>),
    Locked,
}

impl Debug for State {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        let mut debug = fmt.debug_tuple(stringify!(State));

        match *self {
            State::Pending(_) => debug
                .field(&"Box<dyn Future<Item = tokio::fs::File, Error = tokio::io::Error> + Send>")
                .finish(),
            State::Blocking(ref inner) => debug.field(inner).finish(),
            State::Locked => debug.finish(),
        }
    }
}

#[derive(Debug)]
#[must_use = "futures do nothing unless polled"]
pub struct MetadataFuture {
    inner: file::MetadataFuture,
    wrapper: Option<LockedFile>,
}

impl MetadataFuture {
    pub(super) fn new(mut file: LockedFile) -> Self {
        let std = file.0.take().expect("`LockedFile` must be initialized");
        MetadataFuture {
            inner: std.metadata(),
            wrapper: Some(file),
        }
    }
}

impl Future for MetadataFuture {
    type Item = (LockedFile, Metadata);
    type Error = IoError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let (file, metadata) = try_ready!(self.inner.poll());
        let mut locked = self.wrapper.take().expect("inner `LockedFile` was empty");
        locked.0 = Some(file);
        Ok(Async::Ready((locked, metadata)))
    }
}
