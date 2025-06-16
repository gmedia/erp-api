use utoipa::{
    Modify,
    OpenApi,
};

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearerAuth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::Http::new(utoipa::openapi::security::HttpAuthScheme::Bearer),
                ),
            )
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        // Register your handlers here
        crate::api::v1::inventory::handlers::create_item::create_item,
        crate::api::v1::inventory::handlers::search_items::search_items,
        crate::api::v1::inventory::handlers::update_item::update_item,
        crate::api::v1::inventory::handlers::delete_item::delete_item,
        crate::api::v1::employee::handlers::create_employee,
        crate::api::v1::order::handlers::create_order,
    ),
    components(
        schemas(
            // Register your models here
            crate::api::v1::inventory::models::InventoryItem,
            crate::api::v1::inventory::models::CreateInventoryItem,
            crate::api::v1::inventory::models::UpdateInventoryItem,
            crate::api::v1::employee::models::Employee,
            crate::api::v1::employee::models::CreateEmployee,
            crate::api::v1::order::models::Order,
            crate::api::v1::order::models::CreateOrder,
        )
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;