use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use serde_json::json;
use dotenv::dotenv;
use api::v1::{employee, inventory, order};
use config::db::Db;
use config::meilisearch::Meilisearch;
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

    let env = env::var("APP_ENV").unwrap_or("production".to_string());
    let config_db = Db::new(&env);
    let config_meilisearch = Meilisearch::new(&env);
    
    let db_pool = init_db_pool(&config_db.url)
        .await
        .expect("Gagal inisialisasi pool database");
    let meili_client = init_meilisearch(&config_meilisearch.host, &config_meilisearch.api_key)
        .await
        .expect("Failed to init Meilisearch");

    // Register your Meilisearch indeks for initialization here
    configure_index(&meili_client, "inventory", &["name"])
        .await
        .expect("Failed to configure 'inventory' index");

    configure_index(&meili_client, "orders", &["item", "customer_name"])
        .await
        .expect("Failed to configure 'orders' index");

    HttpServer::new(move || {
        App::new()
            .route("/healthcheck", web::get().to(healthcheck))
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(meili_client.clone()))
            // Register your routes here
            .configure(inventory::routes::init_routes)
            .configure(employee::routes::init_routes)
            .configure(order::routes::init_routes)
            .service(Scalar::with_url("/scalar", ApiDoc::openapi()))
    })
    .bind(("0.0.0.0", 8080))? // Mengikat ke semua antarmuka
    .run()
    .await
}