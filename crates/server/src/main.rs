use std::{collections::HashMap, sync::Arc};

use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware::Logger, web::Data};
use route_v1::route_v1;
use tokio::sync::Mutex;
use utils::{
  context::AppContext,
  error::AppResult,
  image::{ImageConfig, ImageUploader},
};

pub mod route_v1;

#[actix_web::main]
async fn main() -> AppResult<()> {
  env_logger::init();

  let storage = ImageUploader::new(&ImageConfig {
    access_key: "admin".to_string(),
    secret_key: "password123".to_string(),
    bucket: "kalamche".to_string(),
    endpoint: "http://localhost:9000".to_string(),
    path_style: true,
  })?;

  let context = Data::new(AppContext {
    storage,
    progress: Arc::new(Mutex::new(HashMap::new())),
  });

  HttpServer::new(move || {
    let cors = Cors::default()
      .allow_any_origin()
      .allow_any_header()
      .allow_any_method();

    App::new()
      .wrap(cors)
      .wrap(Logger::default())
      .app_data(context.clone())
      .configure(route_v1)
  })
  .bind("127.0.0.1:7319")?
  .workers(2)
  .run()
  .await?;

  Ok(())
}
