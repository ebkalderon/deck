use std::path::PathBuf;

use super::{AddedFuture, Manifest, PackageFuture, PlatformFuture, Store, VerifyFuture};
use binary_cache::BinaryCache;

use self::dir::{CreationError, OpenError, StoreDirectory};

mod dir;

#[derive(Debug)]
pub enum LocalStoreError {
    CreationFailed(CreationError),
    OpenFailed(OpenError),
}

impl From<CreationError> for LocalStoreError {
    fn from(e: CreationError) -> Self {
        LocalStoreError::CreationFailed(e)
    }
}

impl From<OpenError> for LocalStoreError {
    fn from(e: OpenError) -> Self {
        LocalStoreError::OpenFailed(e)
    }
}

#[derive(Debug)]
pub struct LocalStore {
    dir: StoreDirectory,
}

impl LocalStore {
    pub fn new<P: Into<PathBuf>>(path: P) -> Result<Self, LocalStoreError> {
        Ok(LocalStore {
            dir: StoreDirectory::open(path.into())?,
        })
    }

    pub fn create_in<P: Into<PathBuf>>(path: P) -> Result<Self, LocalStoreError> {
        Ok(LocalStore {
            dir: StoreDirectory::create_in(path.into())?,
        })
    }
}

impl Store for LocalStore {
    fn supported_platforms(&self) -> PlatformFuture {
        unimplemented!()
    }

    fn build_package(&mut self, _manifest: &Manifest) -> PackageFuture {
        unimplemented!()
    }

    fn add_binary_cache<B: BinaryCache>(&mut self, _cache: B) -> AddedFuture {
        unimplemented!()
    }

    fn add_remote_store<S: Store>(&mut self, _store: S) -> AddedFuture {
        unimplemented!()
    }

    fn verify(&mut self, _repair: bool) -> VerifyFuture {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn create_example_store() {
    //     let thing = Box::new(
    //         LocalStore::create_in(concat!(env!("CARGO_MANIFEST_DIR"), "/store"))
    //             .expect("store error"),
    //     );
    // }
}
