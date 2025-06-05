use meilisearch_sdk::client::Client;

pub async fn init_meilisearch(host: &str, api_key: &str) -> Result<Client, meilisearch_sdk::errors::Error> {
    let client = Client::new(host, Some(api_key))?;
    Ok(client)
}