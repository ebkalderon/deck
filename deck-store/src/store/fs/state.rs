use std::path::{Path, PathBuf};
use std::sync::Arc;

use futures_preview::compat::Future01CompatExt;
use tokio::fs;

use super::dir::Directory;
use crate::id::FilesystemId;

const TEMP_DIR_NAME: &str = "tmp";

#[derive(Debug)]
pub struct State<D> {
    directory: Arc<D>,
}

impl<D> State<D>
where
    D: Directory + 'static,
    D::Id: 'static,
    D::Input: 'static,
    D::Output: 'static,
{
    pub fn new(directory: D) -> Self {
        State {
            directory: Arc::new(directory),
        }
    }

    pub fn contains(&self, prefix: &Path, id: &D::Id) -> bool {
        let path = prefix.join(D::NAME).join(id.to_path());
        path.exists()
    }

    pub async fn read<'a>(&'a self, prefix: &'a Path, id: &'a D::Id) -> Result<Option<D::Output>, ()> {
        let path = prefix.join(D::NAME).join(id.to_path());
        // FIXME: Check if path exists. If path does not exist, try to lock it exclusively. Check
        // again if it doesn't exist. If it does, read it and release the lock. If not, release the
        // lock and return `Ok(None)`.
        await!(self.directory.read(&path, id))
    }

    pub async fn write<'a>(&'a self, prefix: &'a Path, input: D::Input) -> Result<(D::Id, D::Output), ()> {
        // Since the `D::Id` of a given `D::Input` is not known ahead of time, we compute a
        // temporary one here and use it to mark ourselves as writing. A new `D::Id`, which may be
        // different from the temporary one, will be returned from `Directory::write()` along with
        // the `D::Output`.
        let temp_id = await!(self.directory.precompute_id(&input))?;

        // FIXME: Check if path exists. If path does not exist, try to lock it exclusively. Check
        // again if it doesn't exist. If it does, call read() and exit early. If not, continue.

        let temp_path = prefix.join(TEMP_DIR_NAME).join(temp_id.to_path());
        let output = await!(self.directory.write(&temp_path, input))?;
        let new_id = await!(self.directory.compute_id(&temp_path))?;

        // TODO: Need to normalize permissions of temp_path here.

        let final_path = prefix.join(D::NAME).join(new_id.to_path());
        println!("renaming {:?} -> {:?}", temp_path, final_path);
        await!(fs::rename(&temp_path, &final_path).compat()).map_err(|_| ())?;

        // TODO: Register paths in database transaction here.

        Ok((new_id, output))
    }
}
