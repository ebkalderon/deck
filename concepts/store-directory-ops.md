# Store directory operations

```
trait Fetchable {
    type Progress;
}

trait Directory {
    type Id;
    type Data;
    type Fetch: Fetchable;

    // Checks whether the ID exists.
    // If so, returns some data that the builder might want.
    fn read(id: &Self::Id) -> impl Future<Item = Option<PathBuf>, Error = ()>;

    // Writes the provided data directly to disk.
    // Once the data is written, a new ID is computed.
    // If the data already exists on disk, the future ends immediately.
    fn write(data: Self::Data) -> impl Future<Item = (Self::Id, PathBuf), Error = ()>;

    // Fetches the data from a remote source and writes it to the store.
    // Returns a stream of items marking progress.
    // If the data already exists on disk, the stream ends immediately.
    fn fetch(fetch: Self::Fetch) -> impl Stream<Item = (Self::Id, <Self::Fetch as Fetch>::Progress), Error = ()>);
}
```

For example, in the `OutputsDir` directory:

```
impl Fetchable for OutputFetch {
    type Progress = hyper::Chunk;
}

impl Directory for OutputsDir {
    type Id = PackageId;
    type Data = PathToBuildDirectory;
    type Fetch = OutputFetch;
}
```
