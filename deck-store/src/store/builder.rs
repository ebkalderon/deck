//! Asynchronous builder for a set of packages.
//!
//! # Graph structure
//!
//! The build graph that this `Builder` generates always downloads sources in parallel and builds
//! packages in parallel where possible, serializing only where necessary.
//!
//! For example, given a package dependency graph like this:
//!
//! ```text
//! +------+
//! | quux |
//! +--+---+
//!    |
//!    v
//! +-----+       +-----+
//! | foo |       | bar |
//! +--+--+       +--+--+
//!    |             |
//!    +-----+ +-----+
//!          | |
//!          v v
//!        +-----+
//!        | baz |
//!        +-----+
//! ```
//!
//! The builder will generate a build graph like this:
//!
//! ```text
//! +---------------+                 +--------------+
//! | download_quux |                 | download_bar |
//! +------+--------+                 +--------+-----+
//!        |                                   |
//!        v                                   v
//! +------------+   +--------------+    +-----------+
//! | build_quux |   | download_foo |    | build_bar |
//! +------+-----+   +------+-------+    +-----+-----+
//!        |                |                  |
//!        +-----+   +------+                  |
//!              |   |                         |
//!              v   v                         |
//!          +-----------+   +--------------+  |
//!          | build_foo |   | download_baz |  |
//!          +-----+-----+   +-------+------+  |
//!                |                 |         |
//!                +-------+  +------+         |
//!                        |  |  +-------------+
//!                        |  |  |
//!                        v  v  v
//!                     +-----------+
//!                     | build_baz |
//!                     +-----------+
//! ```
//!
//! Notice that the `download_*` steps don't have any direct dependencies, meaning they can all run
//! in parallel, even while the `build_*` steps may block.
//!
//! Once the `Builder` is eventually transformed into a `BuildStream`, the whole build graph is
//! fully self-contained inside the `BuildStream`, ready to be processed with `for_each()` on an
//! executor.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::iter::IntoIterator;

use futures::{future, Future, Poll, Stream};

use super::closure::Closure;
use super::context::Context;
use super::job::{BuildManifest, FetchSource, IntoJob};
use super::progress::{self, Progress, ProgressReceiver, ProgressSender};
use crate::id::ManifestId;

type BuildGraph = BTreeMap<ManifestId, BuildFuture>;
type Channel = (ProgressSender, ProgressReceiver);

/// Asynchronous builder for a set of packages.
#[derive(Debug)]
pub struct Builder {
    context: Context,
    closure: Closure,
    graph: BuildGraph,
    progress: Channel,
}

impl Builder {
    /// Creates a new `Builder` which will make `target` using the given `Context` and `Closure`.
    pub fn new(context: Context, closure: Closure) -> Self {
        Builder {
            context,
            closure,
            graph: BTreeMap::new(),
            progress: progress::progress_channel(4),
        }
    }

    /// Same as `Builder::new()`, except it lets you specify a pre-populated `BuildGraph` and
    /// progress channels.
    ///
    /// This constructor is only called internally, used when recursively building dependencies.
    #[inline]
    fn new_recursive(context: Context, closure: Closure, graph: BuildGraph, prog: Channel) -> Self {
        Builder {
            context,
            closure,
            graph,
            progress: prog,
        }
    }

    /// Fetches all sources required to build this package.
    pub fn fetch_sources(self) -> SourcesFetched {
        let closure = self.closure.clone();
        let manifest = closure.target_manifest();

        // TODO: We need to handle cases where a target does not need to be built.
        //
        // 1. If an already-built version of the target manifest exists on disk, push to
        //    `dependencies` an immediately-returning job for `manifest` pointing to the disk. Do
        //    not actually download any sources, manifests, or dependencies because they already
        //    exist on disk.
        // 2. If an already-built version of the target is available online, push to `dependencies`
        //    a `FetchOutput` job for fetching the built package from the network.

        let download_sources = {
            let mut jobs = Vec::with_capacity(manifest.sources().count());
            for src in manifest.sources() {
                let context = self.context.clone();
                let target = self.closure.target().clone();
                let source = src.clone();
                let progress_tx = self.progress.0.clone();
                jobs.push(FetchSource::new(context, target, source).into_job(progress_tx));
            }

            let downloading = future::join_all(jobs).map(|_| ());
            BuildFuture::new(downloading)
        };

        let mut dependencies = Vec::with_capacity(manifest.dependencies().count() + 1);
        dependencies.push(download_sources);

        SourcesFetched {
            context: self.context,
            closure: self.closure,
            graph: self.graph,
            progress: self.progress,
            dependencies,
        }
    }
}

/// Package builder with the sources fetched.
#[derive(Debug)]
pub struct SourcesFetched {
    context: Context,
    closure: Closure,
    graph: BuildGraph,
    progress: Channel,
    dependencies: Vec<BuildFuture>,
}

impl SourcesFetched {
    /// Recursively traverses the build graph and builds all package dependencies.
    pub fn build_dependencies(mut self) -> DependenciesBuilt {
        println!("Building deps for {}", self.closure.target());

        let dependencies = self.closure.dependent_closures();

        for dep_closure in dependencies {
            let context = self.context.clone();
            let builder = Builder::new_recursive(context, dep_closure, self.graph, self.progress);
            let sources_done = builder.fetch_sources();
            let dependencies_done = sources_done.build_dependencies();
            let (built, graph, progress) = dependencies_done.build_package_recursively();

            self.dependencies.push(built);
            self.graph = graph;
            self.progress = progress;
        }

        DependenciesBuilt {
            context: self.context,
            closure: self.closure,
            graph: self.graph,
            progress: self.progress,
            dependencies: Some(self.dependencies),
        }
    }
}

/// Package builder with all sources fetched and dependencies already built.
#[derive(Debug)]
pub struct DependenciesBuilt {
    context: Context,
    closure: Closure,
    graph: BuildGraph,
    progress: Channel,
    dependencies: Option<Vec<BuildFuture>>,
}

impl DependenciesBuilt {
    /// Computes a final `BuildStream` which drives the builder to completion and reports the
    /// progress for each job.
    pub fn build_package(mut self) -> BuildStream {
        let future = self.build_package_impl();
        let packages_built = self.graph.keys().into_iter().cloned().collect();
        let (_, rx) = self.progress;
        BuildStream::new(future, packages_built, rx)
    }

    /// Builds the package itself, returning it along with the modified `BuildGraph` and progress
    /// channels.
    ///
    /// This method is only called internally, used when recursively building dependencies.
    fn build_package_recursively(mut self) -> (BuildFuture, BuildGraph, Channel) {
        let future = self.build_package_impl();
        let graph = self.graph;
        let progress = self.progress;
        (future, graph, progress)
    }

    /// Returns a `BuildFuture` which waits for all dependencies to be built before building itself.
    ///
    /// Using the internal `BuildGraph`, package builds are memoized, allowing us to lazily link
    /// `BuildFuture`s together into a directed acyclic graph which can be executed on a `tokio`
    /// runtime.
    fn build_package_impl(&mut self) -> BuildFuture {
        match self.graph.get(&self.closure.target()).cloned() {
            Some(node) => return node,
            None => {
                let context = self.context.clone();
                let manifest = self.closure.target_manifest().clone();
                let progress_tx = self.progress.0.clone();
                let deps = self.dependencies.take().expect("dependencies empty");

                let building = future::join_all(deps)
                    .map_err(|_| ())
                    .and_then(move |_| BuildManifest::new(context, manifest).into_job(progress_tx));

                let future = BuildFuture::new(building);
                self.graph.insert(self.closure.target().clone(), future);
                self.graph[self.closure.target()].clone()
            }
        }
    }
}

/// A self-contained node in a build graph.
///
/// This future can represent two possible units of work a builder can perform: downloading package
/// sources, and building packages.
///
/// `BuildFuture`s do not resolve to any particular output values on their own. They are only used
/// to execute work on the threadpool and enforce the ordering of build tasks. The current progress
/// of all `BuildFuture`s in a build graph is aggregated in a single `BuildStream`, which is
/// returned by a `Job`.
///
/// This future is intentionally made `Clone` and is safe to poll from multiple threads.
#[derive(Clone)]
#[must_use = "futures do nothing unless polled"]
pub(crate) struct BuildFuture(future::Shared<Box<dyn Future<Item = (), Error = ()> + Send>>);

impl BuildFuture {
    pub fn new<F: Future<Item = (), Error = ()> + Send + 'static>(inner: F) -> Self {
        let future: Box<dyn Future<Item = _, Error = _> + Send> = Box::new(inner);
        BuildFuture(future.shared())
    }
}

impl Debug for BuildFuture {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.debug_tuple(stringify!(BuildFuture))
            .field(&"future::Shared<Box<dyn Future<Item = (), Error = ()> + Send>>")
            .finish()
    }
}

impl Future for BuildFuture {
    type Item = future::SharedItem<()>;
    type Error = future::SharedError<()>;

    #[inline]
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.0.poll()
    }
}

/// Drives the builder to completion and reports the progress of each job in a stream.
#[must_use = "streams do nothing unless polled"]
pub struct BuildStream {
    inner: Box<dyn Stream<Item = Progress, Error = ()> + Send>,
    packages: BTreeSet<ManifestId>,
}

impl BuildStream {
    /// Creates a new `BuildStream`.
    ///
    /// Requires a `BuildFuture` which represents the entire build graph, a list of package IDs
    /// being built, and the receiving half of the `ProgressReceiver` used to report progress.
    pub(crate) fn new(
        future: BuildFuture,
        pkgs: BTreeSet<ManifestId>,
        rx: ProgressReceiver,
    ) -> Self {
        let building = future::lazy(move || {
            tokio::spawn(future.map_err(|_| ()).map(|_| ()));

            Ok(rx.then(|result| match result {
                Ok(Ok(val)) => Ok(val),
                Ok(Err(e)) => Err(e),
                Err(e) => Err(e),
            }))
        });

        let progress = building.flatten_stream();
        BuildStream {
            inner: Box::new(progress),
            packages: pkgs,
        }
    }

    /// Returns a topologically sorted list of packages being built.
    #[inline]
    pub fn packages_affected(&self) -> impl Iterator<Item = &ManifestId> {
        self.packages.iter()
    }
}

impl Debug for BuildStream {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.debug_struct(stringify!(BuildStream))
            .field(
                "inner",
                &"Box<dyn Stream<Item = Progress, Error = ()> + Send>",
            )
            .field("packages", &self.packages)
            .finish()
    }
}

impl Stream for BuildStream {
    type Item = Progress;
    type Error = ();

    #[inline]
    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.inner.poll()
    }
}
