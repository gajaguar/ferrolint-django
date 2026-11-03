---
type: decision
title: Releases use Trusted Publishing
description: Publishing a GitHub Release uploads the wheel to PyPI and the crate to crates.io through OIDC, with no stored token.
tags: [release, ci]
status: stable
sources:
  - id: pypi-trusted-publishers
    resource: https://docs.pypi.org/trusted-publishers/
    title: PyPI trusted publishers
    author: org:pypi
  - id: crates-trusted-publishing
    resource: https://crates.io/docs/trusted-publishing
    title: crates.io trusted publishing
    author: org:crates.io
---

# Releases use Trusted Publishing

Publishing a GitHub Release (the `published` event) runs
[`.github/workflows/publish.yml`](../../.github/workflows/publish.yml). It
builds the wheel with `make wheel`, imports it in a clean environment, then
publishes it to PyPI under the `pypi` environment and the crate to crates.io
under the `crates-io` environment. Both jobs authenticate with a short-lived
OIDC token, so the repository holds no PyPI or crates.io secret. The build
fails when the release tag, `Cargo.toml` and `pyproject.toml` disagree on the
version.

The wheel is `py3-none-any` because it only carries `plugin.wasm`, so one
runner builds it and there is no sdist: an sdist would not contain the
component, which `make plugin` builds.

## One-time setup

- **PyPI:** add a trusted publisher for `ferrolint-django` (a "pending
  publisher" before the first upload) with owner `gajaguar`, repository
  `ferrolint-django`, workflow `publish.yml` and environment `pypi`.
- **crates.io:** trusted publishing needs the crate to exist, so publish the
  first release by hand with `cargo publish`, then add the same repository,
  workflow `publish.yml` and environment `crates-io` as a trusted publisher.
  Until then the `crates` job fails and PyPI is unaffected.
- **GitHub:** create the `pypi` and `crates-io` environments, restrict them to
  `v*` tags and require a reviewer, so nothing uploads without approval.

## Cutting a release

[Versioning, tags and releases](versioning.md) decides the bump and the tag.

1. Bump `version` in `Cargo.toml` and `pyproject.toml` together and merge.
2. Run `make release-tag` from the updated base branch.
3. Publish a GitHub Release from that tag and approve the deployments.
