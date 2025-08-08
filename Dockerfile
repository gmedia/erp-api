# Stage 1: Planner - Create a dependency plan and cache dependencies
# For reproducible builds, consider pinning the digest: @sha256:...
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
COPY page/Cargo.toml ./page/

# Create dummy files to build the dependency graph
RUN mkdir -p src && echo "fn main() {}" > src/main.rs && \
    mkdir -p api/src && echo "pub fn lib() {}" > api/src/lib.rs && \
    mkdir -p config/src && echo "pub fn lib() {}" > config/src/lib.rs && \
    mkdir -p db/src && echo "pub fn lib() {}" > db/src/lib.rs && \
    mkdir -p entity/src && echo "pub fn lib() {}" > entity/src/lib.rs && \
    mkdir -p migration/src && \
    echo "fn main() {}" > migration/src/main.rs && \
    echo "pub fn lib() {}" > migration/src/lib.rs && \
    mkdir -p search/src && echo "pub fn lib() {}" > search/src/lib.rs && \
    mkdir -p page/src && echo "pub fn lib() {}" > page/src/lib.rs

# Build only the dependencies to cache them
RUN cargo build --release --workspace

# Stage 2: Builder - Build the actual application
# For reproducible builds, consider pinning the digest: @sha256:...
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
RUN cargo clean -p api -p config -p db -p entity -p migration -p search -p page

# Build the application binary, which will use the cached dependencies
RUN cargo build --release --bin erp-api
RUN cargo build --release --bin migration --package migration

# Stage 3: Final Image - Create the small production image
# For reproducible builds, consider pinning the digest: @sha256:...
FROM debian:bullseye-slim AS final

# Create a non-root user and group for security
RUN groupadd --system appuser && useradd --system --gid appuser appuser

# Install runtime dependencies and clean up apt cache
RUN apt-get update && apt-get install -y --no-install-recommends \
    openssl \
    libmariadb3 \
    supervisor \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy configuration files and set ownership
COPY --chown=appuser:appuser config ./config

# Copy the compiled binary from the builder stage and set ownership
COPY --from=builder --chown=appuser:appuser /app/target/release/erp-api .
COPY --from=builder --chown=appuser:appuser /app/target/release/migration .

# Copy wait-for-it
COPY ./wait-for-it.sh /usr/local/bin/wait-for-it.sh
RUN chmod +x /usr/local/bin/wait-for-it.sh

# Copy supervisord configuration file
COPY ./supervisord.conf /etc/supervisor/conf.d/supervisord.conf

# Copy public directory
COPY ./public /app/public

# Switch to the non-root user
USER appuser

# Expose the port the application will run on
EXPOSE 8000

# Run supervisord
CMD ["/usr/bin/supervisord", "-c", "/etc/supervisor/conf.d/supervisord.conf"]
