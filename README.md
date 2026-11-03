# ferrolint-django

Pylint-django's rules as a [ferrolint](https://github.com/gajaguar/ferrolint)
plugin, so a Django project keeps its `pylint_django` messages in a ferrolint
run.

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

- A ferrolint that supports plugin contract 1.1.0, and Python 3.9 or later.
- To build: the toolchain in `mise.toml` (Rust with the `wasm32-wasip2`
  target, `uv`), installed by `make install`, and a
  [ferrolint](https://github.com/gajaguar/ferrolint) binary for the tests; see
  `CONTRIBUTING.md`.

## Usage

Build the wheel and install it into the virtual environment that ferrolint
reads:

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

## Open items

- Only the `pylint_django` messages the syntax decides are ported, plus the
  two migration checks.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

See [LICENSE](LICENSE).
