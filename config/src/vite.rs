//! See: https://github.com/KaioFelps/inertia-rust/tree/v2/examples/actix_ssr

use std::sync::OnceLock;
use vite_rust::{Vite, ViteConfig};

pub static ASSETS_VERSION: OnceLock<&str> = OnceLock::new();

pub async fn initialize_vite() -> Vite {
    let vite_config = ViteConfig::default()
        .set_manifest_path("public/bundle/manifest.json")
        // so that it won't need manifest when development server is running
        .set_entrypoints(vec!["www/app.tsx"])
        // prefix every assets path with "bundle" segment, so that the preload tags
        // help loading the page faster!!
        .set_prefix("/bundle");

    match Vite::new(vite_config).await {
        Err(err) => panic!("{}", err),
        Ok(vite) => {
            let _ = ASSETS_VERSION.set(vite.get_hash().unwrap_or("development").to_string().leak());
            vite
        }
    }
}
