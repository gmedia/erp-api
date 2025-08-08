use std::env;

pub struct Meilisearch {
    pub host: String,
    pub api_key: String,
}

impl Meilisearch {
    pub fn new() -> Self {
        let host =
            env::var("MEILISEARCH_HOST").unwrap_or_else(|_| "http://meilisearch:7700".to_string());
        let api_key = env::var("MEILISEARCH_API_KEY").unwrap_or_else(|_| "masterKey".to_string());

        Meilisearch { host, api_key }
    }
}
