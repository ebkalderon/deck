# Displaying progress

The daemon has a continuously running threadpool which fetches sources and
performs builds. Each client has its own `stdout`/`stderr` and `MultiProgress`
to use for displaying progress. `deck-store` and `deck-daemon` don't use
`indicatif` directly, but they return the current progress for each operation,
allowing the `deck-client` downstream to assign progress bars to them.

The basic flow goes like this:

1. Client connects to the daemon via gRPC over a Unix domain socket.
2. Client submits to the daemon a manifest to be built.
3. Daemon receives the manifest and computes a build graph.
4. Daemon spawns a future for each node in the build graph, with each future
   consisting of:
   1. A `join_all()` on any dependencies that need to be built. (`Waiting`).
   2. And then, a `join_all()` on all sources that need to be downloaded.
      (`Downloading`).
   3. And then, the code necessary to set up the build directory. (`Preparing`)
   4. And then, an async `fork()` of the current process running the builder
      code in a process sandbox. The `stdout` and `stderr` pipes are turned into
      a stream of type (`Vec<u8>`, `Vec<u8>`).
3. Daemon returns to the client a future containing a stream of the continuous
   build progress. The future resolves when the build graph has been computed
   and at least one build has started.
4. Client performs a `take_while()` on the stream and matches on each kind of
   progress received.
   1. `Started`: First message ever received in the stream. Contains a list of
      packages to be built and the timestamp of when the build started.
   2. `Pending`: Build is ongoing. Multiple `Pending` messages will be sent per
      package. Until the first `Pending` message is received for a package, it's
      considered to be "waiting." Each message corresponds to a package ID, the
      current phase of the package (`PREPARING`, `DOWNLOADING`, `CONFIGURING`,
      `COMPILING`, `TESTING`, `FINALIZING`), a one-line description of the
      current phase, the number of the current phase, and the total number of
      phases in this package's build process. If a package output is being
      fetched from a binary cache, the `CONFIGURING` phase won't even appear and
      it will skip straight to the `Complete` message.
   3. `Complete`: A package was built successfully. This will contain the
      package ID that was built and installed, the delta size of the
      installation in bytes, and the timestamp at which the entire build was
      completed.
   4. `Error`: Something went wrong and the build was interrupted. Contains the
      package ID that failed, the entire contents of the worker's `stdout` and
      `stderr`, the error `Status` (`BUILD_FAILED`, `DEPENDENCY_FAILED`,
      `TIMED_OUT`, `WORKER_DISCONNECTED`, etc.), and the timestamp at which the
      build failed.
5. On the `Started` message being received, the client creates progress bars for
   each package being built and associates them with the `MultiProgress` that is
   part of the client. Each subsequent `Pending` received updates the progress
   bars. Once all packages are `Complete` or an `Error` is received, either
   finish all the progress bars and print a success message with the delta size
   of the installation in bytes, or hide the progress bars and print all the
   `stdout` and `stderr` of the failed build.
6. Client spawns this future with `tokio::spawn()`.
7. Client performs a blocking `join()` on the `MultiProgress` so the progress
   bars can all be updated.
8. Once the build has halted and the progress bars have all been finished, the
   program continues.
9. The `tokio` runtime blocks until the stream closes and the future ends, at
   which point the runtime shuts down and the program ends.
