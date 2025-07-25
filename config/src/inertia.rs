use super::vite::initialize_vite;
use inertia_rust::{
    template_resolvers::ViteHBSTemplateResolver, Inertia, InertiaConfig, InertiaError,
    InertiaVersion,
};
use std::io;
use vite_rust::ViteMode;

pub async fn initialize_inertia() -> Result<Inertia, io::Error> {
    let vite = initialize_vite().await;
    let version = vite.get_hash().unwrap_or("development").to_string();
    let dev_mode = *vite.mode() == ViteMode::Development;

    let resolver = ViteHBSTemplateResolver::builder()
        .set_vite(vite)
        .set_template_path("www/root.hbs") // the path to your root handlebars template
        .set_dev_mode(dev_mode)
        .build()
        .map_err(InertiaError::to_io_error)?;


    Inertia::new(
        InertiaConfig::builder()
            .set_url("http://localhost:8080")
            .set_version(InertiaVersion::Literal(version))
            .set_template_resolver(Box::new(resolver))
            .build(),
    )
}