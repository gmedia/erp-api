use actix_web::{
    App, HttpServer,
    dev::{Server, ServerHandle},
    web,
    cookie::{Key, SameSite},
};
use api::v1::auth::models::TokenResponse;
use api::v1::{auth, employee, inventory, order};
use config::{
    app::AppState,
    db::Db,
    meilisearch::Meilisearch,
    inertia::initialize_inertia,
    file_session::FileSessionStore,
    vite::ASSETS_VERSION,
};
use db::mysql::init_db_pool;
use erp_api::healthcheck;
use fake::{Fake, faker::internet::en::SafeEmail};
use reqwest::Client as HttpClient;
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};
use search::{Client, meilisearch::init_meilisearch};
use std::{env, net::TcpListener, time::Duration, sync::Arc};
use page::middlewares::{
    garbage_collector::GarbageCollectorMiddleware,
    reflash_temporary_session::ReflashTemporarySessionMiddleware,
};
use inertia_rust::{
    InertiaProp, IntoInertiaPropResult, actix::InertiaMiddleware, hashmap, prop_resolver,
};
use serde_json::{json, Map, Value};
use actix_session::{SessionExt, SessionMiddleware};

#[derive(Debug)]
pub enum TestError {
    DatabaseInit(String),
    MeilisearchInit(String),
    ServerStartup(String),
    ServerTimeout,
}

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestError::DatabaseInit(msg) => write!(f, "Database initialization failed: {}", msg),
            TestError::MeilisearchInit(msg) => write!(f, "Meilisearch initialization failed: {}", msg),
            TestError::ServerStartup(msg) => write!(f, "Server startup failed: {}", msg),
            TestError::ServerTimeout => write!(f, "Server not ready after timeout"),
        }
    }
}

impl std::error::Error for TestError {}

pub struct TestApp {
    pub db: DatabaseConnection,
    pub meilisearch: Client,
    pub server_url: String,
    pub server_handle: ServerHandle,
}

pub struct TestAppBuilder {
    jwt_expires_in_seconds: Option<u64>,
    bcrypt_cost: Option<u32>,
    jwt_secret: Option<String>,
    jwt_algorithm: Option<jsonwebtoken::Algorithm>,
    clear_tables: bool,
    skip_app_state: bool,
    meili_host: Option<String>,
}

impl TestAppBuilder {
    pub fn new() -> Self {
        Self {
            jwt_expires_in_seconds: None,
            bcrypt_cost: None,
            jwt_secret: None,
            jwt_algorithm: None,
            clear_tables: false,
            skip_app_state: false,
            meili_host: None,
        }
    }

    pub fn jwt_expires_in_seconds(mut self, seconds: u64) -> Self {
        self.jwt_expires_in_seconds = Some(seconds);
        self
    }

    pub fn bcrypt_cost(mut self, cost: u32) -> Self {
        self.bcrypt_cost = Some(cost);
        self
    }

    pub fn jwt_secret(mut self, secret: String) -> Self {
        self.jwt_secret = Some(secret);
        self
    }

    pub fn jwt_algorithm(mut self, algorithm: jsonwebtoken::Algorithm) -> Self {
        self.jwt_algorithm = Some(algorithm);
        self
    }

    pub fn clear_tables(mut self) -> Self {
        self.clear_tables = true;
        self
    }

    pub fn skip_app_state(mut self) -> Self {
        self.skip_app_state = true;
        self
    }

    pub fn meili_host(mut self, host: String) -> Self {
        self.meili_host = Some(host);
        self
    }

    pub async fn build(self) -> Result<TestApp, TestError> {
        dotenvy::dotenv().ok();
        let _ = env_logger::try_init();

        let config_db = Db::new();
        let mut config_meilisearch = Meilisearch::new();
        
        if let Some(ref host) = self.meili_host {
            config_meilisearch.host = host.clone();
        }

        // Initialize database
        let db = init_db_pool(&config_db.url)
            .await
            .map_err(|e| TestError::DatabaseInit(e.to_string()))?;

        // Clear tables if requested
        if self.clear_tables {
            let config_app = config::app::AppConfig::new();
            for table in &config_app.tables {
                db.execute(Statement::from_string(
                    db.get_database_backend(),
                    format!("TRUNCATE TABLE `{table}`;"),
                ))
                .await
                .map_err(|e| TestError::DatabaseInit(e.to_string()))?;
            }
        }

        // Initialize Meilisearch
        let meilisearch = init_meilisearch(&config_meilisearch.host, &config_meilisearch.api_key)
            .await
            .map_err(|e| TestError::MeilisearchInit(e.to_string()))?;

        // Start server
        let listener = TcpListener::bind("0.0.0.0:0")
            .map_err(|e| TestError::ServerStartup(e.to_string()))?;
        let port = listener.local_addr()
            .map_err(|e| TestError::ServerStartup(e.to_string()))?
            .port();
        
        let server_url = format!("http://127.0.0.1:{port}");

        let server = if self.skip_app_state {
            self.create_server_without_state(listener).await?
        } else {
            self.create_server_with_state(listener, db.clone(), meilisearch.clone()).await?
        };

        let server_handle = server.handle();
        tokio::spawn(server);
        self.wait_until_server_ready(&server_url).await?;

        Ok(TestApp {
            db,
            meilisearch,
            server_url,
            server_handle,
        })
    }

    async fn create_server_with_state(
        &self,
        listener: TcpListener,
        db: DatabaseConnection,
        meilisearch: Client,
    ) -> Result<Server, TestError> {
        let app_state = AppState {
            db,
            meilisearch,
            jwt_secret: self.jwt_secret.clone().unwrap_or_else(|| "test-secret".to_string()),
            jwt_expires_in_seconds: self.jwt_expires_in_seconds.unwrap_or(3600),
            bcrypt_cost: self.bcrypt_cost.unwrap_or(bcrypt::DEFAULT_COST),
            jwt_algorithm: self.jwt_algorithm.unwrap_or(jsonwebtoken::Algorithm::HS256),
        };

        run(app_state, listener).await
            .map_err(|e| TestError::ServerStartup(e.to_string()))
    }

    async fn create_server_without_state(
        &self,
        listener: TcpListener,
    ) -> Result<Server, TestError> {
        let server = HttpServer::new(move || {
            App::new()
                .route("/healthcheck", web::get().to(healthcheck))
                .configure(inventory::routes::init_routes)
                .configure(employee::routes::init_routes)
                .configure(order::routes::init_routes)
                .configure(auth::routes::init_routes)
        })
        .listen(listener)
        .map_err(|e| TestError::ServerStartup(e.to_string()))?
        .run();

        Ok(server)
    }

    async fn wait_until_server_ready(&self, server_url: &str) -> Result<(), TestError> {
        const MAX_RETRIES: usize = 20;
        const DELAY_MS: u64 = 250;

        let client = HttpClient::new();
        let health_url = format!("{server_url}/healthcheck");

        for _ in 0..MAX_RETRIES {
            match client.get(&health_url).send().await {
                Ok(response) if response.status().is_success() => return Ok(()),
                _ => tokio::time::sleep(Duration::from_millis(DELAY_MS)).await,
            }
        }

        Err(TestError::ServerTimeout)
    }
}

// Backward compatibility functions
pub async fn setup_test_app(
    jwt_expires_in_seconds: Option<u64>,
    bcrypt_cost: Option<u32>,
    jwt_secret: Option<String>,
    jwt_algorithm: Option<jsonwebtoken::Algorithm>,
) -> (DatabaseConnection, Client, String, ServerHandle) {
    let mut builder = TestAppBuilder::new();
    
    if let Some(exp) = jwt_expires_in_seconds {
        builder = builder.jwt_expires_in_seconds(exp);
    }
    if let Some(cost) = bcrypt_cost {
        builder = builder.bcrypt_cost(cost);
    }
    if let Some(secret) = jwt_secret {
        builder = builder.jwt_secret(secret);
    }
    if let Some(alg) = jwt_algorithm {
        builder = builder.jwt_algorithm(alg);
    }

    let app = builder.build().await.expect("Failed to setup test app");
    (app.db, app.meilisearch, app.server_url, app.server_handle)
}

pub async fn setup_test_app_no_data() -> (DatabaseConnection, Client, String, ServerHandle) {
    let app = TestAppBuilder::new()
        .clear_tables()
        .build()
        .await
        .expect("Failed to setup test app");
    (app.db, app.meilisearch, app.server_url, app.server_handle)
}

pub async fn setup_test_app_no_state() -> (DatabaseConnection, Client, String, ServerHandle) {
    let app = TestAppBuilder::new()
        .skip_app_state()
        .build()
        .await
        .expect("Failed to setup test app");
    (app.db, app.meilisearch, app.server_url, app.server_handle)
}

pub async fn setup_test_app_with_meili_error() -> (DatabaseConnection, Client, String, ServerHandle) {
    let app = TestAppBuilder::new()
        .meili_host("http://localhost:9999".to_string())
        .build()
        .await
        .expect("Failed to setup test app");
    (app.db, app.meilisearch, app.server_url, app.server_handle)
}

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

async fn run(app_state: AppState, listener: TcpListener) -> std::io::Result<Server> {
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

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            // Register your routes here
            .route("/healthcheck", web::get().to(healthcheck))
            .configure(inventory::routes::init_routes)
            .configure(employee::routes::init_routes)
            .configure(order::routes::init_routes)
            .configure(auth::routes::init_routes)
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
    })
    .listen(listener)
    .expect("Failed to listen")
    .run();

    Ok(server)
}