FROM rust:1.76-alpine AS chef
RUN set -eux; apk add --no-cache musl-dev;
RUN apk add --no-cache protoc

RUN cargo install cargo-chef;
WORKDIR /app

FROM chef AS planner
COPY ./protos/ ../protos/
COPY ./docuvault-main/Cargo.toml .
COPY ./docuvault-main/Cargo.lock .
COPY ./docuvault-main/src ./src
COPY ./docuvault-main/macros ./macros
COPY ./docuvault-main/migration ./migration
COPY ./docuvault-main/build.rs .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

COPY ./docuvault-main/macros ./macros
COPY ./docuvault-main/migration ./migration
RUN cargo chef cook --release --recipe-path recipe.json
COPY ./docuvault-main/macros ./macros
COPY ./docuvault-main/migration ./migration
COPY ./protos/ ../protos/
COPY ./docuvault-main/Cargo.toml .
COPY ./docuvault-main/Cargo.lock .
COPY ./docuvault-main/src ./src
COPY ./docuvault-main/build.rs .
RUN cargo build --release

FROM alpine:latest
COPY --from=builder /app/target/release/docuvault /app/docuvault
CMD ["/app/docuvault"]
