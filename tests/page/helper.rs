use actix_web::{test, web, App};
use config::{app::AppState, inertia::initialize_inertia};
use db::mysql::init_db_pool;
use search::meilisearch::init_meilisearch;

pub async fn setup_test_app() -> impl actix_web::dev::Service {
    dotenvy::dotenv().ok();
    
    let config_db = config::db::Db::new();
    let config_meilisearch = config::meilisearch::Meilisearch::new();
    
    let db_pool = init_db_pool(&config_db.url)
        .await
        .expect("Failed to initialize database pool");
    
    let meili_client = init_meilisearch(&config_meilisearch.host, &config_meilisearch.api_key)
        .await
        .expect("Failed to initialize Meilisearch");
    
    let app_state = AppState {
        db: db_pool,
        meilisearch: meili_client,
        jwt_secret: "test-secret".to_string(),
        jwt_expires_in_seconds: 3600,
        bcrypt_cost: bcrypt::DEFAULT_COST,
        jwt_algorithm: jsonwebtoken::Algorithm::HS256,
    };
    
    let inertia = initialize_inertia().await.unwrap();
    
    test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .app_data(web::Data::new(inertia))
            .configure(page::routes::init_routes),
    )
    .await
}