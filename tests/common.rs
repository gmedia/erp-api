use actix_web::{
    App, HttpServer,
    dev::{Server, ServerHandle},
    web,
};
use api::v1::auth::models::TokenResponse;
use api::v1::{auth, employee, inventory, order};
use config::{app::AppState, db::Db, meilisearch::Meilisearch};
use db::mysql::init_db_pool;
use erp_api::healthcheck;
use fake::{Fake, faker::internet::en::SafeEmail};
use reqwest::Client as HttpClient;
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};
use search::{Client, meilisearch::init_meilisearch};
use serde_json::json;
use std::{net::TcpListener, time::Duration};

pub async fn setup_test_app(
    jwt_expires_in_seconds: Option<u64>,
    bcrypt_cost: Option<u32>,
    jwt_secret: Option<String>,
    jwt_algorithm: Option<jsonwebtoken::Algorithm>,
) -> (DatabaseConnection, Client, String, ServerHandle) {
    dotenvy::dotenv().ok();
    let _ = env_logger::try_init();

    let config_db = Db::new("test");
    let config_meilisearch = Meilisearch::new("test");
    let jwt_secret = jwt_secret.unwrap_or_else(|| "test-secret".to_string());

    // Inisialisasi database
    let db_pool = init_db_pool(&config_db.url)
        .await
        .expect("Gagal inisialisasi pool database");

    // Inisialisasi Meilisearch
    let meili_client = init_meilisearch(&config_meilisearch.host, &config_meilisearch.api_key)
        .await
        .expect("Gagal inisialisasi Meilisearch untuk tes");

    // Clone db_pool and meili_client for moving into the closure
    let db_pool_for_server = db_pool.clone();
    let meili_client_for_server = meili_client.clone();

    // Jalankan server di port acak
    let listener = TcpListener::bind("0.0.0.0:0").expect("Failed to bind random port");

    // We retrieve the port assigned to us by the OS
    let port = listener.local_addr().unwrap().port();
    println!("Server is listening on port {port}");

    let app_state = AppState {
        db: db_pool_for_server.clone(),
        meilisearch: meili_client_for_server.clone(),
        jwt_secret: jwt_secret.clone(),
        jwt_expires_in_seconds: jwt_expires_in_seconds.unwrap_or(3600),
        bcrypt_cost: bcrypt_cost.unwrap_or(bcrypt::DEFAULT_COST),
        jwt_algorithm: jwt_algorithm.unwrap_or(jsonwebtoken::Algorithm::HS256),
    };

    let server = run(app_state, listener).await.unwrap();

    let server_url = format!("http://127.0.0.1:{port}");
    let server_handle = server.handle();

    // Jalankan server di background
    tokio::spawn(server);
    wait_until_server_ready(&server_url).await;

    (db_pool, meili_client, server_url, server_handle)
}

#[allow(dead_code)]
pub async fn setup_test_app_no_data() -> (DatabaseConnection, Client, String, ServerHandle) {
    dotenvy::dotenv().ok();
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
                format!("TRUNCATE TABLE `{table}`;"),
            ))
            .await
            .unwrap();
    }

    // Inisialisasi Meilisearch
    let meili_client = init_meilisearch(&config_meilisearch.host, &config_meilisearch.api_key)
        .await
        .expect("Gagal inisialisasi Meilisearch untuk tes");

    // Clone db_pool and meili_client for moving into the closure
    let db_pool_for_server = db_pool.clone();
    let meili_client_for_server = meili_client.clone();

    // Jalankan server di port acak
    let listener = TcpListener::bind("0.0.0.0:0").expect("Failed to bind random port");

    // We retrieve the port assigned to us by the OS
    let port = listener.local_addr().unwrap().port();
    println!("Server is listening on port {port}");

    let app_state = AppState {
        db: db_pool_for_server.clone(),
        meilisearch: meili_client_for_server.clone(),
        jwt_secret: jwt_secret.clone(),
        jwt_expires_in_seconds: 3600,
        bcrypt_cost: bcrypt::DEFAULT_COST,
        jwt_algorithm: jsonwebtoken::Algorithm::HS256,
    };

    let server = run(app_state, listener).await.unwrap();

    let server_url = format!("http://127.0.0.1:{port}");
    let server_handle = server.handle();

    // Jalankan server di background
    tokio::spawn(server);
    wait_until_server_ready(&server_url).await;

    (db_pool, meili_client, server_url, server_handle)
}

#[allow(dead_code)]
pub async fn setup_test_app_no_state() -> (DatabaseConnection, Client, String, ServerHandle) {
    dotenvy::dotenv().ok();
    let _ = env_logger::try_init();

    let config_db = Db::new("test");
    let config_meilisearch = Meilisearch::new("test");

    // Inisialisasi database
    let db_pool = init_db_pool(&config_db.url)
        .await
        .expect("Gagal inisialisasi pool database");

    // Inisialisasi Meilisearch
    let meili_client = init_meilisearch(&config_meilisearch.host, &config_meilisearch.api_key)
        .await
        .expect("Gagal inisialisasi Meilisearch untuk tes");

    // Jalankan server di port acak
    let listener = TcpListener::bind("0.0.0.0:0").expect("Failed to bind random port");

    // We retrieve the port assigned to us by the OS
    let port = listener.local_addr().unwrap().port();
    println!("Server is listening on port {port}");

    let server = HttpServer::new(move || {
        App::new()
            // app_data sengaja dihilangkan untuk pengujian ini
            .route("/healthcheck", web::get().to(healthcheck))
            .configure(inventory::routes::init_routes)
            .configure(employee::routes::init_routes)
            .configure(order::routes::init_routes)
            .configure(auth::routes::init_routes)
    })
    .listen(listener)
    .expect("Failed to listen")
    .run();

    let server_url = format!("http://127.0.0.1:{port}");
    let server_handle = server.handle();

    // Jalankan server di background
    tokio::spawn(server);
    wait_until_server_ready(&server_url).await;

    (db_pool, meili_client, server_url, server_handle)
}

#[allow(dead_code)]
pub async fn setup_test_app_with_meili_error() -> (DatabaseConnection, Client, String, ServerHandle)
{
    dotenvy::dotenv().ok();
    let _ = env_logger::try_init();

    let config_db = Db::new("test");
    let mut config_meilisearch = Meilisearch::new("test");
    config_meilisearch.host = "http://localhost:9999".to_string(); // Bad url
    let jwt_secret = "test-secret".to_string();

    // Inisialisasi database
    let db_pool = init_db_pool(&config_db.url)
        .await
        .expect("Gagal inisialisasi pool database");

    // Inisialisasi Meilisearch
    let meili_client = init_meilisearch(&config_meilisearch.host, &config_meilisearch.api_key)
        .await
        .expect("Gagal inisialisasi Meilisearch untuk tes");

    // Clone db_pool and meili_client for moving into the closure
    let db_pool_for_server = db_pool.clone();
    let meili_client_for_server = meili_client.clone();

    // Jalankan server di port acak
    let listener = TcpListener::bind("0.0.0.0:0").expect("Failed to bind random port");

    // We retrieve the port assigned to us by the OS
    let port = listener.local_addr().unwrap().port();
    println!("Server is listening on port {port}");

    let app_state = AppState {
        db: db_pool_for_server.clone(),
        meilisearch: meili_client_for_server.clone(),
        jwt_secret: jwt_secret.clone(),
        jwt_expires_in_seconds: 3600,
        bcrypt_cost: bcrypt::DEFAULT_COST,
        jwt_algorithm: jsonwebtoken::Algorithm::HS256,
    };

    let server = run(app_state, listener).await.unwrap();

    let server_url = format!("http://127.0.0.1:{port}");
    let server_handle = server.handle();

    // Jalankan server di background
    tokio::spawn(server);
    wait_until_server_ready(&server_url).await;

    (db_pool, meili_client, server_url, server_handle)
}

#[allow(dead_code)]
pub async fn get_auth_token(
    client: &HttpClient,
    server_url: &str,
    db_pool: &DatabaseConnection,
) -> String {
    let username: String = SafeEmail().fake();
    let password = "password123";

    // Clean user
    let backend: sea_orm::DatabaseBackend = db_pool.get_database_backend();
    let _ = db_pool
        .execute(Statement::from_string(
            backend,
            format!("DELETE FROM user where username = '{username}'"),
        ))
        .await;

    let register_req = json!({
        "username": username,
        "password": password,
    });

    client
        .post(format!("{server_url}/v1/auth/register"))
        .json(&register_req)
        .send()
        .await
        .unwrap();

    let login_req = json!({
        "username": username,
        "password": password,
    });

    let response = client
        .post(format!("{server_url}/v1/auth/login"))
        .json(&login_req)
        .send()
        .await
        .unwrap();

    let token_response: TokenResponse = response.json().await.unwrap();
    token_response.token
}

async fn wait_until_server_ready(server_url: &str) {
    const MAX_RETRIES: usize = 20;
    const DELAY_MS: u64 = 250;

    let client = HttpClient::new();
    let health_url = format!("{server_url}/healthcheck");

    for _ in 0..MAX_RETRIES {
        match client.get(&health_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    return; // Server sudah siap
                }
            }
            Err(_) => {
                // Masih gagal, coba lagi nanti
            }
        }

        tokio::time::sleep(Duration::from_millis(DELAY_MS)).await;
    }

    panic!(
        "Server tidak siap setelah menunggu {} ms",
        MAX_RETRIES as u64 * DELAY_MS
    );
}

async fn run(app_state: AppState, listener: TcpListener) -> std::io::Result<Server> {
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            // Register your routes here
            .route("/healthcheck", web::get().to(healthcheck))
            .configure(inventory::routes::init_routes)
            .configure(employee::routes::init_routes)
            .configure(order::routes::init_routes)
            .configure(auth::routes::init_routes)
    })
    .listen(listener)
    .expect("Failed to listen")
    .run();

    Ok(server)
}
