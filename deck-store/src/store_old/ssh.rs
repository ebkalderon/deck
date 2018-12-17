use super::{
    AddedFuture, BuildError, Manifest, PackageFuture, PlatformFuture, Store, VerifyFuture,
};
use binary_cache::BinaryCache;

#[derive(Debug)]
pub struct SshStore;

impl Store for SshStore {
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
