use actix_web::{web, App, HttpServer};
use diesel::prelude::*;
use erp_api::api::v1::{inventory};
use erp_api::config::settings::Settings;
use erp_api::db::mysql::init_db_pool;
use erp_api::search::meilisearch::init_meilisearch;
use meilisearch_sdk::client::Client;

use erp_api::db::mysql::DbPool;

pub async fn setup_test_app() -> (DbPool, Client, String) {
    dotenv::dotenv().ok();
    let settings = Settings::new("test");
    
    // Inisialisasi database
    let db_pool = init_db_pool(&settings.database_url);
    
    // Bersihkan tabel inventory untuk tes
    let conn = &mut db_pool.get().expect("Gagal mendapatkan koneksi database");
    diesel::sql_query("TRUNCATE TABLE inventory")
        .execute(conn)
        .expect("Gagal membersihkan tabel inventory");

    // Inisialisasi Meilisearch
    let meili_client = init_meilisearch(&settings.meilisearch_host, &settings.meilisearch_api_key)
        .await
        .expect("Gagal inisialisasi Meilisearch untuk tes");
    
    // Bersihkan indeks Meilisearch
    let index = meili_client.index("inventory");
    index.delete().await.expect("Gagal menghapus indeks Meilisearch");

    // Clone db_pool and meili_client for moving into the closure
    let db_pool_for_server = db_pool.clone();
    let meili_client_for_server = meili_client.clone();

    // Jalankan server di port acak
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_pool_for_server.clone()))
            .app_data(web::Data::new(meili_client_for_server.clone()))
            .configure(inventory::routes::init_routes)
    });

    let bind_addr = "127.0.0.1:0";
    let server = server.bind(bind_addr).expect("Gagal bind server");
    let server_addr = server.addrs().first().expect("Gagal mendapatkan alamat server").to_owned();
    let server_url = format!("http://{}", server_addr);

    // Jalankan server di background
    tokio::spawn(server.run());

    (db_pool, meili_client, server_url)
}