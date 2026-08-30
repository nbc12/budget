# --- Stage 1: Build the Application ---
FROM rust:1.93-slim AS builder

WORKDIR /usr/src/workspace

# Install build dependencies for SQLite, networking, and the Svelte frontend
# (build.rs shells out to npm to build frontend/ into frontend/dist/)
RUN apt-get update && apt-get install -y pkg-config libssl-dev libsqlite3-dev curl \
    && curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs

# Copy the entire workspace
COPY . .

# Build the specific bin crate named 'app'
RUN cargo build --release --bin app

# --- Stage 2: Create the Minimal Runtime Image ---
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y ca-certificates libsqlite3-0 && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the workspace target directory
# We'll rename it to budget_app to match your systemd naming convention
COPY --from=builder /usr/src/workspace/target/release/app /app/budget_app

# Create a dedicated directory for the SQLite database
RUN mkdir -p /app/data

# Expose port 3000 as defined in your systemd environment
EXPOSE 3000

# Run the binary
CMD ["./budget_app"]
