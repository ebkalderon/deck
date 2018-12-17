extern crate deck_daemon;

use std::path::PathBuf;

/*
fn prototype_v2() {
    let mut store = daemon.store();

    // Adding a source to a store.
    //
    // Saved in `${STORE_DIR}/sources/`.
    let mut source = Source::new("foo/bar.sh");
    let source: Entry<Source> = store.add(source).signature("1234567890abcdef").unwrap();

    // Add derivation manifest to a store.
    //
    // Saved in `${STORE_DIR}/manifests/`.
    let mut manifest: Manifest = Manifest::from_str("...").unwrap();
    let manifest: Entry<Manifest> = store.add(drv).unwrap();

    // Build a derivation from a manifest.
    //
    // Saved in `${STORE_DIR}/derivations/${target}/${name}-${version}-${hash}/`.
    let drv: Entry<Derivation> = store.build_derivation(&manifest).unwrap();
    println!("{:?}", drv.path());

    // Add archived derivation to a store.
    //
    // Saved in `${STORE_DIR}/derivations/${target}/${name}-${version}-${hash}/`.
    let ar = DerivationArchive::open("foo/drv.tar.xz", "foo/drv.info.toml").unwrap();
    let drv: Entry<Derivation> = store.add(ar).unwrap();
}*/
