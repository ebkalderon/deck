use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::rc::Rc;

use rusoto_s3::S3;

use super::super::BinaryCache;

pub struct S3Cache<S> {
    client: Rc<S>,
}

impl<S> Debug for S3Cache<S> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.debug_struct("S3Cache")
            .field("client", &"Rc<[impl]>")
            .finish()
    }
}

impl<S: S3> S3Cache<S> {
    pub fn new(cache: S) -> Self {
        S3Cache {
            client: Rc::new(cache),
        }
    }
}

impl<S: S3> BinaryCache for S3Cache<S> {
    fn query() {
        unimplemented!()
    }
}
