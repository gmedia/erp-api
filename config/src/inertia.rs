//! See: https://github.com/KaioFelps/inertia-rust/tree/v2/examples/actix_ssr
use inertia_rust::{
    template_resolvers::ViteHBSTemplateResolver,
    Inertia,
    InertiaConfig,
    InertiaError,
    InertiaVersion,
    // SsrClient,
};
use std::io;
use vite_rust::ViteMode;

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

    Inertia::new(
        InertiaConfig::builder()
            .set_url("http://0.0.0.0:8080")
            .set_version(InertiaVersion::Literal(ASSETS_VERSION.get().unwrap()))
            .set_template_resolver(Box::new(resolver))
            // .enable_ssr()
            // .set_ssr_client(SsrClient::new("0.0.0.0", 1000))
            .build(),
    )
}
