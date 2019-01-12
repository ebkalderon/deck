# Store directory operations

```rust
enum State<P, D> {
    Progress(P),
    Done(D),
}

trait Fetchable: Send {
    type Progress;

    fn fetch(self, temp_target: &Path) -> impl Stream<Item = Self::Progress, Error = ()>;
}

trait Directory {
    type Id;
    type Input;
    type Progress;
    type Output;

    // Computes a new temporary ID from the given write input.
    // This ID might change later after the data is written to disk.
    fn compute_id(&self, data: &Self::Input) -> impl Future<Item = Self::Id, Error = ()>;

    // Checks whether the ID exists.
    // If so, returns some data that the builder might want.
    fn read(&self, target: &Path, id: &Self::Id) -> impl Future<Item = Option<Self::Output>, Error = ()>;

    // Writes the provided data directly to disk.
    // Once the data is written, a new ID is computed.
    // If the data already exists on disk, the future ends immediately.
    fn write(&self, target: &Path, data: Self::Input) -> impl Stream<Item = State<Self::Progress, Self::Id>, Error = ()>;
}
```

For example, given the fetchers below:

```rust
struct FetchUri {
    uri: Uri,
    hash: Hash,
}

impl Fetchable for FetchUri {
    type Progress = hyper::Chunk;

    fn fetch(self, temp_target: &Path) -> impl Stream<Item = Self::Progress, Error = ()> { ... }
}

struct FetchGit {
    uri: Uri,
    version: RevOrBranch,
}

impl Fetchable for FetchGit {
    type Progress = git2::Progress<'static>;

    fn fetch(self, temp_target: &Path) -> impl Stream<Item = Self::Progress, Error = ()> { ... }
}
```

And the `Fetcher` wrapper specified below:

```rust
pub struct Fetcher<F> { ... }

impl<F: Fetchable> Fetcher<F> {
    pub(crate) new(fetcher: F, output: PathBuf) -> Self { ... }

    pub fn fetch<I, F, U>(self, compute_id: F) -> impl Stream<Item = State<F::Progress, I>, Error = ()>
    where
        I: Send + 'static,
        F: Fn(&Path) -> U + Send + 'static,
        U: IntoFuture<Item = I, Error = ()> + Send + 'static,
        U::Future: Send + 'static,
    {
        // Fetch resource and save it to `output` while streaming progress.
        // Map `State::Progress` over each `F::Progress`.
        // Chain to the stream a future which executes `compute_id()`.
        // Map `State::Done` over the final future output.
        // Return the stream.
    }
}
```

The code for `ManifestsDir` is implemented like so:

```rust
trait FetchManifest: Fetchable {}

impl FetchManifest for UriFetch {}

impl<F: FetchManifest> Directory<F> for ManifestsDir {
    type Id = PackageId;
    type Read = Manifest;
    type Write = ManifestOrRawJsonOrPath;

    fn compute_id(&self, data: &Self::Write) -> impl Future<Item = Self::Id, Error = ()> {
        // If `ManifestOrRawJson::Manifest`, turn to raw JSON first.
        // Convert raw JSON to ID and return.
    }

    fn read(&self, path: &Path, id: &Self::Id) -> impl Future<Item = Option<Self::Read>, Error = ()> {
        // Open file at `path` and lock shared.
        // If file exists, convert to Manifest and return Ok(Some(manifest)).
        // If file does not exist, return Ok(None);
        // If file exists but parsing failed, return Err(e).
        // If an IO error occurred, return Err(e).
    }

    fn write(&self, path: &Path, data: Self::Write) -> impl Future<Item = Self::Id, Error = ()> {
        // Create file at `path` and lock exclusively.
        // If `ManifestOrRawJsonOrPath::Manifest`, turn to raw JSON first.
        // Write raw JSON to file.
        // Return same ID given from `path`.
    }
}
```

And the code for `OutputsDir` is implemented like so:

```rust
trait FetchOutput: Fetchable {}

impl FetchOutput for UriFetch {}

impl<F: FetchOutput> Directory<F> for OutputsDir {
    type Id = OutputId;
    type Read = PathToDirectory;
    type Write = OutputArchiveOrTempDirectory;

    fn compute_id(&self, data: &Self::Write) -> impl Future<Item = Self::Id, Error = ()> {
        // If `OutputArchiveOrTempDirectory::Archive`, return output ID from archive name.
        // If `OutputArchiveOrTempDirectory::TempDir`, compute hash of directory without self-references.
    }

    fn read(&self, path: &Path, id: &Self::Id) -> impl Future<Item = Option<Self::Read>, Error = ()> {
        // Check if `path` exists and is a directory.
        // If dir exists, canonicalize it and return Ok(Some(path)).
        // If dir does not exist, return Ok(None).
        // If IO error occurred, return Err(e).
    }

    fn write(&self, path: &Path, data: Self::Write) -> impl Future<Item = Self::Id, Error = ()> {
        // If `OutputArchiveOrTempDirectory::Archive`, extract archive to `path`.
        // If `OutputArchiveOrTempDirectory::TempDir`, create new directory at `path`.
        // If any error occurs or the directory already exists, return Err(e).
        // Return the ID given from `path`.
    }
}
```

And the code for `SourcesDir` is implemented like so:

```rust
trait FetchSource: Fetchable {}

impl FetchSource for UriFetch {}
impl FetchSource for GitFetch {}

impl<F: FetchSource> Directory<F> for SourcesDir {
    type Id = SourceId;
    type Read = PathToSource;
    type Write = ExternalPathOfSource;

    fn compute_id(&self, data: &Self::Write) -> impl Future<Item = Self::Id, Error = ()> {
        // Compute ID from path to the source and generate a random hash.
    }

    fn read(&self, path: &Path, id: &Self::Id) -> impl Future<Item = Option<Self::Read>, Error = ()> {
        // Check if `path` exists.
        // If dir exists, canonicalize it and return Ok(Some(path)).
        // If dir does not exist, return Ok(None).
        // If IO error occurred, return Err(e).
    }

    fn write(&self, path: &Path, data: Self::Write) -> impl Future<Item = Self::Id, Error = ()> {
        // Copy source from external path to `path`.
        // Compute new ID by taking hash of file/directory.
        // Return new ID.
    }
}
```
