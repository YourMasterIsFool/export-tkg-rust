# Stage 1: Builder
FROM rust:latest AS builder
WORKDIR /app

# Copy Cargo files first for caching deps
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# Copy source code dan build release binary
COPY . .
RUN cargo build --release
RUN rm -rf src  # optional, bersihkan dummy src

# Stage 2: Minimal runtime
FROM debian:bookworm-slim
WORKDIR /app

COPY --from=builder /app/target/release/export_rust /usr/local/bin/export_rust

EXPOSE 5559

CMD ["export_rust"]
