use std::fmt;

pub enum Error {
  Generic,
  Warning(String),
  Error(String)
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      &Error::Generic => { write!(f,"generic error") },
      &Error::Warning(ref text) => { write!(f,"warning: {}",&text)  },
      &Error::Error(ref text) => { write!(f,"error: {}",&text)  },
    }
  }
}