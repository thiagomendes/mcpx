# Development Dockerfile for Rust backend
FROM rustlang/rust:nightly-bookworm

WORKDIR /app

# Copy Cargo files first for dependency caching
COPY Cargo.toml Cargo.lock* ./

# Create a dummy main.rs for dependency compilation
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build 2>/dev/null || true
RUN rm -rf src

# Copy actual source
COPY . .

# Build in debug mode for faster compilation
RUN cargo build

EXPOSE 8080

# Enable backtraces for debugging
ENV RUST_BACKTRACE=1

# Run the binary directly
CMD ["./target/debug/mcpx-backend"]
