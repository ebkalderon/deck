# Musings

Sorted in chronological order, from oldest musing to newest.

## 1. Configuration features

Some nice things to have:

Option       | Description
-------------|----------------------------------------------------------
`--airplane` | Disable downloads for all remote dependencies, local only

## 2. Semantic intent vs. reproducibility

Semantic versioning is a fantastic way to indicate the author's intent with a
release in a friendly, human-readable, chronologically ordered way. It also
allows you to react to upstream version changes quickly and mix and match
versions easily. By contrast, in Nixpkgs, even though Nix itself allows for
multiple versions of a package to be installed without conflicts, there is
usually only one version of a package available in Nixpkgs at a time. The entire
repository is identified by a particular commit hash and relies on CI to ensure
relationships between packages don't break as individual expressions are updated
piecemeal by contributors. Although this approach works well for
reproducibility, it is quite limiting when you find you need to mix and match
versions. If you need to depend on a version of a package from Nixpkgs in commit
1234567 and have a transitive dependency on an older version of the same package
only available in commit 8901234, you're pretty screwed. You will need to either
go to the Nixpkgs repo on GitHub, search the commit history and `git blame` info
until you find the version you want, view it, download the expression, and
finally build it locally to satisfy that other derivation's obscure dependency.
Nix tries to get around this by retaining some old versions of packages in the
repo with the format `foo_1_2_3`, `foo_2_0_1`, etc. but these aren't guaranteed
to remain in the repo forever, and it's unnecessarily cumbersome to deal with,
IMO. Compare this with Cargo's approach of retaining an infinite immutable log
of always accessible older versions of every package ever, tagged with semantic
versions. If a transitive dependency has a hard requirement an older version of
a package, it can be queried and fetched, no questions asked.

The biggest issue with semantic versioning is the lack of provenance associated
with a package version, namely, when a package declares a dependency on package
`foo-1.2.3`, how can you guarantee that the dependency you're building against
is exactly what you want? For example, package `foo-1.2.3` compiled for Windows
with HTTP/2 `curl` support enabled and package `foo-1.2.3` compiled for Linux
with HTTP/2 `curl` support disabled have the same semantic version, yet if you
try to link against both, you will crash and burn because despite having
compatible APIs, they are fundamentally _not the same_. Another issue with
semantic versioning is that floating version ranges suck for reproducibility.
With these floating dependency version ranges coupled with the lack of
provenance, there is no way to ensure reproducibility of a build of `foo-1.2.3`
since it can change from build to build. Having lockfiles with unique hashes for
the entire dependency graph can help mitigate the problem somewhat by recording
provenance for the build, but it's incredibly difficult to manage that from the
point of view of an operating system, a far more complex beast to deal with than
just a downstream application. This is what Spack tries to do, and it uses some
very complex constraint solving in its process of concretization. On the other
hand, Nixpkgs actually has a huge advantage in this regard, with its monolithic
repo. By default, Nix channels are free-floating, and as such, NixOS builds with
the same configuration will mutate over time as they build against different
versions of Nixpkgs. But if you pin your local copy of Nixpkgs to a particular
commit hash and eliminate any substituters or alternate package sources, you are
pretty much guaranteed to get the exact same NixOS image every single time you
rebuild. Dynamic and flexible version specifications based on semantic
versioning add additional complexity to manage. Let's imagine there was some
alternate OS like NixOS but using baked-in semantic versioning in addition to
build hashes called "SpackOS." This hypothetical OS has a monolithic package
repo similar to Nixpkgs call Spackpkgs, except it permits multiple versions of
packages and allows those packages to declare semantic versioned dependencies
potentially with version ranges. Let's say you make an initial build of SpackOS
with Spackpkgs commit 1234567 and you receive an OS image with hash 1212121.
Let's say you update Spackpkgs to commit 8901234, bumping some package versions
in the process. You rebuild your SpackOS against this new commit and get an OS
image with the hash 3232323. Even if you pin your Spackpkgs back to commit
1234567890, some of the packages on your machine might declare version ranges
and will end up unintentionally linking to packages with newer versions that
were cached in your Spack store from before, meaning you will not get an OS
image with the hash 1212121, as you might have expected. In other words, you
will have gotten a different output for the same set of inputs. Reproducibility
has been broken.

With Nix, reproducibility is the key goal. Anything which in its core design
compromises reproducibility is unworthy of consideration. This is why package
versions in Nix are little more than simple strings which only provide human
readable semantic intent, but do not influence the package manager's behavior in
any way.

## 3. Store Design

There are two types of stores: _local_ and _remote_. The local store is trusted
and preferred over any remote store, when one can help it. Access to a store is
restricted by whitelisting a few unique uids and groups and making the entire
store read-only until mutation is necessary.

In Nix, a store contains the following kinds of objects:

Object               | Description
---------------------|------------------------------------------------------------------------
Sources              | Fixed-hash files, e.g. downloaded archives, Git clones, builder scripts
Derivation manifests | ATerm manifest files (essentially a declarative build recipe)
Derivation outputs   | Directories of artifacts produced by derivation manifests

In the current store model, the _extensional model_, sources and derivation
manifests are content-addressable (meaning their store hash is based entirely on
their on-disk contents), but derivation outputs are not because their output
paths are computed ahead of time, before they are built, from the input
derivation and _not_ from the actual contents produced by the builder (Dolstra,
141).

> _Aside:_ Why was this done? Because it corresponds to the fundamental pure
> functional design of Nix. Derivations are supposed to be pure functions, where
> varying inputs produce varying output and old computations can be memoized for
> efficiency. And they indeed are, except for a few anti-features which break
> this assumption ("Breaking the trust property", Dolstra, 138):
>
> 1. Users with root access can rebind the Nix store as read/write and tamper
>    with it, and the Nix daemon will happily build derivations based on this
>    corrupted state, regardless of what the Nix expressions or .drv files
>    describe, because it relies on the hashes in the store paths instead of the
>    actual hashes of the directories' contents.
> 2. It is trivial to write builders that tamper with the sources or outputs of
>    other derivations. This can be solved by enabling sandboxed builds, but
>    some poorly written packages break when this is turned on and is not
>    supported on all platforms and environments, so this is usually turned off.
> 3. Substitutes in particular completely break the integrity of the
>    `fn(&[Input]) -> Output` mapping used in Nix. You are allowing users to
>    take an existing derivation and replace it with another arbitrary one, with
>    different inputs, different on-disk size when installed, and even a
>    different on-disk hash and calling the two the same! The extensional model
>    relies only on the store paths for checking equality and doesn't verify
>    that their outputs actually do. This means that if one user can be tricked
>    into pulling in a malicious substitute for a package, the cache is now
>    tainted for all users who make subsequent builds depending on that
>    derivation. After all, it exists on the local cache, so it must be valid
>    and is preferred over any remote sources. Although the previous two issues
>    have security measures in place (e.g. the Nix daemon, sandboxed builds) to
>    ensure day-to-day integrity, this one doesn't really have any. You could
>    sign every single substitute with PGP keys and enforce verification of
>    signatures every time you substitute one, but this is painful, complex, and
>    difficult to do for non-official packages. No, the introduction of
>    substitutes with the current model fundamentally breaks the trust property
>    necessary for Nix's purity guarantees to hold.
>
> Suddenly, given the revelations above (especially that terrifying third one),
> hashing derivation outputs based on their actual contents rather than their
> inputs makes a lot more sense.

Making all the objects in the store content-addressable, including hashing the
derivation outputs based on their contents rather than their inputs, and
verifying substitutes' output hashes match before adding them to the store,
constitutes the _intensional_ store model. There have been ongoing efforts to
migrate Nix to use the intensional model, but progress has been slow and
difficult due to backwards compatibility concerns, as well as difficulties in
implementing support for self-references in the current code base. It is clear
that if Deck is mimicking Nix's design in many regards, it should adopt the
intensional store model right off the bat.

Therefore, Deck's store model is as follows:

### File system layout

```
/
|
+-- deck/
    |
    +-- store/
        |
        +-- sources/ <------------------------ Downloaded sources are cached
        |   |                                  here for future reuse. Git repos
        |   +-- 134b23c3f4-hello-2.11.0.tar.gz are considered sources and are
        |   |                                  cached here as well.
        |   +-- eb87d98da7-builder.sh
        |   |
        |   +-- ...
        |
        +-- derivations/ <-------------------- Contains the derivation manifests
        |   |                                  which produce future outputs.
        |   +-- 34b9f23cee-hello-2.11.0.toml   TODO: Does this need to exist for
        |   |                                  any reason other than OCD? These
        |   +-- ...                            could just as well live alongside
        |                                      their respective output dirs.
        |
        +-- tmp/ <---------------------------- Uses a random unique hash, not
        |   |                                  based on content. Directory gets
        |   +-- 32th8bc09f-hello-2.11.0/       deleted once output(s) are hashed
        |       |                              and copied to outputs/ under new
        |       +-- ...                        content-addressible name. Both
        |                                      local builds and substitute
        |                                      verifications occur in here.
        |
        +-- outputs/ <------------------------ Contains the actual derivation
            |                                  outputs with build artifacts.
            +-- 83a0e3bf5d-hello-2.11.0/
                |
                +-- bin/
                |   |
                |   +-- hello
                |
                +-- etc/
                    |
                    +-- default.conf
```

Why is the Deck store chunked into subdirectories like this rather than
remaining a monolithic structure like a Nix store?

* **Ease of understanding:** It is clear how each of the files/folders will be
  used by the Deck daemon by compartmentalizing them like this. Each hash can be
  self-describing just by looking at its store path.
* **Security:** This is a very minor benefit (if it can even be called one), but
  splitting up store contents into separate directories like this allows
  permissions to store resources to be chunked by directory. For example, the
  temporary build directories are not supposed to be interfered with during
  other derivation builds, but in Nix, nothing is stopping users from pointing
  one derivation at the temporary directory of another and reading in the
  contents (unless sandboxing is turned on, but that's not always supported).
  In this case, the `tmp` subdirectory can be blanket quarantined such that
  paths pointing to `tmp` directories that aren't your own are disallowed.
  ~~Additionally, in Nix, the `/nix/store` directory is writeable by all workers
  in the `nixbld` group, which introduces issues with shared mutable state. By
  compartmentalizing the Nix store like this, one can better enforce mutability
  and make accidental corruptions more unlikely by setting the `derivations` or
  `sources` directories to be entirely read-only, for example, until you
  absolutely have to write new data to it.~~ This doesn't actually solve that
  problem any better than granular file-level permissions do, which need to
  exist anyway. I'd say that the main benefit is as above: enforcing types on
  the build data such that elements in the Nix store can't be abused and their
  purpose is clearly defined.

One might imagine that this might make looking up the content addressable name
of a Deck store object harder, and they'd probably be right. However, it is
quite trivial to add in the notion of these subdirectories and map them to their
correct locations in the SQLite3 database and in the Deck application logic.

### Hashing

Object               | Hash Input
---------------------|-------------------------------
Source               | "\<multihash-of-source\>:\<file-name\>"
Derivation manifest  | "\<multihash-of-file\>:\<file-name\>"
Temp local build dir | "\<randomly-generated-hash\>:\<build-user-name\>"
Derivation output    | "\<multihash-of-
