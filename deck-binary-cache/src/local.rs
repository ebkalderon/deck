use deck_core::OutputId;

use crate::{BinaryCache, BinaryCacheFuture};

#[derive(Debug)]
pub struct LocalCache;

impl BinaryCache for LocalCache {
    fn query<'a>(&'a mut self, _id: &'a OutputId) -> BinaryCacheFuture<'a, ()> {
        unimplemented!()
    }
}
