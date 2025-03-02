FROM rust:1.85 AS build-root

RUN cargo new --vcs none --bin --edition 2024 blokmap-backend
WORKDIR /blokmap-backend

COPY ./.cargo ./Cargo.toml ./Cargo.lock ./


FROM debian:bookworm-slim AS execution-root

ENV DEBIAN_FRONTEND=noninteractive
RUN apt update \
	&& apt install -y libpq5 curl \
	&& rm -rf /var/lib/apt/lists/*

HEALTHCHECK --interval=30s --timeout=5s --start-period=30s --retries=5 \
	CMD curl -f http://localhost/healthcheck || exit 1

WORKDIR /blokmap-backend

CMD [ "./blokmap-backend" ]


#######################
# DEVELOPMENT TARGETS #
#######################

# Build
FROM build-root AS development-build

RUN cargo build

RUN rm ./src/* ./target/debug/deps/blokmap_backend*

COPY ./src ./src

RUN cargo build

# Run
FROM execution-root AS development

COPY --from=development-build /blokmap-backend/target/debug/blokmap-backend .


######################
# PRODUCTION TARGETS #
######################

# Build
FROM build-root AS production-build

RUN cargo build --release

RUN rm ./src/* ./target/release/deps/blokmap_backend*

COPY ./src ./src

RUN cargo build --release

# Run
FROM execution-root AS production

COPY --from=production-build /blokmap-backend/target/release/blokmap-backend .

