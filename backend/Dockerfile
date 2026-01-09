# Development Dockerfile for Rust backend
# Multi-stage build for web and worker binaries

FROM rustlang/rust:nightly-bookworm AS builder

WORKDIR /app

# Copy Cargo files first for dependency caching
COPY Cargo.toml Cargo.lock* ./

# Create dummy files for dependency compilation
RUN mkdir -p src/bin && \
    echo "fn main() {}" > src/main.rs && \
    echo "fn main() {}" > src/bin/web.rs && \
    echo "fn main() {}" > src/bin/worker.rs && \
    echo "" > src/lib.rs
RUN cargo build 2>/dev/null || true
RUN rm -rf src target/debug/mcpx* target/debug/deps/mcpx*

# Copy actual source
COPY . .

# Force rebuild by touching source files
RUN touch src/lib.rs src/bin/web.rs src/bin/worker.rs

# Build all binaries
RUN cargo build

# ============================================
# Web server image
# ============================================
FROM rustlang/rust:nightly-bookworm AS web

WORKDIR /app
COPY --from=builder /app/target/debug/mcpx-web /app/
COPY --from=builder /app/migrations /app/migrations

EXPOSE 8080
ENV RUST_BACKTRACE=1

CMD ["./mcpx-web"]

# ============================================
# Worker image
# ============================================
FROM rustlang/rust:nightly-bookworm AS worker

WORKDIR /app
COPY --from=builder /app/target/debug/mcpx-worker /app/
COPY --from=builder /app/migrations /app/migrations

ENV RUST_BACKTRACE=1

CMD ["./mcpx-worker"]
