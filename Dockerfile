FROM rust:1-bookworm as builder

WORKDIR /app

# Copy your source code
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build for release
RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app

COPY --from=builder /app/target/release/lux /app/

CMD [ "/app/lux" ]