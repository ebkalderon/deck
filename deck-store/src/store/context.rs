use std::sync::Arc;

use hyper::client::{Client, HttpConnector};

use super::fs::StoreDir;

#[derive(Clone, Debug)]
pub struct Context {
    pub client: Arc<Client<HttpConnector>>,
    pub store: Arc<StoreDir>,
}

impl Context {
    pub fn new(store: Arc<StoreDir>, client: Arc<Client<HttpConnector>>) -> Self {
        Context { store, client }
    }
}
