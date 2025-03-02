FROM clux/muslrust:1.85.0-stable AS build-root

RUN cargo new --vcs none --bin --edition 2024 blokmap-backend

# muslrust starts in /volume
WORKDIR /volume/blokmap-backend

COPY ./.cargo ./Cargo.toml ./Cargo.lock ./


FROM alpine:latest AS execution-root

RUN apk add --no-cache curl

HEALTHCHECK --interval=30s --timeout=5s --start-period=30s --retries=5 \
	CMD curl -f http://localhost/healthcheck || exit 1

WORKDIR /blokmap-backend

CMD [ "./blokmap-backend" ]


#######################
# DEVELOPMENT TARGETS #
#######################

# Build
FROM build-root AS development-build

RUN cargo build --bin blokmap-backend

RUN rm -r ./src
RUN rm ./target/x86_64-unknown-linux-musl/debug/deps/blokmap_backend*

COPY ./src ./src

RUN cargo build --bin blokmap-backend

# Run
FROM execution-root AS development

COPY --from=development-build /volume/blokmap-backend/target/x86_64-unknown-linux-musl/debug/blokmap-backend .


######################
# PRODUCTION TARGETS #
######################

# Build
FROM build-root AS production-build

RUN cargo build --release --bin blokmap-backend

RUN rm -r ./src
RUN rm ./target/x86_64-unknown-linux-musl/release/deps/blokmap_backend*

COPY ./src ./src

RUN cargo build --release --bin blokmap-backend

# Run
FROM execution-root AS production

COPY --from=production-build /volume/blokmap-backend/target/x86_64-unknown-linux-musl/release/blokmap-backend .

