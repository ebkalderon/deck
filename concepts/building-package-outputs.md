# Building package outputs

This document describes the basic steps necessary for building a package output,
disregarding the implementation details of the store, the client and/or daemon,
and so on.

1. Receive the target manifest from the requester.
2. Compute the closure of the target manifest recursively.
   1. If the manifest does not exist in the store, attempt to fetch it from a
      repository, and then a remote store. If this fails, bork. If it already
      exists in the store, do nothing.
   2. If the outputs of the target manifest already exist, stop traversing
      further. The package is already installed, and we are done.
   3. Check the build outputs of the target manifest, looking at their
      references. If a given output with a compatible `equivalence-id` already
      exists in the store, do nothing. If not, attempt to fetch it from a binary
      cache, and then a remote store. If all outputs are fetched successfully,
      stop traversing further. Otherwise, continue to step 4.
   4. For each source, check if it exists in the store. If it does, insert a
      job which returns immediately. If it does not, insert a job which fetches
      it and saves it in the store.
   5. For each item in `dependencies` of the target manifest, recurse into it
      using `dependent_closures()` and run the same process against it. If
      tests are enabled for the target, include both the `dependencies` and
      `dev-dependencies` in the closure.
5. Once the closure has been computed, recursively read the closure to
   construct a build graph from futures. See the document
   [scheduling-work.md](./scheduling-work.md) for more details on how this is
   done.
   1. Request the `StoreDir` to create a new temporary directory for you. This
      will be used as scratch space for the builder to work in. The method will
      accept a stream of progress that can be processed indefinitely until the
      stream terminates in success or in error. If the stream terminates in
      success, the directory permissions are normalized and registered in the
      database, and the directory is atomically renamed to its final location
      in the store. If the stream terminates in error, the directory is deleted
      and the build process is aborted.
   2. TODO
