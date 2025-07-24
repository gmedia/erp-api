use crate::v1::auth::{handlers, middleware::JwtMiddleware};
use actix_web::web;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    let jwt_middleware = JwtMiddleware::new("Bearer ".to_string());
    cfg.service(
        web::scope("/v1/auth")
            .route("/register", web::post().to(handlers::register))
            .route("/login", web::post().to(handlers::login))
            .service(
                web::resource("/me")
                    .wrap(jwt_middleware)
                    .route(web::get().to(handlers::me)),
            ),
    );
}
