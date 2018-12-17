# Crate layout

```
+ deck-store/
  + migrations/
  + src/
    + binary_cache/
      + backends/
        + https.rs
        + local.rs
        + mod.rs
        + s3.rs
        + ssh.rs
      + id.rs
      + mod.rs
    + package/
      + id.rs
      + manifest.rs
      + mod.rs
    + store/
      + backends/
        + local.rs
        + mod.rs
        + ssh.rs
      + fs/
        + closure.rs
        + db.rs
        + fetch_git.rs
        + fetch_uri.rs
        + mod.rs
      + job/
        + build_packages.rs
        + fetch_sources.rs
        + mod.rs
        + progress.rs
      + builder.rs
      + closure.rs
      + context.rs
      + id.rs
      + mod.rs
    + lib.rs
    + platform.rs
```
