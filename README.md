# ERP API

API untuk sistem ERP perusahaan menggunakan Actix-Web, MariaDB, dan Meilisearch, dengan Diesel-RS untuk ORM, Supervisord untuk manajemen proses, dan Utoipa dengan Scalar untuk dokumentasi API, dijalankan dengan Docker.

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
5. Jalankan aplikasi:
   ```bash
   cargo run
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
   cargo tarpaulin --out Html
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
├── /src
│   ├── /api
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
│   │   └── mod.rs
│   ├── /config
│   │   ├── mod.rs
│   │   └── settings.rs
│   ├── /db
│   │   ├── mod.rs
│   │   ├── mysql.rs
│   │   └── schema.rs
│   ├── /search
│   │   ├── mod.rs
│   │   └── meilisearch.rs
│   ├── /middleware
│   │   ├── mod.rs
│   │   └── auth.rs
│   ├── /utils
│   │   ├── mod.rs
│   │   └── errors.rs
│   ├── main.rs
│   └── lib.rs
├── /tests
│   ├── common.rs
│   ├── inventory_tests.rs
│   ├── employee_tests.rs
│   └── order_tests.rs
├── /migrations
│   ├── 2025-06-04-000000_create_inventory
│   │   ├── up.sql
│   │   └── down.sql
│   ├── 2025-06-04-000001_create_employees
│   │   ├── up.sql
│   │   └── down.sql
│   ├── 2025-06-04-000002_create_orders
│   │   ├── up.sql
│   │   └── down.sql
├── Cargo.toml
├── README.md
├── .env
└── docker-compose.yml
```