use actix_web::web::{ServiceConfig, scope};

pub fn route_v1(cfg: &mut ServiceConfig) {
  cfg.service(
    scope("/api/v1/image")
      .service(routes::images::upload::upload_image)
      .service(routes::images::upload::get_upload_progress),
  );
}
