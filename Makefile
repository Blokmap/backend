.PHONY: lint lint-fix fmt fmt-check

CLIPPY_ALLOW = -A clippy::wildcard-imports
CLIPPY_FLAGS = -W clippy::pedantic $(CLIPPY_ALLOW) -D warnings

lint:
	cargo +nightly clippy --all-targets -- $(CLIPPY_FLAGS)

lint-fix:
	cargo +nightly clippy --all-targets --fix -- $(CLIPPY_FLAGS)

fmt:
	cargo +nightly fmt

fmt-check:
	cargo +nightly fmt --check
