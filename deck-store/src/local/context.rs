use std::sync::Arc;

use hyper::{client::HttpConnector, Client};
use hyper_tls::HttpsConnector;

use super::store_dir::StoreDir;

pub(crate) type HttpsClient = Client<HttpsConnector<HttpConnector>>;

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
