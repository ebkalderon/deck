# Managing mutable state on a filesystem

Let's say there are two simultaneous builds which happen to fetch sources which
have the same names and hashes. In this case, we want to ensure the following
properties hold true:

1. Reads and writes do not occur simultaneously with one another.
2. Sources are downloaded exactly once.

To that end, we have several mechanisms to accomplish this.

For point number 1, we have a very ergonomic native OS feature called `flock()`
which allows for _files_ to be locked atomically, either shared or exclusively.
The filesystem will automatically handle the correct synchronization between
readers and writers. Unfortunately, (a) this feature cannot protect
directories, only files, and (b) this protects _existing_ files from being
corrupted, but it cannot protect files are in the process of being created, e.g.
there is no way to use this to indicate to a `JobFuture` that a Git repo is
being downloaded.

This brings us to the second point: if two simultaneous `JobFuture`s request the
same exact sources (bit-perfectly identical), there is no need to download the
two concurrently. Instead, the `State` should download the file for one and
block the other eventually unblocking both simultaneously when the download
completes.

## Handling concurrent downloads

1. Build 1 and Build 2 are running simultaneously and request to download the
   same source.
2. Build 1's request happens to get processed first.
3. The requested ID for the download gets added to a queue, with a future
   attached to it which resolves once the ID is removed. The download proceeds
   as usual.
   * If the download is a file, the file is locked exclusively using `flock()`
     to ensure no concurrent access while the data is being streamed to disk.
     Once the file is downloaded, it is validated against the provided hash,
     set to be read-only, the file is unlocked, and a new ID is computed from
     the hash and the file name and returned.
   * If the download is a Git repository, there is no explicit synchronization
     with `flock()` as we rely on the synchronization provided natively by
     `libgit2`. Once the repository has been downloaded and the correct branch
     is checked out, the repo is set to be read-only, we append the commit ref
     to the repository name to form the ID, and then return it.
4. Build 2's request gets processed second. The future created earlier is
   returned, blocking until completion. Once it resolves, no download takes
   place and instead the ID of the on-disk file is returned immediately.
5. Any subsequent downloads of this source are memoized and the ID of the
   on-disk data is taken from the `Path` and immediately returned.

## Dealing with concurrent reads and writes

Several factors of the design of the Deck store prevent this from being a
problem:

1. The contents of the store are immutable unless you are doing an initial
   write. Once written, the files are set read-only and cannot be mutated in
   place, making this a non-issue. Files are only mutable once copied to the
   `tmp` directory in the store, which is exclusively owned by a builder and
   does not allow for concurrent access.
2. Restricting access to the file system through the `State` struct and
   funneling directory accesses through that `State` using unique IDs prevents
   files from being mutated at random.
