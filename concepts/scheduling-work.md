# Scheduling work with an immutable store

To ensure work gets scheduled and completed in a timely manner while maintaining
the integrity of the Deck store, it may be necessary to batch up things like
download requests and write operations to the store in a persistent threadpool
of some kind. Below are a few potential ideas on how to implement this:

1. All sources needed to be downloaded are fetched and saved to the store ad-hoc
   _by each worker binary that needs it_, with each worker having its own
   `hyper::Client` and `tokio` executor for pooling downloads together.
2. All sources needed to be downloaded are fetched and saved to the store ad-hoc
   _by the daemon_, with each download creating a new `hyper::Client` each time,
   potentially sharing a central `tokio` executor for performance reasons.
3. All sources needed to be downloaded are fetched _before the workers are
   spawned_. As such, the daemon fires up a central `hyper::Client` with a single
   `tokio` executor, requests it download a list of files, and chains operations
   to save files, measure progress, validate the hashes, rename the files to
   their correct names. and then does a `future::join_all()` on them all. Once
   all the files are downloaded, the sources are bind mounted to the workers and
   spawned.

The first option is arguably the most straightforward in terms of design,
because each worker is responsible for its own downloads. When the build graph
is computed as a graph of chained futures, the downloads can be interleaved with
the build process. But it has a few drawbacks; namely, it requires workers to
somehow coordinate the shared access to the store, and it introduces a lot of
overhead with spawning a new threadpool and `hyper::Client` for each downloader.

The second option is a bit better in that the daemon synchronizes access to the
store and downloads the resources for the workers immediately before the workers
are launched, and it's intuitive in that the downloads are interleaved in
between the build process (like #1). It also reuses the existing `tokio`
threadpool for downloads, which is a boon for performance. However, it's still
not quite perfect because the HTTP connections aren't pooled and the downloads
are sporadic.

The third option is arguably the best for performance and correctness because
the daemon synchronizes shared access to the store (like #2), the `tokio`
threadpool is reused (like #2), and there is only one `hyper::Client` in use at
any given time, allowing for HTTP connections to be pooled efficiently. However,
it prevents the build process from starting until all workers' resources have
finished downloading.

I think the optimal solution to this is a blend of options #2 and #3. In this
fourth approach, the daemon still synchronizes shared access to the store and
there is only one central `hyper::Client` in use. However, instead of blocking
all the workers until all downloads are complete, we concurrently download
sources for packages that are on the same level of the build graph, only
blocking a worker from starting unless (1) a worker hasn't received all of
required downloaded sources and/or (2) a worker's dependencies haven't finished
building.

## Example

Given a build graph like this:

```
+------+
| quux |
+--+---+
   |
   v
+-----+       +-----+
| foo |       | bar |
+--+--+       +--+--+
   |             |
   +-----+ +-----+
         | |
         v v
       +-----+
       | baz |
       +-----+
```

We split the graph into three levels, like so:

```
+------+
| quux |                      (Level 1)
+--+---+
   |
   v
+-----+       +-----+
| foo |       | bar |         (Level 2)
+--+--+       +--+--+
   |             |
   +-----+ +-----+
         | |
         v v
       +-----+
       | baz |                (Level 3)
       +-----+
```

We start downloading and building at the same time in two separate "threads" of
computation, with downloads being prioritized according to their level in the
graph. We ensure that `quux` and `bar` are downloaded concurrently first, then
`foo`, and finally `baz` gets downloaded. _Simultaneously_, we prepare the
temporary build directories in `tmp` for the workers and block the workers until
their respective downloads and dependencies are complete.

In most cases, we can be reasonably sure that the build process for a package
will take longer than the download process, so it's very likely that the
downloads for a given worker will complete before the build graph finally
reaches that node, so there should be little to no blocking. To clarify what the
advantage is to this approach, let's traverse the graph shown above from top to
bottom.

First, we concurrently download the source files for `quux` and `bar`, waiting
for both to finish before we start the workers for `quux` and `bar`. Let's say,
for the sake of simplicity of explanation, that `quux` and `bar` both complete
downloading at roughly the same time and the workers start building at roughly
the same time. While both `quux` and `bar` have started building, we might as
well begin downloading the sources for `foo`. And while `foo` is building, we
might as well pre-emptively begin downloading the sources for `baz`. There's no
reason to block downloads while the builds are going on; we should only block
builds if their downloads aren't finished yet.

As such, the dependency graph actually looks like this:

```
+---------------+                 +--------------+
| download_quux |                 | download_bar |
+------+--------+                 +--------+-----+
       |                                   |
       v                                   v
+------------+   +--------------+    +-----------+
| build_quux |   | download_foo |    | build_bar |
+------+-----+   +------+-------+    +-----+-----+
       |                |                  |
       +-----+   +------+                  |
             |   |                         |
             v   v                         |
         +-----------+   +--------------+  |
         | build_foo |   | download_baz |  |
         +-----+-----+   +-------+------+  |
               |                 |         |
               +-------+  +------+         |
                       |  |  +-------------+
                       |  |  |
                       v  v  v
                    +-----------+
                    | build_baz |
                    +-----------+
```

And the pseudo-code of the futures graph will look kind of like this:

```
let build_quux = download_quux(hyper).and_then(build_quux);

let build_foo = future::join_all(vec![download_foo(hyper), build_quux])
    .and_then(build_foo);

let build_bar = download_bar(bar).and_then(build_bar));

let build_baz = future::join_all(vec![build_foo, build_bar, download_baz(hyper)])
    .and_then(build_baz);

runtime.block_on(build_baz).unwrap();
```
