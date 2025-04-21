use aws_sdk_s3::{
  operation::upload_part::UploadPartOutput, primitives::ByteStream, types::CompletedMultipartUpload,
};
use s3::S3ImageUploader;

use crate::error::AppResult;

pub mod s3;

pub struct ImageConfig {
  pub access_key: String,
  pub secret_key: String,
  pub bucket: String,
  pub endpoint: String,
}

pub struct ImageUploader {
  client: S3ImageUploader,
}

impl ImageUploader {
  pub fn new(config: &ImageConfig) -> AppResult<Self> {
    let client = S3ImageUploader::new(config, "kalamche")?;
    Ok(Self { client })
  }

  pub async fn create_multipart_upload(&self, key: &str) -> AppResult<String> {
    self.client.create_multipart_upload(key).await
  }

  pub async fn upload_part(
    &self,
    key: &str,
    upload_id: &str,
    part_number: i32,
    bytes: ByteStream,
  ) -> AppResult<UploadPartOutput> {
    self
      .client
      .upload_part(key, upload_id, part_number, bytes)
      .await
  }

  pub async fn complete_multipart_upload(
    &self,
    key: &str,
    completed_upload: CompletedMultipartUpload,
    upload_id: &str,
  ) -> AppResult<()> {
    self
      .client
      .complete_multipart_upload(key, completed_upload, upload_id)
      .await
  }
}
