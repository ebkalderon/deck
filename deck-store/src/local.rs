use std::ffi::OsString;

use deck_core::{Manifest, ManifestId, OutputId, Platform};
use deck_binary_cache::{BinaryCache, BinaryCacheFuture};
use deck_repository::Repository;

use super::{BuildStream, CheckContents, Repair, Store, StoreFuture};

pub mod builder;
pub mod context;
pub mod dir;
pub mod store_dir;

mod file;

const TEMP_DIR_NAME: &str = "tmp";
const VAR_DIR_NAME: &str = "var";

#[derive(Debug)]
pub struct LocalStore;

impl LocalStore {
    pub async fn add_binary_cache<B: BinaryCache>(&mut self, _cache: B) -> Result<(), ()> {
        unimplemented!()
    }

    pub async fn add_remote_store<S: Store>(&mut self, _store: S) -> Result<(), ()> {
        unimplemented!()
    }

    pub async fn add_repository<R: Repository>(&mut self, _repo: R) -> Result<(), ()> {
        unimplemented!()
    }
}

impl BinaryCache for LocalStore {
    fn query<'a>(&'a mut self, _id: &'a OutputId) -> BinaryCacheFuture<'a, ()> {
        unimplemented!()
    }
}

impl Store for LocalStore {
    fn supported_platforms<'a>(&'a self) -> StoreFuture<'a, Vec<Platform>> {
        unimplemented!()
    }

    fn build_manifest(&mut self, _manifest: Manifest) -> BuildStream {
        unimplemented!()
    }

    fn get_build_log<'a>(&'a mut self, _id: &'a ManifestId) -> StoreFuture<'a, Option<OsString>> {
        unimplemented!()
    }

    fn verify<'a>(&'a mut self, _check: CheckContents, _repair: Repair) -> StoreFuture<'a, ()> {
        unimplemented!()
    }
}
