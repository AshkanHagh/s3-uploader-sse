use std::fmt::{self, Debug};

use actix_web::{ResponseError, http::StatusCode};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json::json;
use strum::{Display, EnumIter};

#[derive(Debug, Display, EnumIter, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum AppErrorType {
  TooMuchItems,
  InternalServerError,
  InvalidFile,
  MimeTypeNotAllowed,
  ToLarge,
  UploadFaild,
  NotFound,
}

pub struct AppError {
  pub error_type: AppErrorType,
  pub inner: anyhow::Error,
}

pub type AppResult<T> = std::result::Result<T, AppError>;

impl<T: Into<anyhow::Error>> From<T> for AppError {
  fn from(t: T) -> Self {
    let cause = t.into();
    Self {
      error_type: AppErrorType::InternalServerError,
      inner: cause,
    }
  }
}

impl Debug for AppError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("AppError")
      .field("error_type", &self.error_type)
      .field("inner", &self.inner)
      .finish()
  }
}

impl fmt::Display for AppError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", &self.error_type)?;
    writeln!(f, "{}", &self.inner)?;
    Ok(())
  }
}

impl From<AppErrorType> for AppError {
  fn from(error_type: AppErrorType) -> Self {
    let inner = anyhow!("{}", error_type);
    Self { error_type, inner }
  }
}

pub trait AppErrorExt<T, E: Into<anyhow::Error>> {
  fn with_app_type(self, error_type: AppErrorType) -> AppResult<T>;
}

impl<T, E: Into<anyhow::Error>> AppErrorExt<T, E> for Result<T, E> {
  fn with_app_type(self, error_type: AppErrorType) -> AppResult<T> {
    self.map_err(|err| AppError {
      error_type,
      inner: err.into(),
    })
  }
}

impl ResponseError for AppError {
  fn status_code(&self) -> actix_web::http::StatusCode {
    match self.error_type {
      AppErrorType::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
      _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
  }
  fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
    println!("ERROR {:?}", self);

    actix_web::HttpResponse::build(self.status_code()).json(json!({
      "success": false,
      "statusCode": self.status_code().to_string(),
      "message": &self.error_type
    }))
  }
}
