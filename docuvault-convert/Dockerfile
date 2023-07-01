FROM rust:1.67-alpine AS chef
RUN apk add --no-cache protoc

WORKDIR /usr/src/docuvault-convert

RUN set -eux; \
    apk add --no-cache musl-dev; \
    cargo install cargo-chef; \
    rm -rf $CARGO_HOME/registry

FROM chef as planner

COPY ./docuvault-convert .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /usr/src/docuvault-convert/recipe.json .
RUN cargo chef cook --release --recipe-path recipe.json

COPY ./protos/ ../protos/
COPY ./docuvault-convert .
RUN cargo build --release

FROM alpine:latest

WORKDIR /usr/local/bin

COPY ./docuvault-convert/.env /usr/local/bin
COPY --from=builder /usr/src/docuvault-convert/target/release/docuvault-convert .

CMD ["./docuvault-convert"]
