use vite_rust::{Vite, ViteConfig};

pub async fn initialize_vite() -> Vite {
    let vite_config = ViteConfig::default()
        .set_manifest_path("path/to/manifest.json")
        // you can add the same client-side entrypoints to vite-rust,
        // so that it won't panic if the manifest file doesn't exist but the
        // development server is running
        .set_entrypoints(vec!["www/app.ts"]);

    match Vite::new(vite_config).await {
        Err(err) => panic!("{}", err),
        Ok(vite) => vite
    }
}