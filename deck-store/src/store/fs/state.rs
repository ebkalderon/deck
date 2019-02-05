use std::path::Path;

use super::dir::{Directory, DirectoryPath, LockedPath};
use crate::id::FilesystemId;

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
