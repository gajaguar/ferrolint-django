# Directory Update Log

## 2026-11-03

* **Change**: version `1.0.0`, the release that declares the plugin contract
  stable. `trusted-publishing.md` describes the `publish.yml` workflow that
  uploads a GitHub Release to PyPI and crates.io, `README.md` gains the
  badges and contents list of the template, and the `re2js` advisory is
  fixed with a pnpm override.

* **Change**: `SECURITY.md` and a README `## Security` section say how to
  report a vulnerability privately, as `ferrolint` does.

* **Change**: `ferrolint-plugin-sdk` is the crates.io `=1.1.0` release, no
  longer a path dependency on `../ferrolint`; the tests still run the sibling
  checkout's `ferrolint` binary.

* **Change**: `README.md` follows the template's section order with the
  plugin's own About, Key features, Requirements, Usage, a Rules table and
  Open items, and `CONTRIBUTING.md` documents the sibling `../ferrolint`
  checkout, `make plugin` and `make wheel`, and how to add a rule.
* **Change**: CI scopes the workflow token to `contents: read`, caches the
  cargo build (saved from `main` only) and the `cargo-deny` binary, installs
  the pinned `cargo-deny` before `make check`, and skips the rust hooks that
  `make check` already runs.
* **Change**: the tests run the `ferrolint` binary from `FERROLINT`, else the
  sibling checkout's build, else the one on `PATH`; CI installs
  `ferrolint==1.1.0` from PyPI, so it no longer needs `../ferrolint`.

## 2026-11-01

* **Initialization**: Initial scaffold (rust).
