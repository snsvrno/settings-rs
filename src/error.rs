#[derive(Debug)]
pub enum Error {
  Error(String)
}

impl Error {
  pub fn unimplemented() -> Error {
    Error::Error("unimplemented".to_string())
  }
}