.PHONY: lint lint-fix fmt fmt-check build-migrations migrate migrate-full

CLIPPY_ALLOW = -A clippy::wildcard-imports -A clippy::missing_errors_doc
CLIPPY_FLAGS = -W clippy::pedantic $(CLIPPY_ALLOW) -D warnings

lint:
	cargo +nightly clippy --all-targets -- $(CLIPPY_FLAGS)

lint-fix:
	cargo +nightly clippy --all-targets --fix -- $(CLIPPY_FLAGS)

fmt:
	cargo +nightly fmt

fmt-check:
	cargo +nightly fmt --check

build-migrations:
	docker build --pull -f Dockerfile.migrations -t blokmap-migrations:latest .

migrate: build-migrations
	docker run \
		--network blokmap-dev-network \
		-e DATABASE_URL=postgresql://blokmap:appel@blokmap-dev-database:5432/blokmap \
		blokmap-migrations:latest

migrate-full: build-migrations
	docker run \
		--network blokmap-dev-network \
		-e DATABASE_URL=postgresql://blokmap:appel@blokmap-dev-database:5432/blokmap \
		blokmap-migrations:latest \
		migration redo -a
