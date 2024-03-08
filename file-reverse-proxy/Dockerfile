FROM rust:1.76-alpine AS chef
RUN set -eux; apk add --no-cache musl-dev;
RUN apk add --no-cache protoc

RUN cargo install cargo-chef;
WORKDIR /app

FROM chef AS planner
COPY ./protos/ ../protos/
COPY ./file-reverse-proxy/Cargo.toml .
COPY ./file-reverse-proxy/Cargo.lock .
COPY ./file-reverse-proxy/src ./src
COPY ./file-reverse-proxy/macros ./macros
COPY ./file-reverse-proxy/build.rs .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

COPY ./file-reverse-proxy/macros ./macros
RUN cargo chef cook --release --recipe-path recipe.json
COPY ./protos/ ../protos/
COPY ./file-reverse-proxy/Cargo.toml .
COPY ./file-reverse-proxy/Cargo.lock .
COPY ./file-reverse-proxy/src ./src
COPY ./file-reverse-proxy/macros ./macros
COPY ./file-reverse-proxy/build.rs .
RUN cargo build --release

FROM alpine:latest
COPY --from=builder /app/target/release/file-reverse-proxy /app/file-reverse-proxy
CMD ["./app/file-reverse-proxy"]

