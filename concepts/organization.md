# Crate Organizaton

## File system layout

* `/` - Cargo workspace
  * `deck` - client binaries, depends on everything else (ensures atomic upgrade)
  * `deck-client` - connecting to and interacting with a store (daemon or local)
  * `deck-daemon` - daemon impl, manifest parsing
  * `deck-protocol` - wire schema for client and server
  * `deck-store` - store and binary cache impls
  * `deck-worker` - worker impl

## Logical crate layout

```
+-----------------------------------------------------+
|                        deck                         |
+------+-------------------+-------------------+------+
       |                   |                   |
       v                   v                   v
+-------------+     +-------------+     +-------------+
| deck-client |     | deck-daemon |     | deck-worker |
+---+-----+---+     +---+-----+---+     +------+------+
    |     |             |     |                |
    |     |             |     | +--------------+
    |     +-------------|-----|-|-----------------+
    |                   |     +-|-----------+     |
    |                   |       |           |     |
    v                   v       v           v     v
+---------------------------------+     +-------------+
|          deck-protocol          |     | deck-store  |
+---------------------------------+     +-------------+
```

## Note about single/multi-user modes

* Client and server implementations are parameterized by a type, allowing for
  single and multi user versions to be selected at compile-time.
* When building in multi-user mode, both `deck-client` and `deck-daemon` are
  configured in the `deck` crate to communicate via gRPC through a Unix domain
  socket. Only `deck-daemon` interacts with the store directly.
* When building in single-user mode, `deck-client` interacts with the store
  directly, and the `deck-daemon` crate and binary never gets built.
