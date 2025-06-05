# ERP API
API untuk sistem ERP perusahaan menggunakan Actix-Web, MariaDB, dan Meilisearch, dengan Diesel-RS untuk ORM dan Supervisord untuk manajemen proses, dijalankan dengan Docker.

## Prasyarat
- Docker
- Docker Compose

## Setup dengan Docker
- Clone repository ini.
- Pastikan Docker dan Docker Compose terinstall.
- (Opsional) Salin file .env.example ke .env dan sesuaikan jika perlu (default sudah disediakan di docker-compose.yml).
- Jalankan aplikasi, MySQL, dan Meilisearch menggunakan Docker Compose:
```
docker-compose up -d
```
- Aplikasi akan berjalan di http://localhost:8080, MySQL di localhost:3306, dan Meilisearch di localhost:7700.

## Menjalankan Tes
1. Pastikan layanan Docker Compose berjalan (docker-compose up -d).
2. Masuk ke container aplikasi:
```
docker-compose exec app bash
```
3. Di dalam container, jalankan tes:
```
cargo test
```

## Menghasilkan Laporan Code Coverage
1. Masuk ke container aplikasi:
```
docker-compose exec app bash
```
2. Install cargo-tarpaulin di dalam container:
```
cargo install cargo-tarpaulin
```
3. Jalankan laporan coverage:
```
cargo tarpaulin --out Html
```
4. Salin file tarpaulin-report.html ke host:
```
docker cp <container_id>:/app/tarpaulin-report.html .
```
5. Buka file tarpaulin-report.html di browser untuk melihat laporan.

## Debugging
### Database (MariaDB):
1. Masuk ke container:
```
docker-compose exec app bash
```
2. Gunakan mariadb client:
```
mariadb -h mariadb -u user -ppassword erp_db
```
### Meilisearch:
1. Periksa status Meilisearch:
```
docker-compose exec app bash
curl -f http://meilisearch:7700/health
```
2. Jika gagal, periksa log Meilisearch:
```
docker-compose logs meilisearch
```
### Log Aplikasi:
1. Periksa log Supervisord:
```
docker-compose exec app bash
cat /var/log/erp_api.log
cat /var/log/erp_api_err.log
cat /var/log/diesel_migration.log
cat /var/log/diesel_migration_err.log
```

## Endpoint
- POST /inventory/create: Membuat item inventory baru. Contoh payload:
```
{
    "name": "Laptop Test",
    "quantity": 5,
    "price": 999.99
}
```
- GET /inventory/search?q={query}: Mencari item inventory menggunakan Meilisearch. Contoh: http://localhost:8080/inventory/search?q=Laptop.

## Menghentikan Layanan
Hentikan dan hapus container:
```
docker-compose down
```

## Catatan
- Supervisord digunakan untuk mengelola proses migrasi Diesel dan aplikasi dalam container, dengan wait-for-it untuk memastikan MariaDB siap.
- File migrasi di /migrations akan otomatis dijalankan saat container MySQL dimulai.
- Pastikan port 8080, 3306, dan 7700 tidak digunakan oleh aplikasi lain di host.
- Jika ingin mengubah kredensial database atau kunci Meilisearch, edit docker-compose.yml atau buat file .env.
