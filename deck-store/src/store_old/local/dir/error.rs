use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::Error as IoError;
use std::path::PathBuf;
use std::sync::PoisonError;

use diesel::ConnectionError;
use diesel_migrations::RunMigrationsError;
use ignore::Error as IgnoreError;

#[derive(Debug)]
pub enum OpenError {
    InvalidPath(IoError),
    MissingIndex(IoError),
    InvalidIndex,
    Connection(ConnectionError),
}

impl Display for OpenError {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match *self {
            OpenError::InvalidPath(ref e) => write!(fmt, "invalid local store path: {}", e),
            OpenError::MissingIndex(ref e) => {
                write!(fmt, "unable to locate index database in directory: {}", e)
            }
            OpenError::InvalidIndex => write!(fmt, "store does not contain a valid index database"),
            OpenError::Connection(ref e) => {
                write!(fmt, "could not read store index database file: {}", e)
            }
        }
    }
}

impl Error for OpenError {
    fn cause(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            OpenError::InvalidPath(ref e) => Some(e),
            OpenError::MissingIndex(ref e) => Some(e),
            OpenError::Connection(ref e) => Some(e),
            _ => None,
        }
    }
}

impl From<ConnectionError> for OpenError {
    fn from(e: ConnectionError) -> Self {
        OpenError::Connection(e)
    }
}

#[derive(Debug)]
pub enum CreationError {
    /// Failed to create a new store directory because the path is already in use.
    AlreadyExists(PathBuf),
    /// Unable to access the base store path.
    AccessDenied(IoError),
    /// Failed to create a directory inside the store.
    CreateDirectory(IoError),
    /// Failed to create the store index database file.
    CreateDatabase(ConnectionError),
    /// Unable to read the directory tree of the store.
    ReadDirectory(IgnoreError),
    /// Unable to read the permissions of a file/directory inside the store.
    ReadPermissions(IoError),
    /// Unable to write fixed `mtime`/`atime` timestamps to a file/directory inside the store.
    WriteTimestamps(IoError),
    /// Unable to write permissions to file/directory inside the store.
    WritePermissions(IoError),
    /// Unable to run database migrations against the store index.
    RunMigrations(RunMigrationsError),
    /// Final rename of `.tmp` directory failed.
    Rename(IoError),
}

impl Display for CreationError {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match *self {
            CreationError::AlreadyExists(ref path) => {
                write!(fmt, "directory `{:?}` already exists, skipping", path)
            }
            CreationError::AccessDenied(ref e) => write!(
                fmt,
                "access to the requested directory was denied, skipping: {}",
                e
            ),
            CreationError::CreateDirectory(ref e) => {
                write!(fmt, "unable to create subdirectory in the new store: {}", e)
            }
            CreationError::CreateDatabase(ref e) => write!(
                fmt,
                "unable to create an index database file in the new store: {}",
                e
            ),
            CreationError::ReadDirectory(ref e) => {
                write!(fmt, "unable to read subdirectory in the new store: {}", e)
            }
            CreationError::ReadPermissions(ref e) => write!(
                fmt,
                "unable to read permissions of item inside the new store: {}",
                e
            ),
            CreationError::WriteTimestamps(ref e) => write!(
                fmt,
                "unable to write fixed `mtime`/`atime` timestamps to item in the new store: {}",
                e
            ),
            CreationError::WritePermissions(ref e) => write!(
                fmt,
                "unable to write permissions to an item in the new store: {}",
                e
            ),
            CreationError::RunMigrations(ref e) => write!(
                fmt,
                "failed to run database migrations against the index database: {}",
                e
            ),
            CreationError::Rename(ref e) => {
                write!(fmt, "final rename of `.tmp` directory failed: {}", e)
            }
        }
    }
}

impl Error for CreationError {
    fn cause(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            CreationError::AccessDenied(ref e) => Some(e),
            CreationError::CreateDirectory(ref e) => Some(e),
            CreationError::CreateDatabase(ref e) => Some(e),
            CreationError::ReadDirectory(ref e) => Some(e),
            CreationError::ReadPermissions(ref e) => Some(e),
            CreationError::WriteTimestamps(ref e) => Some(e),
            CreationError::WritePermissions(ref e) => Some(e),
            CreationError::RunMigrations(ref e) => Some(e),
            CreationError::Rename(ref e) => Some(e),
            _ => None,
        }
    }
}

impl From<ConnectionError> for CreationError {
    fn from(e: ConnectionError) -> Self {
        CreationError::CreateDatabase(e)
    }
}

impl From<IgnoreError> for CreationError {
    fn from(e: IgnoreError) -> Self {
        CreationError::ReadDirectory(e)
    }
}

impl From<RunMigrationsError> for CreationError {
    fn from(e: RunMigrationsError) -> Self {
        CreationError::RunMigrations(e)
    }
}
