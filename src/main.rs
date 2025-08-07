use actix_web::{
    App, HttpServer, web,    
    cookie::{Key, SameSite},
};
use api::{
    openapi::{ApiDoc},
    v1::{auth, employee, inventory, order},
};
use config::{
    app::{AppConfig, AppState},
    db::Db,
    meilisearch::Meilisearch,
    inertia::initialize_inertia,
    file_session::FileSessionStore,
    vite::ASSETS_VERSION,
};
use db::mysql::init_db_pool;
use search::meilisearch::{configure_index, init_meilisearch};
use std::env;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};
use page::middlewares::{
    reflash_temporary_session::ReflashTemporarySessionMiddleware,
    garbage_collector::GarbageCollectorMiddleware,
};
use actix_session::{SessionExt, SessionMiddleware};
use inertia_rust::{
    actix::InertiaMiddleware, hashmap, prop_resolver, InertiaProp, IntoInertiaPropResult,
};
use serde_json::{Map, Value};
use std::{sync::Arc, net::TcpListener};
use erp_api::healthcheck;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let env = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
    let config_db = Db::new(&env);
    let config_meilisearch = Meilisearch::new(&env);
    let config_app = AppConfig::new(&env);
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set.");

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
            .unwrap_or_else(|_| panic!("Failed to configure '{index_name}' index"));
    }

    let app_state = AppState {
        db: db_pool,
        meilisearch: meili_client,
        jwt_secret,
        jwt_expires_in_seconds: 3600, // Default to 1 hour
        bcrypt_cost: bcrypt::DEFAULT_COST,
        jwt_algorithm: jsonwebtoken::Algorithm::HS256,
    };

    // starts a Inertia manager instance.
    let inertia = initialize_inertia().await?;
    let inertia = web::Data::new(inertia);

    let key = Key::from(
        env::var("APP_KEY")
            .expect("You must provide a valid APP_KEY environment variable.")
            .as_bytes(),
    );

    let storage = FileSessionStore::default();

    let rust_env = env::var("RUST_ENV").unwrap_or_else(|_| "production".to_string());
    let use_secure_cookie = rust_env.as_str() == "production";

    let listener = TcpListener::bind("0.0.0.0:8080")
        .expect("Failed to bind port 8080");

    // We retrieve the port assigned to us by the OS
    let port = listener.local_addr().unwrap().port();
    println!("Server is listening on port {}", port);

    HttpServer::new(move || {
        App::new()
            .route("/healthcheck", web::get().to(healthcheck))
            // Config for api
            .service(Scalar::with_url("/scalar", ApiDoc::openapi()))
            .configure(inventory::routes::init_routes)
            .configure(employee::routes::init_routes)
            .configure(order::routes::init_routes)
            .configure(auth::routes::init_routes)
            .app_data(web::Data::new(app_state.clone()))
            // Config for page
            .service(
                web::scope("/page")
                    .wrap(GarbageCollectorMiddleware::new())
                    .wrap(                
                        InertiaMiddleware::new().with_shared_props(Arc::new(move |req| {
                            let flash = req.get_session()
                                .get::<Map<String, Value>>("_flash")
                                .unwrap_or_default()
                                .unwrap_or_default();
                            
                            Box::pin(async move {
                                hashmap![
                                    "version" => InertiaProp::always("0.1.0"),
                                    "assetsVersion" => InertiaProp::lazy(prop_resolver!({ ASSETS_VERSION.get().unwrap().into_inertia_value() })),
                                    "flash" => InertiaProp::always(flash)
                                ]
                            })
                        })),
                    )
                    .wrap(ReflashTemporarySessionMiddleware::new())
                    .wrap(
                        SessionMiddleware::builder(storage.clone(), key.clone())
                            .cookie_domain(Some(env::var("DOMAIN").unwrap_or_else(|_| "localhost".to_string())))
                            .cookie_http_only(true)
                            .cookie_same_site(SameSite::Strict)
                            .cookie_name(env::var("SESSION_COOKIE_NAME").unwrap_or_else(|_| "rust_session_id".to_string()))
                            .cookie_secure(use_secure_cookie)
                            .build()
                    )
                    .configure(page::routes::init_routes)
                    .app_data(inertia.clone())
            )
            .service(actix_files::Files::new("public/", "./public").prefer_utf8(true))
    })
    // .bind(("0.0.0.0", 8080))? // Mengikat ke semua antarmuka
    .listen(listener)?
    .run()
    .await
}
