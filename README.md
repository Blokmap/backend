# Backend Setup Guide

## Requirements

A recent version of [Rust](https://www.rust-lang.org/tools/install)

## Starting the Local Development Server

See the [deploy](https://github.com/Blokmap/deploy) repository for instructions on running the application

## Useful Commands

Since we are working with a dockerized application, some commands need to be run inside the container. We provided a `Makefile` for these cases.

The testing commands will require the application to run locally, this requires
setting certain environment variables. To this end an example
[`.env`](./.env.example) file has been provided.

### Database Management

```sh
make migrate      # Run pending migrations
make migrate-full # Redo all migrations (!!deletes data!!)
```

### Running the Linter and Formatter

```sh
make lint
make fmt
```

### Running the Tests

```sh
cargo test
cargo test --tests                    # Skip doctests
cargo test --test <target>            # Only test <Target>
cargo test <test-function> -- --exact # Only run <test-function>
```

If you want the application logs to be printed while running the tests:

```sh
CI=true cargo tets -- --nocapture
```