use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware::Logger, web::Data};
use route_v1::route_v1;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use utils::{
  context::AppContext, error::AppResult, settings::SETTINGS, utils::s3::S3ImageUploader,
};

pub mod route_v1;

#[actix_web::main]
async fn main() -> AppResult<()> {
  env_logger::init();

  let s3 = S3ImageUploader::new(&SETTINGS)?;
  let context = Data::new(AppContext {
    s3,
    progress: Arc::new(Mutex::new(HashMap::new())),
  });

  HttpServer::new(move || {
    App::new()
      .wrap(any_cors())
      .wrap(Logger::default())
      .app_data(context.clone())
      .configure(route_v1)
  })
  .bind("127.0.0.1:7319")?
  .run()
  .await?;

  Ok(())
}

fn any_cors() -> Cors {
  Cors::default()
    .allow_any_origin()
    .allow_any_header()
    .allow_any_method()
}
