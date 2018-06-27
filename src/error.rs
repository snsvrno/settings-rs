#[derive(Debug)]
pub enum Error {
  Error(String)
}

impl Error {
  pub fn unimplemneted() -> Error {
    Error::Error("Unimplemented".to_string())
  }
}