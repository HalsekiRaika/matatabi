FROM rust:1.56.1 as builder

WORKDIR /usr/src/matatabi

COPY cargo.toml .
COPY cargo.lock .
