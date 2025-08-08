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
ARG NODE_VERSION=22

# Install build dependencies (required for final linking)
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    libssl-dev \
    libmariadb-dev \
    pkg-config \
    gnupg \
    curl

RUN mkdir -p /etc/apt/keyrings \
    && curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key | gpg --dearmor -o /etc/apt/keyrings/nodesource.gpg \
    && echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_$NODE_VERSION.x nodistro main" > /etc/apt/sources.list.d/nodesource.list

RUN apt-get update && apt-get install -y --no-install-recommends \
    nodejs

RUN npm install --ignore-scripts -g \
    npm

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

RUN npm install
RUN npm run build

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
    curl \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy configuration files and set ownership
COPY --from=builder --chown=appuser:appuser /app/config.toml .

# Copy the compiled binary from the builder stage and set ownership
COPY --from=builder --chown=appuser:appuser /app/target/release/erp-api .
COPY --from=builder --chown=appuser:appuser /app/target/release/migration .
COPY --from=builder --chown=appuser:appuser /app/public ./public
COPY --from=builder --chown=appuser:appuser /app/www/root.hbs ./www/root.hbs

RUN mkdir -p storage/sessions && chown -R appuser:appuser storage/sessions

# Copy wait-for-it
COPY ./wait-for-it.sh /usr/local/bin/wait-for-it.sh
RUN chmod +x /usr/local/bin/wait-for-it.sh

# Copy supervisord configuration file
COPY ./supervisord.conf /etc/supervisor/conf.d/supervisord.conf

# Switch to the non-root user
USER appuser

# Expose the port the application will run on
EXPOSE 8000

# Run supervisord
CMD ["/usr/bin/supervisord", "-c", "/etc/supervisor/conf.d/supervisord.conf"]
