use actix_multipart::Field;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::CompletedPart;
use futures::TryStreamExt;
use sha2::{Digest, Sha256};
use tokio::sync::broadcast;
use tokio::{
  fs::File,
  io::{AsyncReadExt, AsyncWriteExt},
};
use utils::{
  context::AppContext,
  error::{AppErrorExt, AppErrorType, AppResult},
  utils::temp_file::TempFile,
};

pub async fn generate_file_hash(filename: &str) -> String {
  let mut hasher = Sha256::new();
  hasher.update(filename.as_bytes());
  let hash = format!("{:x}", hasher.finalize());
  hash[..16].to_string()
}

pub async fn create_temp_file_with_size(
  mut field: Field,
  file_hash: &str,
) -> AppResult<(TempFile, u64)> {
  let temp_file_path = format!("/tmp/{}", file_hash);
  let temp_file = TempFile::new(temp_file_path).await?;

  let mut file = temp_file
    .create_file()
    .await
    .with_app_type(AppErrorType::UploadFaild)?;
  while let Ok(Some(bytes)) = field.try_next().await {
    file
      .write_all(&bytes)
      .await
      .with_app_type(AppErrorType::UploadFaild)?;
  }

  file
    .flush()
    .await
    .with_app_type(AppErrorType::UploadFaild)?;
  drop(file);

  let total_bytes = temp_file.get_size().await?;

  Ok((temp_file, total_bytes))
}

pub async fn upload_part(
  file: &mut File,
  context: &AppContext,
  file_hash: &str,
  multipart_upload_id: &str,
  part_number: i32,
  buffer_size: usize,
  uploaded_bytes: &mut u64,
  parts: &mut Vec<CompletedPart>,
  tx: &broadcast::Sender<u64>,
) -> AppResult<()> {
  let mut buffer = vec![0u8; buffer_size];
  file
    .read_exact(&mut buffer)
    .await
    .with_app_type(AppErrorType::UploadFaild)?;

  let s3_stream = ByteStream::from(buffer);
  let upload_part = context
    .s3
    .upload_part(file_hash, multipart_upload_id, part_number, s3_stream)
    .await?;

  parts.push(
    CompletedPart::builder()
      .e_tag(upload_part.e_tag().unwrap())
      .part_number(part_number)
      .build(),
  );

  *uploaded_bytes += buffer_size as u64;
  let _ = tx.send(*uploaded_bytes);
  Ok(())
}
