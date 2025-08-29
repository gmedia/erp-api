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
use sea_orm::DatabaseConnection;
use search::{Client, meilisearch::init_meilisearch};
use std::{env, net::TcpListener, time::Duration, sync::Arc};
// Entity imports are moved to the test_db_utils module
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
            TestError::DatabaseInit(msg) => write!(f, "Database initialization failed: {msg}"),
            TestError::MeilisearchInit(msg) => write!(f, "Meilisearch initialization failed: {msg}"),
            TestError::ServerStartup(msg) => write!(f, "Server startup failed: {msg}"),
            TestError::ServerTimeout => write!(f, "Server not ready after timeout"),
        }
    }
}

impl std::error::Error for TestError {}

/// Safe database operations for testing
pub mod test_db_utils {
    use super::*;
    use sea_orm::{DatabaseConnection, DeleteResult, QueryFilter, ColumnTrait, TransactionTrait, PaginatorTrait, EntityTrait};
    use entity::prelude::{Order, Employee, User, Inventory};

    /// Clean all test data from specific tables using Entity-based deletion
    pub async fn clean_all_tables(db: &DatabaseConnection) -> Result<(), TestError> {
        // Delete in dependency-aware order to avoid foreign key constraints
        // Order table depends on other tables, so delete first
        Order::delete_many()
            .exec(db)
            .await
            .map_err(|e| TestError::DatabaseInit(format!("Failed to clean orders: {e}")))?;
        
        // Then delete other tables
        Inventory::delete_many()
            .exec(db)
            .await
            .map_err(|e| TestError::DatabaseInit(format!("Failed to clean inventory: {e}")))?;
        
        Employee::delete_many()
            .exec(db)
            .await
            .map_err(|e| TestError::DatabaseInit(format!("Failed to clean employees: {e}")))?;
        
        User::delete_many()
            .exec(db)
            .await
            .map_err(|e| TestError::DatabaseInit(format!("Failed to clean users: {e}")))?;

        Ok(())
    }

    /// Delete specific user by username safely
    pub async fn delete_user_by_username(
        db: &DatabaseConnection,
        username: &str
    ) -> Result<DeleteResult, TestError> {
        User::delete_many()
            .filter(entity::user::Column::Username.eq(username))
            .exec(db)
            .await
            .map_err(|e| TestError::DatabaseInit(format!("Failed to delete user: {e}")))
    }

    /// Check if user exists by username
    pub async fn user_exists(
        db: &DatabaseConnection,
        username: &str
    ) -> Result<bool, TestError> {
        User::find()
            .filter(entity::user::Column::Username.eq(username))
            .one(db)
            .await
            .map(|opt| opt.is_some())
            .map_err(|e| TestError::DatabaseInit(format!("Failed to check user existence: {e}")))
    }

    /// Clean tables within a transaction for test isolation
    pub async fn clean_tables_with_transaction(
        db: &DatabaseConnection
    ) -> Result<(), TestError> {
        let txn = db.begin().await
            .map_err(|e| TestError::DatabaseInit(format!("Failed to start transaction: {e}")))?;
        
        // Delete in dependency-aware order to avoid foreign key constraints
        let result = async {
            // Order table depends on other tables, so delete first
            Order::delete_many().exec(&txn).await?;
            
            // Then delete other tables
            Inventory::delete_many().exec(&txn).await?;
            Employee::delete_many().exec(&txn).await?;
            User::delete_many().exec(&txn).await?;
            
            Ok::<(), sea_orm::DbErr>(())
        }.await;

        match result {
            Ok(_) => {
                txn.commit().await
                    .map_err(|e| TestError::DatabaseInit(format!("Failed to commit transaction: {e}")))?;
                Ok(())
            }
            Err(e) => {
                txn.rollback().await
                    .map_err(|e| TestError::DatabaseInit(format!("Failed to rollback transaction: {e}")))?;
                Err(TestError::DatabaseInit(format!("Transaction failed: {e}")))
            }
        }
    }

    /// Batch delete users by usernames
    pub async fn delete_users_batch(
        db: &DatabaseConnection,
        usernames: &[&str]
    ) -> Result<DeleteResult, TestError> {
        User::delete_many()
            .filter(entity::user::Column::Username.is_in(usernames.to_vec()))
            .exec(db)
            .await
            .map_err(|e| TestError::DatabaseInit(format!("Failed to batch delete users: {e}")))
    }

    /// Get count of records in a table for verification
    pub async fn get_table_counts(db: &DatabaseConnection) -> Result<(u64, u64, u64, u64), TestError> {
        let user_count = User::find().count(db).await
            .map_err(|e| TestError::DatabaseInit(format!("Failed to count users: {e}")))?;
            
        let employee_count = Employee::find().count(db).await
            .map_err(|e| TestError::DatabaseInit(format!("Failed to count employees: {e}")))?;
            
        let inventory_count = Inventory::find().count(db).await
            .map_err(|e| TestError::DatabaseInit(format!("Failed to count inventory: {e}")))?;
            
        let order_count = Order::find().count(db).await
            .map_err(|e| TestError::DatabaseInit(format!("Failed to count orders: {e}")))?;
            
        Ok((user_count, employee_count, inventory_count, order_count))
    }
}

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

impl Default for TestAppBuilder {
    fn default() -> Self {
        Self::new()
    }
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
            test_db_utils::clean_all_tables(&db).await?;
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

pub async fn get_auth_token(
    client: &HttpClient,
    server_url: &str,
    db_pool: &DatabaseConnection,
) -> String {
    let username: String = SafeEmail().fake();
    let password = "password123";

    // Clean user safely using Entity
    let _ = test_db_utils::delete_user_by_username(db_pool, &username).await;

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