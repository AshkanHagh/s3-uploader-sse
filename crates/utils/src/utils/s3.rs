use crate::{
  error::{AppErrorType, AppResult},
  settings::{SETTINGS, structs::Settings},
};
use aws_sdk_s3::{
  Client, config::Credentials, operation::upload_part::UploadPartOutput, primitives::ByteStream,
  types::CompletedMultipartUpload,
};

pub struct S3ImageUploader {
  pub(super) bucket: Client,
}

impl S3ImageUploader {
  pub fn new(config: &Settings) -> AppResult<Self> {
    let credentials = Credentials::new(&config.access_key, &config.secret_key, None, None, "s3");
    let config = aws_sdk_s3::Config::builder()
      .region(aws_sdk_s3::config::Region::new(""))
      .credentials_provider(credentials)
      .endpoint_url(&config.endpoint)
      .force_path_style(config.path_style)
      .behavior_version_latest()
      .build();

    Ok(Self {
      bucket: Client::from_conf(config),
    })
  }

  pub async fn put_object(
    &self,
    key: &str,
    content_type: &str,
    bytes: ByteStream,
  ) -> AppResult<()> {
    self
      .bucket
      .put_object()
      .bucket(&SETTINGS.bucket)
      .key(key)
      .content_type(content_type)
      .body(bytes)
      .send()
      .await?;

    Ok(())
  }

  pub async fn create_multipart_upload(&self, key: &str, content_type: &str) -> AppResult<String> {
    let multipart_upload = self
      .bucket
      .create_multipart_upload()
      .bucket(&SETTINGS.bucket)
      .content_type(content_type)
      .key(key)
      .send()
      .await?;

    let upload_id = multipart_upload
      .upload_id()
      .ok_or(AppErrorType::UploadFaild)?
      .to_string();

    Ok(upload_id)
  }

  pub async fn upload_part(
    &self,
    key: &str,
    upload_id: &str,
    part_number: i32,
    bytes: ByteStream,
  ) -> AppResult<UploadPartOutput> {
    let upload_part = self
      .bucket
      .upload_part()
      .bucket(&SETTINGS.bucket)
      .key(key)
      .upload_id(upload_id)
      .part_number(part_number)
      .body(bytes)
      .send()
      .await?;

    Ok(upload_part)
  }

  pub async fn complete_multipart_upload(
    &self,
    key: &str,
    completed_upload: CompletedMultipartUpload,
    upload_id: &str,
  ) -> AppResult<()> {
    self
      .bucket
      .complete_multipart_upload()
      .bucket(&SETTINGS.bucket)
      .key(key)
      .multipart_upload(completed_upload)
      .upload_id(upload_id)
      .send()
      .await?;

    Ok(())
  }
}
