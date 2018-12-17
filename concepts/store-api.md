# Client

```rust
fn deck_build() {
    let client = Client::connect("/var/deck.sock").unwrap();
    let id = client.download()
        .source("bar.tar.gz")
        .url("https://foo.com/bar.tar.gz", "1234567890abcdef")
        .unwrap();

    let manifest = /* ... */;
    let id = client.build_derivation(manifest).unwrap();
}
```

# Daemon

```rust
fn deck_daemon() {
    let cfg = std::env::var("DECK_CONFIG_DIR").unwrap_or("/etc/deck");
    let mut daemon = Daemon::new(cfg);
    daemon.run().unwrap();
}
```
