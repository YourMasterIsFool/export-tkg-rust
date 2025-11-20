# ===========================
# 1) BUILD STAGE
# ===========================
FROM rust:1.82 AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

COPY . .

RUN cargo build --release


# ===========================
FROM debian:bookworm-slim AS runtime


RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary dari builder
COPY --from=builder /app/target/release/export_rust /app/app

# Expose port Axum
EXPOSE 3000

# Jalankan
CMD ["/app/app"]
