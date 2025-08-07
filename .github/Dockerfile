FROM rust:1.88.0-slim

# Install build dependencies
ENV DEBIAN_FRONTEND=noninteractive
RUN apt update \
	&& apt install -y libpq5 libpq-dev curl pkg-config openssl libssl-dev \
	&& rm -rf /var/lib/apt/lists/*

WORKDIR /blokmap

COPY . .
