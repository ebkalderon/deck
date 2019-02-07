use std::path::PathBuf;

use deck_store::core::Manifest;
use deck_store::local::fs::StoreDir;
use futures_preview::future::{FutureExt, TryFutureExt};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref STORE: StoreDir = {
        let path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/store"));
        println!("{:?}", path);
        StoreDir::open(path).unwrap()
    };
}

fn main() {
    let manifest = Manifest::build("hello", "1.0.0", "fc3j3vub6kodu4jtfoakfs5xhumqi62m", None)
        .finish()
        .expect("failed to create manifest");

    // FIXME: `tokio::fs` requires a `tokio` executor, but `StoreDir` produces a non-`'static`
    // future which `tokio` cannot execute. `tokio::current_thread::block_on_all()` can execute
    // them, but it panics on `tokio::fs::File` because the entire `tokio-fs` crate doesn't
    // work with `current_thread` and requires a `tokio`-specific threadpool with `::blocking`.
    // See this issue for details: https://github.com/tokio-rs/tokio/issues/386
    //
    // `lazy_static!` was the only way I was able to get this test working.
    let write1 = STORE
        .write_manifest(manifest)
        .map_ok(|_| ())
        .boxed()
        .compat();

    tokio::run(write1);
}
