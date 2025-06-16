# ERP API

API untuk sistem ERP perusahaan menggunakan Actix-Web, MariaDB, dan Meilisearch, dengan SeaORM untuk ORM, Supervisord untuk manajemen proses, dan Utoipa dengan Scalar untuk dokumentasi API, dijalankan dengan Docker.

## Prasyarat
- Docker
- Docker Compose

## Setup dengan Docker
1. Clone repository ini.
2. Pastikan Docker dan Docker Compose terinstall.
3. Salin file `.env.example` ke `.env` dan sesuaikan jika perlu.
4. MariaDB, dan Meilisearch menggunakan Docker Compose:
   ```bash
   docker-compose up -d
   ```
   Pastikan semua service healthy.
5. Jalankan database migration:
   ```bash
   cargo install --locked sea-orm-cli
   sea-orm-cli migrate up
   ```
5. Jalankan aplikasi:
   ```bash
   cargo run
   ```
   Untuk mengaktifkan watch mode dapat dilakukan dengan:
   ```bash
   cargo install --locked watchexec-cli
   watchexec -w src -r cargo run
   ```
6. Aplikasi akan berjalan di `http://localhost:8080`, MariaDB di `localhost:3306`, dan Meilisearch di `localhost:7700`.

## Mengakses Dokumentasi API
- Buka Scalar UI di: `http://localhost:8080/scalar`.
- Spesifikasi OpenAPI tersedia di: `http://localhost:8080/api-docs/openapi.json` (otomatis disajikan oleh `utoipa-scalar`).

## Menjalankan Tes
1. Jalankan tes:
   ```bash
   cargo test
   ```

## Menghasilkan Laporan Code Coverage
1. Install `cargo-tarpaulin`:
   ```bash
   cargo install cargo-tarpaulin
   ```
2. Jalankan laporan coverage:
   ```bash
   chmod +x test.sh
   ./test.sh
   ```
3. Buka file `tarpaulin-report.html` di browser untuk melihat laporan.

## Endpoint
- `POST /inventory/create`: Membuat item inventory baru. Contoh payload:
  ```json
  {
      "name": "Laptop Test",
      "quantity": 5,
      "price": 999.99
  }
  ```
- `GET /inventory/search?q={query}`: Mencari item inventory menggunakan Meilisearch. Contoh: `http://localhost:8080/inventory/search?q=Laptop`.

## Menghentikan Layanan
Hentikan dan hapus container:
```bash
docker-compose down
```
## Building App
1. Pastikan sudah login ke registry.gmedia.id
2. Build image
   ```bash
   docker build -t registry.gmedia.id/gmd/erp-api:rust -f ./docker/Dockerfile .
   ```

## Struktur Proyek
```
/erp-api
├── /api
│   ├── /src
│   │   ├── /v1
│   │   │   ├── /inventory
│   │   │   │   ├── mod.rs
│   │   │   │   ├── handlers.rs
│   │   │   │   ├── models.rs
│   │   │   │   └── routes.rs
│   │   │   ├── /employee
│   │   │   │   ├── mod.rs
│   │   │   │   ├── handlers.rs
│   │   │   │   ├── models.rs
│   │   │   │   └── routes.rs
│   │   │   ├── /order
│   │   │   │   ├── mod.rs
│   │   │   │   ├── handlers.rs
│   │   │   │   ├── models.rs
│   │   │   │   └── routes.rs
│   │   │   └── mod.rs
│   │   ├── openapi.rs
│   │   └── lib.rs
│   └── Cargo.toml
├── /config
│   ├── /src
│   │   ├── db.rs
│   │   ├── meilisearch.rs
│   │   └── lib.rs
│   └── Cargo.toml
├── /db
│   ├── /src
│   │   ├── mysql.rs
│   │   └── lib.rs
│   └── Cargo.toml
├── /entity
│   ├── /src
│   │   ├── employee.rs
│   │   ├── inventory.rs
│   │   ├── order.rs
│   │   ├── prelude.rs
│   │   └── lib.rs
│   └── Cargo.toml
├── /migration
│   ├── /src
│   │   ├── m20250604_000000_create_inventory.rs
│   │   ├── m20250604_000001_create_employee.rs
│   │   ├── m20250604_000002_create_order.rs
│   │   ├── main.rs
│   │   └── lib.rs
│   └── Cargo.toml
├── /search
│   ├── /src
│   │   ├── meilisearch.rs
│   │   └── lib.rs
│   └── Cargo.toml
├── /src
│   ├── main.rs
│   └── lib.rs
├── /tests
│   ├── common.rs
│   ├── inventory.rs
│   ├── employee.rs
│   └── order.rs
├── .env
├── Cargo.toml
├── docker-compose.yml
└── README.md
```
