use actix_web::{
    get, web, HttpRequest, Responder,
};

use inertia_rust::{
    hashmap, prop_resolver, Inertia, InertiaFacade,
    InertiaProp, IntoInertiaPropResult,
};
use serde::Deserialize;
use super::task::services::{get_tasks};

#[get("/todo")]
async fn render(req: HttpRequest, query: web::Query<TodoQuery>) -> impl Responder {
    let page = query.page.unwrap_or(1);

    Inertia::render_with_props(
        &req,
        "Todo/Index".into(),
        hashmap![
            "tasks" => InertiaProp::defer(prop_resolver!({
                let tasks = get_tasks(page).await;
                tasks.into_inertia_value()
            })).into_mergeable(),
            "page" => InertiaProp::data(page)
        ],
    )
    .await
}

#[derive(Deserialize)]
struct TodoQuery {
    page: Option<usize>,
}
