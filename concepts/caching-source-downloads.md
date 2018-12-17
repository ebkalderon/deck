# Caching source downloads

When downloading sources for a particular builder, care should be taken that
multiple concurrent builders do not independently redownload files that already
exist in the store. Likewise, workers should be able to recognize when a file
already exists in the store with the same content, but possibly a different
name, and be able to reuse that file accordingly.

The following cases are known:

Names (excluding hash) match? | Hashes match? | Action taken
------------------------------|---------------|--------------------------------------------
Yes                           | Yes           | No action, use memoized copy in cache
Yes                           | No            | Download the file and save it
No                            | Yes           | Duplicate the equivalent file and rename it
No                            | No            | Download the file and save it

Additionally, it should be noted that build nodes requesting downloads should
check which files are in progress before downloading anything. If two packages
are fetching files with the same name and hash simultaneously, one of them
should block until the other completes its download and moves it from `tmp` to
`sources`, and then both build nodes can continue uninterrupted.
