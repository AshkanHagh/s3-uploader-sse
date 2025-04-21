use bytes::Bytes;
use futures::Stream;
use std::{
  pin::Pin,
  task::{Context, Poll},
};
use tokio::sync::broadcast;

use crate::error::AppResult;

#[derive(Debug)]
pub struct UploadProgress {
  pub total_bytes: u64,
  pub image_hash: String,
  pub sender: broadcast::Sender<u64>,
}

pub struct ProgressTrackingStream<S> {
  inner: S,
  sender: broadcast::Sender<u64>,
}

impl<S> ProgressTrackingStream<S>
where
  S: Stream<Item = AppResult<Bytes>> + Unpin,
{
  pub fn new(stream: S, sender: broadcast::Sender<u64>) -> Self {
    Self {
      inner: stream,
      sender,
    }
  }
}

impl<S> Stream for ProgressTrackingStream<S>
where
  S: Stream<Item = AppResult<Bytes>> + Unpin,
{
  type Item = AppResult<Bytes>;
  fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
    match std::pin::Pin::new(&mut self.inner).poll_next(cx) {
      Poll::Ready(Some(Ok(bytes))) => {
        let _ = self.sender.send(bytes.len() as u64);
        Poll::Ready(Some(Ok(bytes)))
      }
      other => other,
    }
  }
}
