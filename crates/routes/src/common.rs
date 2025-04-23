use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct UploadFileResponse {
  pub file_hashes: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetUploadProgress {
  #[serde(rename = "hash")]
  pub file_hash: String,
}

#[derive(Debug, Serialize)]
pub struct GetUploadProgressResponse {
  pub total_bytes: u64,
  pub bytes_uploaded: u64,
}
