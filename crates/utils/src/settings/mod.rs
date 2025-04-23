use std::{
  cell::LazyCell,
  net::{IpAddr, Ipv4Addr},
};
use structs::Settings;

pub mod structs;

pub const SETTINGS: LazyCell<Settings> = LazyCell::new(|| Settings::init());

impl Settings {
  fn init() -> Self {
    Self {
      access_key: Self::get_var("S3_ACCESS_KEY"),
      secret_key: Self::get_var("S3_SECRET_KEY"),
      bucket: Self::get_var("S3_BUCKET_NAME"),
      path_style: Self::get_var("S3_ALLOW_PATH_STYLE")
        .parse::<bool>()
        .unwrap(),
      endpoint: Self::get_var("S3_API_ENDPOINT"),

      hostname: Self::get_var("HOSTNAME"),
      port: Self::get_var("PORT").parse::<u16>().unwrap(),
      bind: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
    }
  }

  fn get_var(name: &str) -> String {
    dotenvy::var(name).unwrap()
  }
}
