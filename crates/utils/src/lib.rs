pub mod context;
pub mod error;
pub mod settings;
pub mod utils;

use tokio::sync::broadcast;

#[derive(Debug)]
pub struct UploadProgress {
  pub total_bytes: u64,
  pub file_hash: String,
  pub sender: broadcast::Sender<u64>,
}
