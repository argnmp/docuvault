FROM rust:1.67-alpine AS chef
RUN apk add --no-cache protoc

WORKDIR /usr/src/file-reverse-proxy

RUN set -eux; \
    apk add --no-cache musl-dev; \
    cargo install cargo-chef; \
    rm -rf $CARGO_HOME/registry

FROM chef as planner

COPY ./file-reverse-proxy .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /usr/src/file-reverse-proxy/recipe.json .
RUN cargo chef cook --release --recipe-path recipe.json

COPY ./protos/ ../protos/
COPY ./file-reverse-proxy .
RUN cargo build --release

FROM alpine:latest

WORKDIR /usr/local/bin

COPY ./file-reverse-proxy/.env /usr/local/bin
COPY --from=builder /usr/src/file-reverse-proxy/target/release/file-reverse-proxy .

CMD ["./file-reverse-proxy"]
