use std::net::IpAddr;

pub struct Settings {
  /// AWS access key ID for S3 authentication.
  pub access_key: String,
  /// AWS secret access key for S3 authentication.
  pub secret_key: String,
  /// Name of the S3 bucket to upload files to.
  pub bucket: String,
  /// S3 endpoint URL (e.g., "s3.amazonaws.com" or custom endpoint).
  pub endpoint: String,
  /// Use path-style S3 URLs (true) or virtual-hosted style (false).
  pub path_style: bool,
  /// Hostname for the API server (e.g., "localhost" or "example.com").
  pub hostname: String,
  /// IP address to bind the API server to (e.g., 127.0.0.1).
  pub bind: IpAddr,
  /// Port for the API server to listen on (e.g., 8080).
  pub port: u16,
}
