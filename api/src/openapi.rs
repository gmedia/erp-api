use utoipa::{Modify, OpenApi};

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearerAuth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::Http::new(
                        utoipa::openapi::security::HttpAuthScheme::Bearer,
                    ),
                ),
            )
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        // Register your handlers here
        crate::v1::inventory::handlers::create_item,
        crate::v1::inventory::handlers::search_items,
        crate::v1::inventory::handlers::get_all_items,
        crate::v1::inventory::handlers::get_item_by_id,
        crate::v1::inventory::handlers::update_item,
        crate::v1::inventory::handlers::delete_item,
        crate::v1::employee::handlers::create_employee,
        crate::v1::order::handlers::create_order,
    ),
    components(
        schemas(
            // Register your models here
            crate::v1::inventory::models::InventoryItem,
            crate::v1::inventory::models::CreateInventoryItem,
            crate::v1::inventory::models::UpdateInventoryItem,
            crate::v1::employee::models::Employee,
            crate::v1::employee::models::CreateEmployee,
            crate::v1::order::models::Order,
            crate::v1::order::models::CreateOrder,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "inventory", description = "Inventory management endpoints.")
    )
)]
pub struct ApiDoc;
