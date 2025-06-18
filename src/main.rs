use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use serde_json::json;
use dotenv::dotenv;
use api::v1::{auth, employee, inventory, order};
use config::{app::{AppConfig, AppState}, db::Db, meilisearch::Meilisearch};
use db::mysql::init_db_pool;
use search::meilisearch::{init_meilisearch, configure_index};
use std::env;
use api::openapi::ApiDoc;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

async fn healthcheck() -> impl Responder {
    HttpResponse::Ok().json(json!({ "status": "active" }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let env = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
    let config_db = Db::new(&env);
    let config_meilisearch = Meilisearch::new(&env);
    let config_app = AppConfig::new(&env);
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    
    let db_pool = init_db_pool(&config_db.url)
        .await
        .expect("Gagal inisialisasi pool database");
    let meili_client = init_meilisearch(&config_meilisearch.host, &config_meilisearch.api_key)
        .await
        .expect("Failed to init Meilisearch");

    for (index_name, p_key) in &config_app.meilisearch_indexes {
        let pk: Vec<&str> = p_key.iter().map(|s| s.as_str()).collect();
        configure_index(&meili_client, index_name, &pk)
            .await
            .unwrap_or_else(|_| panic!("Failed to configure '{}' index", index_name));
    }

    let app_state = AppState {
        db: db_pool,
        meilisearch: meili_client,
        jwt_secret,
    };

    HttpServer::new(move || {
        App::new()
            .route("/healthcheck", web::get().to(healthcheck))
            .app_data(web::Data::new(app_state.clone()))
            // Register your routes here
            .configure(inventory::routes::init_routes)
            .configure(employee::routes::init_routes)
            .configure(order::routes::init_routes)
            .configure(auth::routes::init_routes)
            .service(Scalar::with_url("/scalar", ApiDoc::openapi()))
    })
    .bind(("0.0.0.0", 8080))? // Mengikat ke semua antarmuka
    .run()
    .await
}