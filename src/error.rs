use std;

#[derive(Debug)]
pub enum Error {
  Error(String),
  Blank
}

impl Error {
  pub fn unimplemented() -> Error {
    Error::Error("unimplemented".to_string())
  }

  pub fn wrap<E>( error : E ) -> Error 
    where E : std::string::ToString,
  {
    return Error::Error(error.to_string());
  }
}