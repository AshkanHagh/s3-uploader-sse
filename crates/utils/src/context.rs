use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

use crate::{image::ImageUploader, stream::UploadProgress};

pub struct AppContext {
  pub storage: ImageUploader,
  pub progress: Arc<Mutex<HashMap<String, UploadProgress>>>,
}
