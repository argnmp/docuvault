FROM rust:1.76-alpine AS chef
RUN set -eux; apk add --no-cache musl-dev;
RUN apk add --no-cache protoc

RUN cargo install cargo-chef;
WORKDIR /app

FROM chef AS planner
COPY ./protos/ ../protos/
COPY ./docuvault-file/Cargo.toml .
COPY ./docuvault-file/Cargo.lock .
COPY ./docuvault-file/src ./src
COPY ./docuvault-file/macros ./macros
COPY ./docuvault-file/build.rs .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

COPY ./docuvault-file/macros ./macros
RUN cargo chef cook --release --recipe-path recipe.json
COPY ./protos/ ../protos/
COPY ./docuvault-file/Cargo.toml .
COPY ./docuvault-file/Cargo.lock .
COPY ./docuvault-file/src ./src
COPY ./docuvault-file/macros ./macros
COPY ./docuvault-file/build.rs .
RUN cargo build --release

FROM alpine:latest
COPY --from=builder /app/target/release/docuvault-file /app/docuvault-file
CMD ["./app/docuvault-file"]

