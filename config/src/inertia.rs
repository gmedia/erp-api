//! See: https://github.com/KaioFelps/inertia-rust/tree/v2/examples/actix_ssr

use inertia_rust::{
    template_resolvers::ViteHBSTemplateResolver, Inertia, InertiaConfig, InertiaError, InertiaVersion,
    // SsrClient,
};
use std::io;
use vite_rust::ViteMode;
use std::env;

use super::vite::{initialize_vite, ASSETS_VERSION};

pub async fn initialize_inertia() -> Result<Inertia, io::Error> {
    let vite = initialize_vite().await;

    let dev_mode = *vite.mode() == ViteMode::Development;

    let resolver = ViteHBSTemplateResolver::builder()
        .set_vite(vite)
        .set_template_path("www/root.hbs")
        .set_dev_mode(dev_mode)
        .build()
        .map_err(InertiaError::to_io_error)?;

    let url = env::var("APP_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());

    Inertia::new(
        InertiaConfig::builder()
            .set_url(url)
            .set_version(InertiaVersion::Literal(ASSETS_VERSION.get().unwrap()))
            .set_template_resolver(Box::new(resolver))
            // .enable_ssr()
            // .set_ssr_client(SsrClient::new("127.0.0.1", 1000))
            .build(),
    )
}
