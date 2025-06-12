use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use erp_api::api::v1::{employee, inventory, order};
use erp_api::config::settings::Settings;
use erp_api::db::mysql::init_db_pool;
use erp_api::search::meilisearch::init_meilisearch;
use std::env;
use utoipa::{OpenApi, Modify};
use utoipa_scalar::{Scalar, Servable};

#[derive(OpenApi)]
#[openapi(
    paths(
        erp_api::api::v1::inventory::handlers::create_item,
        erp_api::api::v1::inventory::handlers::search_items,
        erp_api::api::v1::employee::handlers::create_employee,
        erp_api::api::v1::order::handlers::create_order,
    ),
    components(
        schemas(
            erp_api::api::v1::inventory::models::InventoryItem,
            erp_api::api::v1::inventory::models::CreateInventoryItem,
            erp_api::api::v1::employee::models::Employee,
            erp_api::api::v1::employee::models::CreateEmployee,
            erp_api::api::v1::order::models::Order,
            erp_api::api::v1::order::models::CreateOrder,
        )
    ),
    modifiers(&SecurityAddon)
)]
struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearerAuth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::Http::new(utoipa::openapi::security::HttpAuthScheme::Bearer),
                ),
            )
        }
    }
}

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