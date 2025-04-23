use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

use crate::{UploadProgress, utils::s3::S3ImageUploader};

pub struct AppContext {
  pub s3: S3ImageUploader,
  pub progress: Arc<Mutex<HashMap<String, UploadProgress>>>,
}
