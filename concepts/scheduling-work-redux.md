# Scheduling Work: Redux

The original design of [Scheduling Work](./scheduling-work.md) made a few
assumptions that make the design untenable. These assumptions are as follows:

1. It assumed that all manifests are pre-loaded into memory before the builder
   is started. We could load them all ahead of time, sure, but the progress
   won't be reflected in the resulting `BuildStream`, which is pretty shitty.
   The biggest problem is that the manifest loading/fetching can't be delayed
   until build time because the entire dependency graph must be known ahead of
   time before the futures are executed.
2. It didn't account for substituters. If a manifest exists on disk, we should
   load it. If it doesn't exist, we should check if any substituters supply it.
   If a substituter supplies it, we should fetch the manifest from there. If
   not, we should cancel the build with error.
3. It didn't account for output fetching. If all outputs belonging to a manifest
   exist on disk, the package is already installed and we shouldn't need to
   build anything further. If they do not, we should check our binary caches and
   substituters if they can supply them. If they can, we should fetch the
   outputs and install them. If they cannot, _then and only then_ should we
   continue building the package locally.

## Ideas on how to fix this

### Idea 1: Tackle assumptions 2 & 3 with a custom future

We could construct the entire build graph normally as though we are planning to
build every single package, but the `BuildFuture` produced with
`build_package_recursively()` will actually perform the following operations:

1. Check if all outputs for a `ManifestId` exist on disk. This is a _non-async_
   operation which requires access to a `Context`.
2. If yes, return a `Progress` which indicates the package is memoized. If no,
   continue to step 3.
3. Check if any substituters can supply all the outputs for the `ManifestId`.
   This involves querying each substituter if they have all these outputs. If
   they can, execute an inner future which attempts to fetch all the outputs
   using access to a `Context`. If they cannot, execute an inner future which
   builds the package as normal. This is an _async_ operation.

**The Good:**

* This would keep the recursive nature of the `Builder` nice and simple, without
  any branching logic that attempts to reason about substituters and binary
  caches.

**The Bad:**

* Doesn't exactly solve assumption 2! The manifest needs to be loaded from a
  substituter first before the build graph can be constructed, so you can't
  delay it.
* Why construct the entire build graph if the package is going to be substituted
  anyway? Lots of wasted work will be done only to be completely ignored at the
  very last step, wasting CPU cycles and memory.

**The Ugly:**

* Doesn't attempt to solve assumption 1.

#### Verdict: Flawed idea, won't entirely work. Don't do it.

### Idea 2: Handle packages already on disk by inserting a `BuildFuture` early

The prototype code which handles output fetching uses a `MaybeBuilding` monad
represented as an enum with three states: `AlreadyExists`, `Substituted`, and
`Building`. In the case of `AlreadyExists`, we could insert a `BuildFuture`
directly into the `BuildGraph` that returns a `Progress` indicating that the
package is memoized, and all subsequent steps in the builder chain could just
check if this job exists before doing anything.

**The Good:**

* This works pretty well! It completely eliminates the need for `AlreadyExists`
  in the `MaybeBuilding` enum and makes the code look a bit nicer.

**The Bad:**

* Nothing in particular. It makes the code a touch nicer.

**The Ugly:**

* While it improves the ergonomics of the existing code a bit, it doesn't
  resolve any of the 3 assumptions completely.

#### Verdict: We should do it regardless, but it's not a solution.

### Idea 3: Make the dep graph generation process itself asynchronous.

Existing implementations of `Builder` recursively traverse the manifests and
generate the final build graph synchronously, meaning that waiting for manifests
to be fetched, written to disk, and loaded into memory blocks the build process.
We could work around this problem by making the graph building process
asynchronous instead.

Each step of the `Builder` will return a future which must be resolved with
`await!()` before the next step can be taken. If a package happens to not have a
manifest available, the `Builder` will delay the graph creation process
temporarily and fetch it from a substituter, if available, and then resume. Once
the graph has been fully constructed, then the entire build is done
asynchronously and in parallel in the same way as described in the original
[Scheduling Work](./scheduling-work.md) document.

What's also interesting is that the graph building steps are still performed
sequentially as they were before, but asynchronously. This happens to mean 
that multiple `Builder`s can now also be invoked simultaneously, which is
essential for the daemon implementation.

Unlike Idea 1, this addresses all three assumptions. For example, we could do
something like this:

```rust
// Return this from gRPC server or direct function call.
async {
    let builder = Builder::new(context, manifest_id);

    let loaded = match await!(builder.load_manifest()) {
        Ok(loaded) => loaded,
        Err(fatal_err) => return make_stream(Progress::Error(fatal_err));
    };

    let maybe_substituted = match await!(loaded.try_substitute()) {
        Ok(maybe_substituted) => maybe_substituted,
        Err(fatal_err) => return make_stream(Progress::Error(fatal_err));
    };

    let sources_fetched = await!(maybe_substituted.fetch_sources());

    let deps_built = match await!(sources_fetched.build_dependencies()) {
        Ok(deps_built) => deps_built,
        Err(fatal_err) => return make_stream(Progress::Error(fatal_err));
    };

    let build_stream = deps_built.build_package();
    build_stream
}
```

Ideally, we should do this, delaying errors until the very end, but I have no
idea how to realistically accomplish this:

```rust
// Return this from gRPC server or direct function call.
async {
    let builder = Builder::new(context, manifest_id);
    let loaded = builder.load_manifest();
    let maybe_substituted = loaded.try_substituted();
    let sources_fetched = maybe_substituted.fetch_sources();
    let deps_built = sources_fetched.build_dependencies();
    let build_stream = deps_built.build_package();
    build_stream
}
```

**The Good:**

* This resolves all three assumptions made in the original design proposal in a
  reasonably elegant and scalable way.

**The Bad:**

* There is a significant increase in complexity for the `Builder` implementation
  and also in the public-facing API, requiring each step to be awaited and
  handled for errors in a short-circuiting manner.

**The Ugly:**

* How do we report progress for the sequential pre-build steps? Normally, we
  would insert `JobFuture`s with a `ProgressSender` into the `BuildGraph`, but
  here we are blocking the construction of the final `BuildStream` until all
  manifests are fetched/loaded and all viable substituters queried.

#### Verdict: Viable, but needs further investigation.

---

### UPDATE: Achieved the ideal API described at the bottom of Idea 3!

This was very tricky to achieve, but it works and it does so well. Here are the
changes that were made:

* Rather than make each step of the `Builder` an `async fn`, we store a
  `MaybeBuilding` future inside of each step coming after `Builder` and use
  `async` blocks to pass the state along without exposing its asynchronicity
  through its public API.
  * `Builder` contains all the same fields that it currently does, e.g.
    `context`, `build_graph`, `progress`, `target_id`, etc.
  * Inside of `load_manifest()`, there is an `async` block which does an
    `await!(context.store.load_manifest(&target_id))?`. This `async` block
    returns a `Result<BuildData, Error>`, where `BuildData` happens to contain
    all of the common fields present in `Builder` plus a loaded `Manifest`. This
    `BuildData` will be carried through the future chain through each step of
    the builder in the form of a `MaybeBuilding` future.
* Each subsequent step in the builder will hold a `MaybeBuilding` and an
  `Option<ProgressReceiver>`. Only in the final `build_package()` step will we
  do a `.take()` on this future and use the `ProgressReceiver` to construct our
  `BuildStream`. When we call `Builder::new_recursive()`, this field will be a
  `None` and only the top-level `Builder` will have `Some(ProgressReceiver)`.
* The `build_package_impl()` method will be turned into an `async fn` and return
  `Result<(BuildFuture, BuildGraph), Error>`. The progress channel doesn't need
  to be passed back because the `ProgressReceiver` is stored in the top-most
  `Builder`, and the `ProgressSender` is inside `BuildData` and can be cloned as
  many times as you want, no need to pass it around.
* When we call `build_package()`, we will spawn the final `BuildFuture` and
  `.select()` over the `ProgressReceiver` stream and a
  `stream::once(MaybeBuilding)`. The `MaybeBuilding` future only resolves once
  the build graph has been fully constructed, and the `ProgressReceiver` stream
  only starts moving once the build graph has been spawned. In practice, this
  means that the build progress will hold until the `MaybeBuilding` future
  resolves in `Ok(Progress::TransactionStarted)`. If it errors out before the
  build can actually start, the `ProgressReceiver` will short circuit and so
  will the entire `BuildStream`.
* The `load_manifest()` and `substitutes_available()` methods of `StoreDir` will
  be able to report progress by either:
  1. Accepting a `ProgressSender` as input.
  2. Returning a `Stream` that can be forwarded to the `ProgressSender`.
* Not exactly sure which approach is better, though. Currently leaning towards
  returning a `Stream`, but I'm not sure how to do that while also returning a
  `bool` from `substitutes_available()`.
