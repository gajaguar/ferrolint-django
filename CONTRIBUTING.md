# Contributing

Thanks for considering a contribution. Bug reports, feature ideas,
documentation fixes, and code are all welcome.

## Ways to contribute

- Report a bug or propose a feature by opening a GitHub issue. For a bug,
  include the steps to reproduce it and what you expected to happen.
- Fix or extend documentation under `docs/`.
- Submit a pull request for a bug fix or a new feature. For anything larger
  than a small fix, open an issue first so the approach can be agreed before
  you invest time in it.

## Set up

Install the toolchain and dependencies as described in the
[README](README.md#requirements), then:

```bash
make install
```

The tests run a `ferrolint` binary against the fixtures in `tests/fixtures/`:
`FERROLINT` if set, else the build of a sibling `../ferrolint` checkout
(`make ferrolint-bin`), else the `ferrolint` on `PATH`
(`uv tool install ferrolint==1.1.0`, as CI does). `make plugin` builds the WASM
component into `python/ferrolint_django/plugin.wasm`, and `make wheel` packs
it into `dist/`.

Run `make help` for the full list of targets.

## Before opening a pull request

Run the gate and the tests; both MUST pass:

```bash
make check
make test
```

`make fix` applies the safe automatic fixes for what `make check` reports.
CI runs the same targets and blocks the merge when either fails.

## Adding or changing a rule

A rule keeps the message ID and symbol of the `pylint_django` message it
ports. List it in `RULES` in `src/rules.rs`, implement it there or in
`src/migrations.rs`, and add it to the table in the [README](README.md#rules).
Cover it
with a fixture: a `.py` file and its `.expected` findings in `tests/fixtures/`.

## Commits and branches

- Commit messages follow
  [Conventional Commits](https://www.conventionalcommits.org/).
- Branch names follow
  [Conventional Branch](https://conventionalbranch.org/):
  `<type>/<description>`, for example `feat/add-login` or
  `fix/normalize-description-grammar`. The branch type is one of `feat`,
  `fix`, `hotfix`, `release`, or `chore`; documentation work uses `chore/`.

A pre-commit hook and `make commits-check` enforce both; see
[`docs/conventions/commits-check.md`](docs/conventions/commits-check.md).

## Versioning

The project follows [Semantic Versioning](https://semver.org/). A pull request
that changes behavior bumps the version in its own commit; maintainers tag
minor and major bumps and publish releases on demand. See
[`docs/conventions/versioning.md`](docs/conventions/versioning.md).

## Documentation

`docs/` is a bundle of atomic notes: one concept per file. A new note is
added to its directory's `index.md` and to [`docs/log.md`](docs/log.md) in
the same pull request. Read [`AGENTS.md`](AGENTS.md) for the rules that apply
to both people and coding agents working in this repository.

## License

By contributing, you agree that your contribution is licensed under the
project's license; see [LICENSE](LICENSE).
