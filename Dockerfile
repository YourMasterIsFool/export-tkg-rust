# Stage 1: Build
FROM rust:latest AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Stage 2: Runtime — gunakan image dengan GLIBC yang cocok
FROM ubuntu:24.04

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/export_rust /usr/local/bin/

CMD ["/usr/local/bin/export_rust"]