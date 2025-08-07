FROM rust:1.88.0-slim AS base

# Install build dependencies
ENV DEBIAN_FRONTEND=noninteractive
RUN apt update \
	&& apt install -y make libpq5 libpq-dev curl pkg-config openssl libssl-dev \
	&& rm -rf /var/lib/apt/lists/*

RUN rustup default nightly
RUN rustup component add clippy rustfmt

WORKDIR /blokmap

COPY . .


FROM base AS lint

CMD [ "make", "lint" ]


FROM base AS test

RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall -y --force diesel_cli
RUN diesel migration run

CMD [ "cargo", "test", "--tests" ]
