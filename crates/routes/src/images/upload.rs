use actix_multipart::Multipart;
use actix_web::web::{Data, Path};
use actix_web::{HttpResponse, get, post};
use async_stream::stream;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::{CompletedMultipartUpload, CompletedPart};
use bytes::Bytes;
use futures::TryStreamExt;
use tokio::sync::broadcast;
use utils::context::AppContext;
use utils::error::{AppError, AppErrorType, AppResult};
use utils::stream::{ProgressTrackingStream, UploadProgress};

use crate::common::{GetUploadProgress, GetUploadProgressResponse};
use crate::images::utils::generate_file_hash;

#[post("/upload/large")]
pub async fn upload_image(
  context: Data<AppContext>,
  mut payload: Multipart,
) -> AppResult<HttpResponse> {
  while let Ok(Some(field)) = payload.try_next().await {
    let content_disposition = field.content_disposition();
    let content_type = field
      .content_type()
      .ok_or(AppErrorType::InvalidFile)?
      .to_string();
    let filename = content_disposition
      .get_filename()
      .ok_or(AppErrorType::InvalidFile)?;

    // TODO: find a way to get full file size in stream
    let total_bytes = 0;
    let file_hash = generate_file_hash(filename).await;

    let (tx, _rx) = broadcast::channel::<u64>(500);
    {
      let mut progress = context.progress.lock().await;
      progress.insert(
        file_hash.to_owned(),
        UploadProgress {
          total_bytes,
          file_hash: file_hash.to_owned(),
          sender: tx.clone(),
        },
      );
    }

    let field_stream = field.map_err(|_| AppError::from(AppErrorType::UploadFaild));
    let mut progress_stream = ProgressTrackingStream::new(field_stream, tx);

    let multipart_upload_id = context
      .storage
      .create_multipart_upload(&file_hash, &content_type)
      .await?;
    let mut parts = Vec::new();
    let mut part_number = 1;

    while let Ok(Some(bytes)) = progress_stream.try_next().await {
      let s3_stream = ByteStream::from(bytes.to_vec());
      let upload_part = context
        .storage
        .upload_part(&file_hash, &multipart_upload_id, part_number, s3_stream)
        .await?;

      parts.push(
        CompletedPart::builder()
          .e_tag(upload_part.e_tag().unwrap())
          .part_number(part_number)
          .build(),
      );
      part_number += 1;
    }

    let completed_upload = CompletedMultipartUpload::builder()
      .set_parts(Some(parts))
      .build();
    context
      .storage
      .complete_multipart_upload(&file_hash, completed_upload, &multipart_upload_id)
      .await?;
  }

  Ok(HttpResponse::NoContent().finish())
}

#[post("upload/small")]
pub async fn upload_small(
  context: Data<AppContext>,
  mut payload: Multipart,
) -> AppResult<HttpResponse> {
  while let Ok(Some(field)) = payload.try_next().await {
    let content_disposition = field.content_disposition();
    let content_type = field
      .content_type()
      .ok_or(AppErrorType::InvalidFile)?
      .to_string();
    let filename = content_disposition
      .get_filename()
      .ok_or(AppErrorType::InvalidFile)?;

    let total_bytes = 0;
    let file_hash = generate_file_hash(filename).await;

    let (tx, _rx) = broadcast::channel::<u64>(100);
    {
      let mut progress_map = context.progress.lock().await;
      progress_map.insert(
        file_hash.to_owned(),
        UploadProgress {
          file_hash: file_hash.to_owned(),
          sender: tx.clone(),
          total_bytes,
        },
      );
    }

    let field_stream = field.map_err(|_| AppError::from(AppErrorType::UploadFaild));
    let mut progress_stream = ProgressTrackingStream::new(field_stream, tx);

    let mut buffer = Vec::new();
    while let Ok(Some(bytes)) = progress_stream.try_next().await {
      buffer.extend_from_slice(&bytes);
    }

    let byte_stream = ByteStream::from(buffer);
    context
      .storage
      .put_object(&file_hash, &content_type, byte_stream)
      .await?;
  }

  Ok(HttpResponse::NoContent().finish())
}

#[get("/{hash}")]
pub async fn get_upload_progress(
  context: Data<AppContext>,
  path: Path<GetUploadProgress>,
) -> AppResult<HttpResponse> {
  let file_hash = generate_file_hash(&path.file_hash).await;

  let (total_bytes, mut rx, image_hash) = {
    let progress_map = context.progress.lock().await;
    let progress = progress_map.get(&file_hash).ok_or(AppErrorType::NotFound)?;

    (
      progress.total_bytes,
      progress.sender.subscribe(),
      progress.file_hash.clone(),
    )
  };

  let body = stream! {
    let mut uploaded = 0;
    while let Ok(bytes) = rx.recv().await {
      uploaded += bytes;

      let response = serde_json::to_string(&GetUploadProgressResponse {
        bytes_uploaded: uploaded,
        total_bytes,
        file_hash: image_hash.to_owned()
      })?;
      yield Ok::<_, actix_web::Error>(Bytes::from(format!("data: {}\n\n", response)))
    }
  };

  Ok(
    HttpResponse::Ok()
      .content_type("text/event-stream")
      .insert_header(("Cache-Control", "no-cache"))
      .insert_header(("Connection", "keep-alive"))
      .streaming(body),
  )
}
