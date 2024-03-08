FROM rust:1.76-alpine AS chef
RUN set -eux; apk add --no-cache musl-dev;
RUN apk add --no-cache protoc

RUN cargo install cargo-chef;
WORKDIR /app

FROM chef AS planner
COPY ./protos/ ../protos/
COPY ./docuvault-convert/Cargo.toml .
COPY ./docuvault-convert/Cargo.lock .
COPY ./docuvault-convert/src ./src
COPY ./docuvault-convert/macros ./macros
COPY ./docuvault-convert/build.rs .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

COPY ./docuvault-convert/macros ./macros
RUN cargo chef cook --release --recipe-path recipe.json
COPY ./protos/ ../protos/
COPY ./docuvault-convert/Cargo.toml .
COPY ./docuvault-convert/Cargo.lock .
COPY ./docuvault-convert/src ./src
COPY ./docuvault-convert/macros ./macros
COPY ./docuvault-convert/build.rs .
RUN cargo build --release

FROM alpine:latest
RUN apk add --no-cache pandoc
COPY --from=builder /app/target/release/docuvault-convert /app/docuvault-convert
CMD ["./app/docuvault-convert"]

