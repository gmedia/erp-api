use meilisearch_sdk::client::Client;

pub async fn init_meilisearch(
    host: &str,
    api_key: &str,
) -> Result<Client, meilisearch_sdk::errors::Error> {
    let client = Client::new(host, Some(api_key))?;
    Ok(client)
}

pub async fn configure_index(
    client: &Client,
    index_name: &str,
    searchable_attributes: &[&str],
) -> Result<(), meilisearch_sdk::errors::Error> {
    let index = client.index(index_name);
    index
        .set_searchable_attributes(searchable_attributes)
        .await?;
    Ok(())
}
