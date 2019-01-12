use std::sync::Arc;

use super::fs::{HttpsClient, StoreDir};

#[derive(Clone, Debug)]
pub struct Context {
    pub client: Arc<HttpsClient>,
    pub store: Arc<StoreDir>,
}

impl Context {
    pub fn new(store: Arc<StoreDir>, client: Arc<HttpsClient>) -> Self {
        Context { store, client }
    }
}
