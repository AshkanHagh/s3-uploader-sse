use sha2::{Digest, Sha256};

pub async fn generate_file_hash(filename: &str) -> String {
  let mut hasher = Sha256::new();
  hasher.update(filename.as_bytes());
  let hash = format!("{:x}", hasher.finalize());
  hash[..16].to_string()
}
