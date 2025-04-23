pub struct TempFile {
  path: String,
}

impl TempFile {
  pub async fn new(path: String) -> Result<Self, std::io::Error> {
    if let Some(parent) = std::path::Path::new(&path).parent() {
      tokio::fs::create_dir_all(parent).await?;
    }

    Ok(Self { path })
  }

  pub fn path(&self) -> &str {
    &self.path
  }

  pub async fn create_file(&self) -> Result<tokio::fs::File, std::io::Error> {
    tokio::fs::File::create(&self.path).await
  }

  pub async fn open_file(&self) -> Result<tokio::fs::File, std::io::Error> {
    tokio::fs::File::open(&self.path).await
  }

  pub async fn get_size(&self) -> Result<u64, std::io::Error> {
    let metadata = tokio::fs::metadata(&self.path).await?;
    Ok(metadata.len())
  }

  pub async fn read_to_vec(&self) -> Result<Vec<u8>, std::io::Error> {
    tokio::fs::read(&self.path).await
  }
}

impl Drop for TempFile {
  fn drop(&mut self) {
    let path = self.path.clone();
    tokio::spawn(async move {
      let _ = tokio::fs::remove_file(path).await;
    });
  }
}

// async fn example_with_temp_file() -> Result<(), std::io::Error> {
//   let temp = TempFile::new("/tmp/example".to_string()).await?;

//   Ok(())
// }
