use super::super::todo::task::{dtos::StoreTask, models::Task, services::save_task};
use actix_session::SessionExt;
use actix_web::{
    post,
    web::{Json, Redirect},
    HttpRequest, Responder,
};
use inertia_rust::{hashmap, validators::InertiaValidateOrRedirect};
use serde_json::{Map, Value};

#[post("/v1/todo/store")]
pub async fn handle(req: HttpRequest, body: Json<StoreTask>) -> impl Responder {
    let payload = match body.validate_or_back(&req) {
        Err(err_redirect) => {
            return err_redirect;
        }
        Ok(payload) => payload,
    };

    let title = payload.title.unwrap();

    let task = Task {
        title: title.clone(),
        description: payload.content.unwrap(),
        done: false,
    };

    save_task(task).await;

    let flash = Map::from_iter(
        hashmap![ "success".to_string() => Value::String(format!("Task {title} created successfully!")) ],
    );

    let _ = req.get_session().insert("_flash", flash);

    Redirect::to("/page/v1/todo").see_other()
}
