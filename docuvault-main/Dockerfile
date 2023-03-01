FROM rust:1.67-alpine AS chef

WORKDIR /usr/src/docuvault

RUN set -eux; \
    apk add --no-cache musl-dev; \
    cargo install cargo-chef; \
    rm -rf $CARGO_HOME/registry

FROM chef as planner

COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /usr/src/docuvault/recipe.json .
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release

FROM alpine:latest

WORKDIR /usr/local/bin

COPY ./.env /usr/local/bin
COPY --from=builder /usr/src/docuvault/target/release/docuvault .

CMD ["./docuvault"]
