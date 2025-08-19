use actix_session::SessionMiddleware;
use actix_web::{
    App, HttpServer,
    dev::{Server, ServerHandle},
    web, cookie::{Key, SameSite},
};
use config::{db::Db, file_session::FileSessionStore, meilisearch::Meilisearch};
use db::mysql::init_db_pool;
use fake::{Fake, faker::internet::en::SafeEmail};
use reqwest::Client as HttpClient;
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};
use search::{Client, meilisearch::init_meilisearch};
use serde_json::json;
use std::{net::TcpListener, time::Duration};
use tempfile::TempDir;

/// Test page app state that includes page-specific configurations
#[derive(Clone)]
pub struct TestPageAppState {
    pub db: DatabaseConnection,
    pub meilisearch: Client,
    pub jwt_secret: String,
    pub jwt_expires_in_seconds: u64,
    pub bcrypt_cost: u32,
    pub jwt_algorithm: jsonwebtoken::Algorithm,
}

/// Setup a test application specifically for page routes
pub async fn setup_test_page_app(
    jwt_expires_in_seconds: Option<u64>,
    bcrypt_cost: Option<u32>,
    jwt_secret: Option<String>,
    jwt_algorithm: Option<jsonwebtoken::Algorithm>,
) -> (DatabaseConnection, Client, String, ServerHandle, TempDir) {
    dotenvy::dotenv().ok();
    let _ = env_logger::try_init();

    let config_db = Db::new();
    let config_meilisearch = Meilisearch::new();
    let jwt_secret = jwt_secret.unwrap_or_else(|| "test-secret".to_string());

    // Create temporary directory for session storage
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Initialize database
    let db_pool = init_db_pool(&config_db.url)
        .await
        .expect("Failed to initialize database pool");

    // Initialize Meilisearch
    let meili_client = init_meilisearch(&config_meilisearch.host, &config_meilisearch.api_key)
        .await
        .expect("Failed to initialize Meilisearch for tests");

    // Clone for server
    let db_pool_for_server = db_pool.clone();
    let meili_client_for_server = meili_client.clone();

    // Run server on random port
    let listener = TcpListener::bind("0.0.0.0:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    println!("Page test server is listening on port {port}");

    let app_state = TestPageAppState {
        db: db_pool_for_server.clone(),
        meilisearch: meili_client_for_server.clone(),
        jwt_secret: jwt_secret.clone(),
        jwt_expires_in_seconds: jwt_expires_in_seconds.unwrap_or(3600),
        bcrypt_cost: bcrypt_cost.unwrap_or(bcrypt::DEFAULT_COST),
        jwt_algorithm: jwt_algorithm.unwrap_or(jsonwebtoken::Algorithm::HS256),
    };

    let server = run_page_server(app_state, listener).await.unwrap();

    let server_url = format!("http://127.0.0.1:{port}");
    let server_handle = server.handle();

    // Run server in background
    tokio::spawn(server);
    wait_until_server_ready(&server_url).await;

    (db_pool, meili_client, server_url, server_handle, temp_dir)
}

/// Setup test app with clean database for page testing
pub async fn setup_test_page_app_no_data() -> (DatabaseConnection, Client, String, ServerHandle, TempDir) {
    dotenvy::dotenv().ok();
    let _ = env_logger::try_init();

    let config_db = Db::new();
    let config_meilisearch = Meilisearch::new();
    let config_app = config::app::AppConfig::new();
    let jwt_secret = "test-secret".to_string();

    // Create temporary directory for session storage
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Initialize database
    let db_pool = init_db_pool(&config_db.url)
        .await
        .expect("Failed to initialize database pool");

    // Clean tables for testing
    for table in &config_app.tables {
        db_pool
            .execute(Statement::from_string(
                db_pool.get_database_backend(),
                format!("TRUNCATE TABLE `{table}`;"),
            ))
            .await
            .unwrap();
    }

    // Initialize Meilisearch
    let meili_client = init_meilisearch(&config_meilisearch.host, &config_meilisearch.api_key)
        .await
        .expect("Failed to initialize Meilisearch for tests");

    // Clone for server
    let db_pool_for_server = db_pool.clone();
    let meili_client_for_server = meili_client.clone();

    // Run server on random port
    let listener = TcpListener::bind("0.0.0.0:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    println!("Page test server is listening on port {port}");

    let app_state = TestPageAppState {
        db: db_pool_for_server.clone(),
        meilisearch: meili_client_for_server.clone(),
        jwt_secret,
        jwt_expires_in_seconds: 3600,
        bcrypt_cost: bcrypt::DEFAULT_COST,
        jwt_algorithm: jsonwebtoken::Algorithm::HS256,
    };

    let server = run_page_server(app_state, listener).await.unwrap();

    let server_url = format!("http://127.0.0.1:{port}");
    let server_handle = server.handle();

    // Run server in background
    tokio::spawn(server);
    wait_until_server_ready(&server_url).await;

    (db_pool, meili_client, server_url, server_handle, temp_dir)
}

/// Setup test app without state for page testing
pub async fn setup_test_page_app_no_state() -> (String, ServerHandle, TempDir) {
    dotenvy::dotenv().ok();
    let _ = env_logger::try_init();

    // Create temporary directory for session storage
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Run server on random port
    let listener = TcpListener::bind("0.0.0.0:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    println!("Page test server is listening on port {port}");

    use erp_api::healthcheck;
    
    let server = HttpServer::new(move || {
        App::new()
            .route("/healthcheck", web::get().to(healthcheck))
            // Configure page routes
            .configure(page::routes::init_routes)
    })
    .listen(listener)
    .expect("Failed to listen")
    .run();

    let server_url = format!("http://127.0.0.1:{port}");
    let server_handle = server.handle();

    // Run server in background
    tokio::spawn(server);
    wait_until_server_ready(&server_url).await;

    (server_url, server_handle, temp_dir)
}

/// Helper struct for testing page responses
pub struct PageTestClient {
    pub client: HttpClient,
    pub server_url: String,
}

impl PageTestClient {
    pub fn new(server_url: String) -> Self {
        let client = HttpClient::builder()
            .cookie_store(true)
            .build()
            .expect("Failed to build HTTP client");
        
        Self { client, server_url }
    }

    /// Get a page and return the response text
    pub async fn get_page(&self, path: &str) -> Result<String, reqwest::Error> {
        let response = self.client
            .get(format!("{}{}", self.server_url, path))
            .send()
            .await?;
        
        response.text().await
    }

    /// Submit a form and return the response
    pub async fn submit_form(
        &self,
        path: &str,
        form_data: &serde_json::Value,
    ) -> Result<reqwest::Response, reqwest::Error> {
        self.client
            .post(format!("{}{}", self.server_url, path))
            .form(form_data)
            .send()
            .await
    }

    /// Submit JSON data and return the response
    pub async fn submit_json(
        &self,
        path: &str,
        json_data: &serde_json::Value,
    ) -> Result<reqwest::Response, reqwest::Error> {
        self.client
            .post(format!("{}{}", self.server_url, path))
            .json(json_data)
            .send()
            .await
    }

    /// Get session data from response cookies
    pub async fn get_session_data(&self, response: &reqwest::Response) -> Option<String> {
        response
            .cookies()
            .find(|c| c.name() == "actix-session")
            .map(|c| c.value().to_string())
    }
}

/// Helper for testing form submissions
pub struct FormTester {
    pub client: PageTestClient,
}

impl FormTester {
    pub fn new(server_url: String) -> Self {
        Self {
            client: PageTestClient::new(server_url),
        }
    }

    /// Test CSRF token validation
    pub async fn test_csrf_protection(
        &self,
        form_path: &str,
        form_data: &serde_json::Value,
    ) -> Result<bool, reqwest::Error> {
        let response = self.client.submit_form(form_path, form_data).await?;
        Ok(response.status().is_success())
    }

    /// Test form validation errors
    pub async fn test_form_validation(
        &self,
        form_path: &str,
        invalid_data: &serde_json::Value,
    ) -> Result<String, reqwest::Error> {
        let response = self.client.submit_form(form_path, invalid_data).await?;
        response.text().await
    }

    /// Test successful form submission
    pub async fn test_successful_submission(
        &self,
        form_path: &str,
        valid_data: &serde_json::Value,
    ) -> Result<reqwest::Response, reqwest::Error> {
        self.client.submit_form(form_path, valid_data).await
    }
}

/// Helper for testing flash messages
pub struct FlashMessageTester {
    pub client: PageTestClient,
}

impl FlashMessageTester {
    pub fn new(server_url: String) -> Self {
        Self {
            client: PageTestClient::new(server_url),
        }
    }

    /// Check if flash messages are present in response
    pub async fn has_flash_message(&self, response_text: &str, message: &str) -> bool {
        response_text.contains(message)
    }

    /// Check for success flash messages
    pub async fn has_success_message(&self, response_text: &str) -> bool {
        response_text.contains("alert-success") || response_text.contains("success")
    }

    /// Check for error flash messages
    pub async fn has_error_message(&self, response_text: &str) -> bool {
        response_text.contains("alert-danger") || response_text.contains("error")
    }
}

/// Helper for testing Inertia.js responses
pub struct InertiaTester {
    pub client: PageTestClient,
}

impl InertiaTester {
    pub fn new(server_url: String) -> Self {
        Self {
            client: PageTestClient::new(server_url),
        }
    }

    /// Test Inertia page response
    pub async fn test_inertia_response(
        &self,
        path: &str,
        _component: &str,
    ) -> Result<serde_json::Value, reqwest::Error> {
        let response = self.client
            .client
            .get(format!("{}{}", self.client.server_url, path))
            .header("X-Inertia", "true")
            .send()
            .await?;
        
        response.json().await
    }

    /// Test Inertia partial reload
    pub async fn test_inertia_partial_reload(
        &self,
        path: &str,
        _partial_data: &str,
    ) -> Result<serde_json::Value, reqwest::Error> {
        let response = self.client
            .client
            .get(format!("{}{}", self.client.server_url, path))
            .header("X-Inertia", "true")
            .send()
            .await?;
        
        response.json().await
    }
}

/// Get authentication token for API testing within page tests
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

    let token_response: api::v1::auth::models::TokenResponse = response.json().await.unwrap();
    token_response.token
}

/// Wait until server is ready
async fn wait_until_server_ready(server_url: &str) {
    const MAX_RETRIES: usize = 20;
    const DELAY_MS: u64 = 250;

    let client = HttpClient::new();
    let health_url = format!("{server_url}/healthcheck");

    for _ in 0..MAX_RETRIES {
        match client.get(&health_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    return; // Server is ready
                }
            }
            Err(_) => {
                // Still failing, try again later
            }
        }

        tokio::time::sleep(Duration::from_millis(DELAY_MS)).await;
    }

    panic!(
        "Server not ready after waiting {} ms",
        MAX_RETRIES as u64 * DELAY_MS
    );
}

/// Run the page server with proper configuration
async fn run_page_server(
    app_state: TestPageAppState,
    listener: TcpListener,
) -> std::io::Result<Server> {
    use erp_api::healthcheck;
    
    // Generate a secret key for session middleware
    let secret_key = Key::generate();
    
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .route("/healthcheck", web::get().to(healthcheck))
            // Session middleware for page testing
            .wrap(
                SessionMiddleware::builder(
                    FileSessionStore::default(),
                    secret_key.clone(),
                )
                .cookie_secure(false)
                .cookie_same_site(SameSite::Lax)
                .cookie_http_only(true)
                .build()
            )
            // Configure page routes
            .configure(page::routes::init_routes)
    })
    .listen(listener)
    .expect("Failed to listen")
    .run();

    Ok(server)
}

/// Utility to clean up test sessions
pub async fn cleanup_test_sessions(temp_dir: &TempDir) {
    let session_path = temp_dir.path().join("sessions");
    if session_path.exists() {
        let _ = tokio::fs::remove_dir_all(session_path).await;
    }
}

/// Create a test user for page testing
pub async fn create_test_user(
    db_pool: &DatabaseConnection,
    username: &str,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use entity::user;
    use sea_orm::{ActiveModelTrait, Set};

    let hashed_password = bcrypt::hash(password, bcrypt::DEFAULT_COST)?;
    
    let user = user::ActiveModel {
        username: Set(username.to_string()),
        password: Set(hashed_password),
        ..Default::default()
    };
    
    user.insert(db_pool).await?;
    Ok(())
}

/// Helper to extract CSRF token from HTML
pub fn extract_csrf_token(html: &str) -> Option<String> {
    let re = regex::Regex::new(r#"<meta name="csrf-token" content="([^"]+)">"#).unwrap();
    re.captures(html)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
}

/// Helper to test redirect responses
pub async fn test_redirect(
    client: &PageTestClient,
    path: &str,
    expected_location: &str,
) -> Result<bool, reqwest::Error> {
    let response = client.client
        .get(format!("{}{}", client.server_url, path))
        .send()
        .await?;
    
    Ok(response.status().is_redirection() && 
       response.headers().get("location")
           .and_then(|v| v.to_str().ok())
           .map(|loc| loc.contains(expected_location))
           .unwrap_or(false))
}