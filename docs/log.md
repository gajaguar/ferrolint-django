# Directory Update Log

## 2026-11-03

* **Change**: CI scopes the workflow token to `contents: read`, caches the
  cargo build (saved from `main` only) and the `cargo-deny` binary, installs
  the pinned `cargo-deny` before `make check`, and skips the rust hooks that
  `make check` already runs.

## 2026-11-01

* **Initialization**: Initial scaffold (rust).
