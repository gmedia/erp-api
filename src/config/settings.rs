use std::env;

pub struct Settings {
    pub database_url: String,
    pub meilisearch_host: String,
    pub meilisearch_api_key: String,
}

impl Settings {
    pub fn new(_env: &str) -> Self {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "mysql://user:password@mariadb:3306/erp_db".to_string());
        let meilisearch_host = env::var("MEILISEARCH_HOST")
            .unwrap_or_else(|_| "http://meilisearch:7700".to_string());
        let meilisearch_api_key = env::var("MEILISEARCH_API_KEY")
            .unwrap_or_else(|_| "masterKey".to_string());

        Settings {
            database_url,
            meilisearch_host,
            meilisearch_api_key,
        }
    }
}