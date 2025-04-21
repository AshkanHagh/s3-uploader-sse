use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct GetUploadProgress {
  #[serde(rename = "name")]
  pub image_name: String,
}

#[derive(Debug, Serialize)]
pub struct GetUploadProgressResponse {
  pub total_bytes: u64,
  pub bytes_uploaded: u64,
  pub image_hash: String,
}
