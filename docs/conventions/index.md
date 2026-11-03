# Conventions

Commit and branch naming, how they're enforced, and versioning.

* [Enforcing commits and branches](commits-check.md) - the pre-commit hook
  and `make commits-check` that enforce
  [Conventional Commits](https://www.conventionalcommits.org/) and
  [Conventional Branch](https://conventionalbranch.org/).
* [Versioning, tags and releases](versioning.md) - SemVer bump criteria, which
  bumps get a tag, and on-demand releases.
* [Trusted Publishing](trusted-publishing.md) - how a GitHub Release uploads
  the wheel to PyPI and the crate to crates.io without a stored token.
* [Help-line check](help-check.md) - `make help-check` fails on a Makefile
  target without its `##` help line.
* [No CLAUDE.md check](claude-md-check.md) - `make claude-md-check` fails when
  a `CLAUDE.md` exists, since `AGENTS.md` is the only agent file.
* [Tag vocabulary](tag-vocabulary.md) - the tags the notes carry and what
  each one means.
