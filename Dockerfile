# Stage 1: Planner - Create a dependency plan and cache dependencies
FROM rust:1.88-slim-bullseye AS planner
WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    libssl-dev \
    libmariadb-dev \
    pkg-config

# Copy dependency definitions
COPY Cargo.toml Cargo.lock ./
COPY api/Cargo.toml ./api/
COPY config/Cargo.toml ./config/
COPY db/Cargo.toml ./db/
COPY entity/Cargo.toml ./entity/
COPY migration/Cargo.toml ./migration/
COPY search/Cargo.toml ./search/

# Create dummy files to build the dependency graph
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN mkdir -p api/src && echo "pub fn lib() {}" > api/src/lib.rs
RUN mkdir -p config/src && echo "pub fn lib() {}" > config/src/lib.rs
RUN mkdir -p db/src && echo "pub fn lib() {}" > db/src/lib.rs
RUN mkdir -p entity/src && echo "pub fn lib() {}" > entity/src/lib.rs
RUN mkdir -p migration/src
RUN echo "fn main() {}" > migration/src/main.rs
RUN echo "pub fn lib() {}" > migration/src/lib.rs
RUN mkdir -p search/src && echo "pub fn lib() {}" > search/src/lib.rs

# Build only the dependencies to cache them
RUN cargo build --release --workspace


# Stage 2: Builder - Build the actual application
FROM rust:1.88-slim-bullseye AS builder
WORKDIR /app

# Install build dependencies (required for final linking)
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    libssl-dev \
    libmariadb-dev \
    pkg-config

# Copy pre-built dependencies from the planner stage
COPY --from=planner /app/target ./target
COPY --from=planner /usr/local/cargo/registry /usr/local/cargo/registry

# Copy all source code
COPY . .

# Clean the dummy artifacts of local crates to force recompilation
RUN cargo clean -p api -p config -p db -p entity -p migration -p search

# Build the application binary, which will use the cached dependencies
RUN cargo build --release --bin erp-api


# Stage 3: Final Image - Create the small production image
FROM debian:bullseye-slim AS final

# Create a non-root user and group for security
RUN groupadd --system appuser && useradd --system --gid appuser appuser

# Install runtime dependencies and clean up apt cache
RUN apt-get update && apt-get install -y --no-install-recommends \
    openssl \
    libmariadb3 \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy configuration files and set ownership
COPY --chown=appuser:appuser config ./config

# Copy the compiled binary from the builder stage and set ownership
COPY --from=builder --chown=appuser:appuser /app/target/release/erp-api .

# Switch to the non-root user
USER appuser

# Expose the port the application will run on
EXPOSE 8000

# Set the command to run the application
CMD ["./erp-api"]