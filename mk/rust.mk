CARGO := cargo
CARGO_DENY_VERSION := 0.20.2
WASM_TARGET := wasm32-wasip2
PLUGIN := target/$(WASM_TARGET)/release/ferrolint_django.wasm
PACKAGE_DIR := python/ferrolint_django

LANG_INSTALL_TARGETS += install-rust
LANG_CHECK_TARGETS   += clippy fmt-check deny
LANG_FIX_TARGETS     += fmt
LANG_TEST_TARGETS    += cargo-test

##@ Rust

install-rust: ## Fetch the crate's dependencies and install the pinned cargo-deny
	$(CARGO) fetch --locked
	$(CARGO) install --locked cargo-deny@$(CARGO_DENY_VERSION)

clippy: ## Lint with Clippy for the host and the plugin target, treating warnings as errors
	$(CARGO) clippy --locked --all-targets -- -D warnings
	$(CARGO) clippy --locked --target $(WASM_TARGET) -- -D warnings

fmt: ## Format code with rustfmt in place — accepts FILES="..."
	$(CARGO) fmt -- $(FILES)

fmt-check: ## Check formatting with rustfmt, without writing — accepts FILES="..."
	$(CARGO) fmt --check -- $(FILES)

deny: ## Check dependencies for advisories, licenses and sources with cargo-deny
	$(CARGO) deny --locked check

cargo-test: ## Run the test suite with cargo test (needs a ferrolint binary, see CONTRIBUTING.md)
	$(CARGO) test --locked

build: ## Build the crate into target/
	$(CARGO) build --locked

plugin: ## Build the plugin as a WASM component into python/ferrolint_django/plugin.wasm
	$(CARGO) build --locked --release --target $(WASM_TARGET)
	cp $(PLUGIN) $(PACKAGE_DIR)/plugin.wasm

wheel: plugin ## Build the py3-none-any wheel that carries plugin.wasm into dist/
	uv build --wheel

ferrolint-bin: ## Build the ferrolint binary of the sibling checkout that tests/fixtures.rs runs
	$(CARGO) build --locked --release -j 5 --manifest-path ../ferrolint/Cargo.toml -p ferrolint

release-tag: ## Tag the base branch as v<package.version> and push the tag (minor and major bumps)
	@version=$$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -1); base=$(BASE:origin/%=%); tag=v$$version; \
	test "$$version" && test "$$(git rev-parse --abbrev-ref HEAD)" = "$$base" && test -z "$$(git status --porcelain)" || { echo "release-tag: run it from a clean $$base, which must have a version in Cargo.toml"; exit 1; }; \
	git fetch --quiet origin && test "$$(git rev-parse HEAD)" = "$$(git rev-parse origin/$$base)" || { echo "release-tag: $$base is not at origin/$$base"; exit 1; }; \
	git rev-parse --quiet --verify "refs/tags/$$tag" >/dev/null && { echo "release-tag: $$tag already exists"; exit 1; }; \
	git tag -a "$$tag" -m "$$tag" && git push origin "$$tag"

.PHONY: install-rust clippy fmt fmt-check deny cargo-test build plugin wheel ferrolint-bin release-tag
