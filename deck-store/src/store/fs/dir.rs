pub use self::manifests::{ManifestsDir, ManifestsInput};
pub use self::outputs::OutputsDir;
pub use self::path::{DirectoryPath, LockedPath, ReadPath, WritePath};
pub use self::sources::SourcesDir;

use std::fmt::Debug;
use std::future::Future;
use std::path::Path;
use std::pin::Pin;

use crate::id::FilesystemId;

mod manifests;
mod outputs;
mod path;
mod sources;

// NOTE: All this noise has been to work fine with a simple `async fn`, with no need for associated
// types, this type alias, or `Pin<Box<_>>`. Replace _immediately_ once `async fn` in traits is
// stabilized in Rust.

pub type DirFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, ()>> + Send + 'a>>;

pub trait Directory: Debug + Send + Sync {
    type Id: FilesystemId;
    type Input: Send;
    type Output: Send;

    const NAME: &'static str;

    fn precompute_id<'a>(&'a self, input: &'a Self::Input) -> DirFuture<'a, Self::Id>;
    fn compute_id<'a>(&'a self, path: &'a ReadPath) -> DirFuture<'a, Self::Id>;
    fn read<'a>(&'a self, path: &'a ReadPath) -> DirFuture<'a, Option<Self::Output>>;
    fn write<'a>(
        &'a self,
        path: &'a mut WritePath,
        input: Self::Input,
    ) -> DirFuture<'a, Self::Output>;
}

#[derive(Debug)]
pub struct State<D> {
    directory: D,
}

impl<D> State<D>
where
    D: Directory + 'static,
    D::Id: 'static,
    D::Input: 'static,
    D::Output: 'static,
{
    pub fn new(directory: D) -> Self {
        State { directory }
    }

    pub fn contains(&self, prefix: &Path, id: &D::Id) -> bool {
        let path = prefix.join(D::NAME).join(id.to_path());
        path.exists()
    }

    pub async fn read<'a>(
        &'a self,
        prefix: &'a Path,
        id: &'a D::Id,
    ) -> Result<Option<D::Output>, ()> {
        let path = DirectoryPath::new(prefix, D::NAME, id.clone());

        if let Some(read_path) = await!(path.lock_reading())? {
            await!(self.directory.read(&read_path))
        } else {
            Ok(None)
        }
    }

    pub async fn write<'a>(
        &'a self,
        prefix: &'a Path,
        input: D::Input,
    ) -> Result<(D::Id, D::Output), ()> {
        // Since the `D::Id` of a given `D::Input` is not known ahead of time, we compute a
        // temporary one here and use it to mark ourselves as writing. A new `D::Id`, which may be
        // different from the temporary one, will be returned from `Directory::write()` along with
        // the `D::Output`.
        let temp_id = await!(self.directory.precompute_id(&input))?;
        let path = DirectoryPath::new(prefix, D::NAME, temp_id.clone());
        let locked = await!(path.lock_writing())?;

        match locked {
            LockedPath::ReadExisting(path) => {
                println!("already in store");
                let output = await!(self.directory.read(&path))?;
                Ok((temp_id, output.unwrap()))
            }
            LockedPath::WriteNew(mut path) => {
                let output = await!(self.directory.write(&mut path, input))?;
                let read_only = path.to_read_only();
                let new_id = await!(self.directory.compute_id(&read_only))?;
                // TODO: Register paths in database transaction here.
                await!(path.normalize_and_rename())?;
                Ok((new_id, output))
            }
        }
    }
}
