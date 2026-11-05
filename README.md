# ferrolint-django

[![ci](https://img.shields.io/github/actions/workflow/status/gajaguar/ferrolint-django/ci.yml?branch=main&style=flat-square&label=ci)](https://github.com/gajaguar/ferrolint-django/actions/workflows/ci.yml)
[![rust](https://img.shields.io/github/actions/workflow/status/gajaguar/ferrolint-django/rust.yml?branch=main&style=flat-square&label=rust)](https://github.com/gajaguar/ferrolint-django/actions/workflows/rust.yml)
[![PyPI](https://img.shields.io/pypi/v/ferrolint-django?style=flat-square)](https://pypi.org/project/ferrolint-django/)
[![crates.io](https://img.shields.io/crates/v/ferrolint-django?style=flat-square)](https://crates.io/crates/ferrolint-django)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](LICENSE)
[![Topics](https://img.shields.io/badge/topics-django%20%7C%20ferrolint%20%7C%20linter%20%7C%20rust-informational?style=flat-square)](https://github.com/gajaguar/ferrolint-django)

Pylint-django's rules as a [ferrolint](https://github.com/gajaguar/ferrolint)
plugin, so a Django project keeps its `pylint_django` messages in a ferrolint
run.

## Contents

- [About](#about)
- [Key features](#key-features)
- [Requirements](#requirements)
- [Usage](#usage)
- [Rules](#rules)
- [Security](#security)
- [Contributing](#contributing)
- [Open items](#open-items)
- [License](#license)

## About

[pylint-django](https://github.com/pylint-django/pylint-django) adds Django
checks to Pylint. ferrolint runs Pylint-compatible checks as WebAssembly
plugins next to Ruff. This crate ports the `pylint_django` rules that the
syntax decides, plus its migration checks, to a ferrolint plugin built for
`wasm32-wasip2`. The rules keep Pylint's message IDs and symbols, so
`# pylint: disable=` comments and selections in an existing configuration
keep working.

## Key features

- **Thirteen `pylint_django` rules** with their original IDs and symbols; see
  [Rules](#rules).
- **Migration checks** as a sub-plugin that loads only when the configuration
  names it, as Pylint does.
- **One component, two plugin names.** The package registers `pylint_django`
  and `pylint_django.checkers.migrations` as `ferrolint.plugins` entry points,
  so a `load-plugins` line written for Pylint works unchanged.

## Requirements

- A ferrolint that supports plugin contract 1.2.0, and Python 3.9 or later.
- To build: the toolchain in `mise.toml` (Rust with the `wasm32-wasip2`
  target, `uv`), installed by `make install`, and a
  [ferrolint](https://github.com/gajaguar/ferrolint) binary for the tests; see
  `CONTRIBUTING.md`.

## Usage

Install the package into the virtual environment that ferrolint reads:

```bash
uv pip install ferrolint-django
```

To try an unreleased change, build the wheel from a checkout instead:

```bash
make wheel
uv pip install dist/*.whl
```

ferrolint finds a plugin only through `VIRTUAL_ENV`, and loads it only when
`load-plugins` names it. In `pyproject.toml`:

```toml
[tool.pylint.main]
load-plugins = "pylint_django, pylint_django.checkers.migrations"
```

Then run `ferrolint` in the project. Leave
`pylint_django.checkers.migrations` out to skip the two migration rules.

## Rules

| ID      | Symbol                                     | Flags                                                              |
| :------ | :----------------------------------------- | :----------------------------------------------------------------- |
| `E5101` | `model-unicode-not-callable`               | A model's `__unicode__` assigned something that is not callable.   |
| `E5141` | `hard-coded-auth-user`                     | The string `'auth.User'`.                                          |
| `E5142` | `imported-auth-user`                       | `User` imported from `django.contrib.auth.models`.                 |
| `R5101` | `http-response-with-json-dumps`            | `HttpResponse(json.dumps(data))`.                                  |
| `R5102` | `http-response-with-content-type-json`     | `HttpResponse(content_type='application/json')`.                   |
| `R5103` | `redundant-content-type-for-json-response` | `JsonResponse(content_type=...)`.                                  |
| `W5102` | `model-has-unicode`                        | A model that defines `__unicode__`.                                |
| `W5103` | `model-no-explicit-unicode`                | A model that only inherits `__unicode__`.                          |
| `W5104` | `modelform-uses-exclude`                   | A `ModelForm` whose `Meta` sets `exclude`.                         |
| `W5150` | `format-html-interpolation`                | `format_html()` with an interpolated format string.                |
| `W5151` | `mark-safe-interpolation`                  | `mark_safe()` of an interpolated string.                           |
| `W5197` | `missing-backwards-migration-callable`     | A `RunPython` migration without a backwards callable.              |
| `W5198` | `new-db-field-with-default`                | An `AddField` migration with a default, in an app's latest one.    |

## Security

Report a vulnerability by emailing <dev@gajaguar.com> rather than opening a
public issue. [SECURITY.md](SECURITY.md) has the supported versions.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Open items

- Only the `pylint_django` messages the syntax decides are ported, plus the
  two migration checks.

## License

See [LICENSE](LICENSE).
