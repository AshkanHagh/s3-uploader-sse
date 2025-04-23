use actix_web::web::{ServiceConfig, scope};

pub fn route_v1(cfg: &mut ServiceConfig) {
  cfg.service(
    scope("/api/v1")
      .service(routes::uploads::upload::upload_image)
      .service(routes::uploads::upload::get_upload_progress),
  );
}
