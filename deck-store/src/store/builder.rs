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

pub use self::futures::BuildStream;

use std::collections::BTreeMap;

use futures_preview::future::{self, TryFutureExt};
use futures_preview::stream;

use self::futures::{BuildFuture, BuilderState, InnerFuture, JobFuture};
use self::job::{BuildManifest, FetchSource, IntoJob};
use super::context::Context;
use super::progress::{self, ProgressReceiver, ProgressSender};
use crate::id::ManifestId;
use crate::package::Manifest;

mod futures;
mod job;

type BuildGraph = BTreeMap<ManifestId, BuildFuture>;

/// Asynchronous builder for a set of packages.
#[derive(Debug)]
pub struct Builder {
    context: Context,
    package: ManifestId,
    graph: BuildGraph,
    progress: (ProgressSender, Option<ProgressReceiver>),
}

impl Builder {
    /// Creates a new `Builder` which will build `package` using data from the given `Context`.
    pub fn new(context: Context, package: ManifestId) -> Self {
        let (tx, rx) = progress::progress_channel(4);
        Builder {
            context,
            package,
            graph: BTreeMap::new(),
            progress: (tx, Some(rx)),
        }
    }

    /// Same as `Builder::new()`, except it lets you specify a pre-populated `BuildGraph` and a
    /// progress channel.
    ///
    /// This constructor is only called internally, used when recursively building dependencies.
    #[inline]
    fn new_recursive(ctx: Context, pkg: ManifestId, graph: BuildGraph, tx: ProgressSender) -> Self {
        Builder {
            context: ctx,
            package: pkg,
            graph,
            progress: (tx, None),
        }
    }

    /// Loads and parses the package manifest.
    ///
    /// If the manifest does not exist in the store, the builder will attempt to fetch it from a
    /// remote source, if one exists.
    pub fn load_manifest(self) -> ManifestLoaded {
        let context = self.context;
        let manifest_id = self.package;
        let graph = self.graph;
        let (tx, rx) = self.progress;

        let future = async {
            // TODO: Implementation needed.
            // let manifest = await!(context.store.load_manifest(&manifest_id))?;
            let manifest =
                Manifest::build("foo", "1.0.0", "fc3j3vub6kodu4jtfoakfs5xhumqi62m", None)
                    .finish()
                    .unwrap();

            Ok(BuilderState {
                context,
                manifest,
                manifest_id,
                graph,
                progress: tx,
                dependencies: Vec::new(),
            })
        };

        ManifestLoaded {
            inner: InnerFuture::new(future),
            progress: rx,
        }
    }
}

/// Package builder with the target manifest loaded.
#[derive(Debug)]
pub struct ManifestLoaded {
    inner: InnerFuture,
    progress: Option<ProgressReceiver>,
}

impl ManifestLoaded {
    /// Attempts to substitute this build with a pre-built package, if one is available.
    pub fn try_substitute(self) -> MaybeSubstituted {
        let inner = self.inner;

        let future = async {
            let mut builder = await!(inner)?;
            let progress = builder.progress.clone();

            // TODO: Implementation needed.
            // let package_installed = builder
            //     .manifest
            //     .outputs()
            //     .all(|id| builder.context.output_exists(&id));
            let package_installed = true;

            if package_installed {
                // package already installed on disk.
                let job = JobFuture::new(stream::once(future::err(())), progress);
                let memoized = BuildFuture::new(job);
                builder.graph.insert(builder.manifest_id.clone(), memoized);
            // TODO: Implementation needed.
            // } else if await!(builder.context.substitutes_available(builder.manifest.outputs()))? {
            } else if await!(future::ok(false))? {
                // substituted outputs.
                let job = JobFuture::new(stream::once(future::err(())), progress);
                let fetched = BuildFuture::new(job);
                builder.graph.insert(builder.manifest_id.clone(), fetched);
            }

            // building
            Ok(builder)
        };

        MaybeSubstituted {
            inner: InnerFuture::new(future),
            progress: self.progress,
        }
    }
}

/// Package builder with all pre-built packages fetched, if any are available.
#[derive(Debug)]
pub struct MaybeSubstituted {
    inner: InnerFuture,
    progress: Option<ProgressReceiver>,
}

impl MaybeSubstituted {
    /// Fetches all sources required to build this package.
    pub fn fetch_sources(self) -> SourcesFetched {
        let inner = self.inner;

        let future = async move {
            let mut builder = await!(inner)?;

            if !builder.graph.contains_key(&builder.manifest_id) {
                let mut jobs = Vec::with_capacity(builder.manifest.sources().count());
                for src in builder.manifest.sources() {
                    let context = builder.context.clone();
                    let target = builder.manifest_id.clone();
                    let source = src.clone();
                    let progress = builder.progress.clone();
                    jobs.push(
                        future::ok(FetchSource::new(context, target, source)).into_job(progress),
                    );
                }

                let download_sources = BuildFuture::join_all(jobs);
                builder
                    .dependencies
                    .reserve(builder.manifest.dependencies().count() + 1);
                builder.dependencies.push(download_sources);
            }

            Ok(builder)
        };

        SourcesFetched {
            inner: InnerFuture::new(future),
            progress: self.progress,
        }
    }
}

/// Package builder with all sources fetched.
#[derive(Debug)]
pub struct SourcesFetched {
    inner: InnerFuture,
    progress: Option<ProgressReceiver>,
}

impl SourcesFetched {
    /// Recursively traverses the build graph and builds all package dependencies.
    pub fn build_dependencies(self) -> DependenciesBuilt {
        let inner = self.inner;

        let future = async {
            let mut builder = await!(inner)?;
            println!("Building deps for {}", builder.manifest_id);

            let dependencies = builder.manifest.dependencies();

            for dep in dependencies.cloned() {
                let context = builder.context.clone();
                let progress = builder.progress.clone();

                let child = Builder::new_recursive(context, dep, builder.graph, progress);
                let loaded = child.load_manifest();
                let maybe_sub = loaded.try_substitute();
                let sources_done = maybe_sub.fetch_sources();
                let deps_done = sources_done.build_dependencies();
                let (built, graph) = await!(deps_done.build_package_recursively())?;

                builder.dependencies.push(built);
                builder.graph = graph;
            }

            Ok(builder)
        };

        DependenciesBuilt {
            inner: InnerFuture::new(future),
            progress: self.progress,
        }
    }
}

/// Package builder with all sources fetched and dependencies already built.
#[derive(Debug)]
pub struct DependenciesBuilt {
    inner: InnerFuture,
    progress: Option<ProgressReceiver>,
}

impl DependenciesBuilt {
    /// Computes a final `BuildStream` which drives the builder to completion and reports the
    /// progress for each job.
    pub fn build_package(mut self) -> BuildStream {
        let progress = self.progress.take().unwrap();
        let built = self.build_package_recursively().map_ok(|(built, _)| built);
        BuildStream::new(built, progress)
    }

    /// Builds the package itself, returning it along with the modified `BuildGraph`.
    ///
    /// Using the internal `BuildGraph`, package builds are memoized, allowing us to lazily link
    /// `BuildFuture`s together into a directed acyclic graph which can be executed on a `tokio`
    /// runtime.
    ///
    /// This method is only called internally, used when recursively building dependencies.
    async fn build_package_recursively(self) -> Result<(BuildFuture, BuildGraph), ()> {
        let mut builder = await!(self.inner)?;

        match builder.graph.get(&builder.manifest_id).cloned() {
            Some(node) => Ok((node, builder.graph)),
            None => {
                let context = builder.context.clone();
                let manifest = builder.manifest.clone();
                let progress = builder.progress.clone();
                let dependencies = builder.dependencies;

                let job = future::ok(BuildManifest::new(context, manifest)).into_job(progress);
                let building = BuildFuture::join_all_and_then(dependencies, job);
                builder.graph.insert(builder.manifest_id.clone(), building);
                let node = builder.graph[&builder.manifest_id].clone();

                Ok((node, builder.graph))
            }
        }
    }
}
