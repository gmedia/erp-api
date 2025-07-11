# Gunakan image Rust resmi sebagai base untuk build
FROM rust:1.87 AS builder

# Buat direktori kerja
WORKDIR /usr/src/erp-api

# Salin file konfigurasi dan source code
COPY Cargo.toml .
COPY api ./api
COPY config ./config
COPY db ./db
COPY entity ./entity
COPY migration ./migration
COPY search ./search
COPY src ./src

# Install diesel_cli untuk migrasi
RUN cargo install diesel_cli --no-default-features --features mysql

# Build aplikasi
RUN cargo build --release

# Image final untuk produksi
FROM rust:1.87-slim

# Install libmariadb-dev untuk MariaDB, supervisor, mariadb-client, wget untuk wait-for-it, dan curl untuk healthcheck
RUN apt-get update && apt-get install -y \
    libmariadb-dev \
    supervisor \
    mariadb-client \
    wget \
    curl \
    telnet \
    iputils-ping \
    && rm -rf /var/lib/apt/lists/*

# Download wait-for-it
RUN wget -O /usr/local/bin/wait-for-it.sh https://raw.githubusercontent.com/vishnubob/wait-for-it/master/wait-for-it.sh \
    && chmod +x /usr/local/bin/wait-for-it.sh

WORKDIR /app

# Salin binary dan migrasi dari builder
COPY --from=builder /usr/src/erp-api/target/release/erp-api /app/erp-api
COPY --from=builder /usr/src/erp-api/migration /app/migration

# Salin diesel_cli untuk migrasi
COPY --from=builder /usr/local/cargo/bin/diesel /usr/local/bin/diesel

# Salin file konfigurasi supervisord
COPY docker/supervisord.conf /etc/supervisor/conf.d/supervisord.conf

# Jalankan supervisord
CMD ["/usr/bin/supervisord", "-c", "/etc/supervisor/conf.d/supervisord.conf"]