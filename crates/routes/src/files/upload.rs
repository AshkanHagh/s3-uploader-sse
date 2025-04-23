use actix_multipart::Multipart;
use actix_web::web::{Data, Path};
use actix_web::{HttpResponse, get, post};
use async_stream::stream;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::{CompletedMultipartUpload, CompletedPart};
use bytes::Bytes;
use futures::TryStreamExt;
use tokio::sync::broadcast;
use utils::UploadProgress;
use utils::context::AppContext;
use utils::error::{AppError, AppErrorExt, AppErrorType, AppResult};
use utils::utils::temp_file::TempFile;

use super::MIN_PART_SIZE;
use super::utils::{create_temp_file_with_size, generate_file_hash};
use crate::common::{GetUploadProgress, GetUploadProgressResponse, UploadFileResponse};

use super::utils::upload_part;

#[post("/upload")]
pub async fn upload_image(
  context: Data<AppContext>,
  mut payload: Multipart,
) -> AppResult<HttpResponse> {
  let mut file_hashes: Vec<String> = Vec::with_capacity(5);

  while let Ok(Some(field)) = payload.try_next().await {
    let content_disposition = field.content_disposition();
    let content_type = field
      .content_type()
      .ok_or(AppErrorType::InvalidFile)?
      .to_string();
    let filename = content_disposition
      .get_filename()
      .ok_or(AppErrorType::InvalidFile)?;

    let file_hash = generate_file_hash(filename).await;
    file_hashes.push(file_hash.to_owned());
    let (temp_file, total_bytes) = create_temp_file_with_size(field, &file_hash).await?;

    let (tx, _rx) = broadcast::channel::<u64>(100);
    {
      let mut progress = context.progress.lock().await;
      progress.insert(
        file_hash.to_owned(),
        UploadProgress {
          total_bytes,
          sender: tx.clone(),
        },
      );
    }

    if total_bytes < MIN_PART_SIZE as u64 {
      let file_data = temp_file
        .read_to_vec()
        .await
        .with_app_type(AppErrorType::UploadFaild)?;

      let s3_stream = ByteStream::from(file_data);
      context
        .s3
        .put_object(&file_hash, &content_type, s3_stream)
        .await?;

      let _ = tx.send(total_bytes);
    } else {
      let (parts, multipart_upload_id) = upload_large_file(
        temp_file,
        &context,
        &file_hash,
        total_bytes,
        &content_type,
        &tx,
      )
      .await?;

      let completed_upload = CompletedMultipartUpload::builder()
        .set_parts(Some(parts))
        .build();
      context
        .s3
        .complete_multipart_upload(&file_hash, completed_upload, &multipart_upload_id)
        .await?;
    }
  }

  Ok(HttpResponse::Ok().json(UploadFileResponse { file_hashes }))
}

async fn upload_large_file(
  temp_file: TempFile,
  context: &AppContext,
  file_hash: &str,
  total_bytes: u64,
  content_type: &str,
  tx: &broadcast::Sender<u64>,
) -> Result<(Vec<CompletedPart>, String), AppError> {
  let complete_parts = (total_bytes / MIN_PART_SIZE as u64) as usize;
  let remainder_size = (total_bytes % MIN_PART_SIZE as u64) as usize;

  let multipart_upload_id = context
    .s3
    .create_multipart_upload(file_hash, content_type)
    .await?;

  let mut parts = Vec::new();
  let mut part_number = 1;
  let mut uploaded_bytes: u64 = 0;
  let mut file = temp_file
    .open_file()
    .await
    .with_app_type(AppErrorType::UploadFaild)?;

  for _ in 0..complete_parts {
    upload_part(
      &mut file,
      context,
      file_hash,
      &multipart_upload_id,
      part_number,
      MIN_PART_SIZE,
      &mut uploaded_bytes,
      &mut parts,
      tx,
    )
    .await?;
    part_number += 1;
  }

  if remainder_size > 0 {
    upload_part(
      &mut file,
      context,
      file_hash,
      &multipart_upload_id,
      part_number,
      remainder_size,
      &mut uploaded_bytes,
      &mut parts,
      tx,
    )
    .await?;
  }

  Ok((parts, multipart_upload_id))
}

#[get("/progress/{hash}")]
pub async fn get_upload_progress(
  context: Data<AppContext>,
  path: Path<GetUploadProgress>,
) -> AppResult<HttpResponse> {
  let file_hash = generate_file_hash(&path.file_hash).await;

  let (total_bytes, mut rx) = {
    let progress_map = context.progress.lock().await;
    let progress = progress_map.get(&file_hash).ok_or(AppErrorType::NotFound)?;

    (progress.total_bytes, progress.sender.subscribe())
  };

  let body = stream! {
    while let Ok(bytes) = rx.recv().await {
      let response = serde_json::to_string(&GetUploadProgressResponse {
        bytes_uploaded: bytes,
        total_bytes,
      })?;
      yield Ok::<_, actix_web::Error>(Bytes::from(format!("data: {}\n\n", response)));

      if bytes >= total_bytes {
        let mut progress_map = context.progress.lock().await;
        progress_map.remove(&file_hash);
        break;
      }
    }

    let mut progress_map = context.progress.lock().await;
    progress_map.remove(&file_hash);
  };

  Ok(
    HttpResponse::Ok()
      .content_type("text/event-stream")
      .insert_header(("Cache-Control", "no-cache"))
      .insert_header(("Connection", "keep-alive"))
      .streaming(body),
  )
}
