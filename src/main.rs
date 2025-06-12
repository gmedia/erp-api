use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use erp_api::api::v1::{employee, inventory, order};
use erp_api::config::settings::Settings;
use erp_api::db::mysql::init_db_pool;
use erp_api::search::meilisearch::init_meilisearch;
use std::env;
use erp_api::api::openapi::ApiDoc;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let settings = Settings::new(&env::var("APP_ENV").unwrap_or("production".to_string()));
    let db_pool = init_db_pool(&settings.database_url)
        .await
        .expect("Gagal inisialisasi pool database");
    let meili_client = init_meilisearch(&settings.meilisearch_host, &settings.meilisearch_api_key)
        .await
        .expect("Failed to init Meilisearch");

    // Inisialisasi indeks Meilisearch
    let index = meili_client.index("inventory");
    index
        .set_searchable_attributes(&["name"])
        .await
        .expect("Failed to configure Meilisearch index");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(meili_client.clone()))
            .configure(inventory::routes::init_routes)
            .configure(employee::routes::init_routes)
            .configure(order::routes::init_routes)
            .service(Scalar::with_url("/scalar", ApiDoc::openapi()))
    })
    .bind(("0.0.0.0", 8080))? // Mengikat ke semua antarmuka
    .run()
    .await
}