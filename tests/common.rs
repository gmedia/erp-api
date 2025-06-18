use actix_web::{web, App, HttpServer};
use api::v1::{auth, employee, inventory, order};
use config::db::Db;
use config::meilisearch::Meilisearch;
use db::mysql::init_db_pool;
use search::meilisearch::init_meilisearch;
use search::Client;
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};

pub async fn setup_test_app() -> (DatabaseConnection, Client, String) {
    dotenv::dotenv().ok();
    let _ = env_logger::try_init();
    let config_db = Db::new("test");
    let config_meilisearch = Meilisearch::new("test");
    let config_app = config::app::AppConfig::new("test");
    let jwt_secret = "test-secret".to_string();

    // Inisialisasi database
    let db_pool = init_db_pool(&config_db.url)
        .await
        .expect("Gagal inisialisasi pool database");

    // Bersihkan tabel untuk tes
    for table in &config_app.tables {
        db_pool
            .execute(Statement::from_string(
                db_pool.get_database_backend(),
                format!("TRUNCATE TABLE `{}`;", table),
            ))
            .await
            .unwrap();
    }

    // Inisialisasi Meilisearch
    let meili_client =
        init_meilisearch(&config_meilisearch.host, &config_meilisearch.api_key)
            .await
            .expect("Gagal inisialisasi Meilisearch untuk tes");

    // Bersihkan indeks Meilisearch
    for (index_name, p_key) in &config_app.meilisearch_indexes {
        let pk: Vec<&str> = p_key.iter().map(|s| s.as_str()).collect();
        let _ = meili_client.create_index(index_name, Some(&pk[0])).await;
    }

    // Clone db_pool and meili_client for moving into the closure
    let db_pool_for_server = db_pool.clone();
    let meili_client_for_server = meili_client.clone();

    // Jalankan server di port acak
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(config::app::AppState {
                db: db_pool_for_server.clone(),
                meilisearch: meili_client_for_server.clone(),
                jwt_secret: jwt_secret.clone(),
            }))
            // Register your routes here
            .configure(inventory::routes::init_routes)
            .configure(employee::routes::init_routes)
            .configure(order::routes::init_routes)
            .configure(auth::routes::init_routes)
    });

    let bind_addr = "127.0.0.1:0";
    let server = server.bind(bind_addr).expect("Gagal bind server");
    let server_addr = server
        .addrs()
        .first()
        .expect("Gagal mendapatkan alamat server")
        .to_owned();
    let server_url = format!("http://{}", server_addr);

    // Jalankan server di background
    tokio::spawn(server.run());

    (db_pool, meili_client, server_url)
}