use std::collections::HashMap;

#[derive(Serialize,Deserialize,Debug,Clone)]
#[serde(untagged)]
pub enum Subsetting {
  Single(String),
  Complex(HashMap<String,Subsetting>)
}

impl Subsetting {
  pub fn to_string(&self) -> Option<String> {
    match *self {
      Subsetting::Single(ref string) => { Some(string.clone()) },
      Subsetting::Complex(ref _c) => { None }
    }
  }

  pub fn to_hash(&self) -> Option<HashMap<String,Subsetting>> {
    match *self {
      Subsetting::Single(ref _s) => { None },
      Subsetting::Complex(ref complex) => { Some(complex.clone()) }
    }
  }

  pub fn is_string(&self) -> bool {
    match *self {
      Subsetting::Single(ref _s) => { true }
      Subsetting::Complex(ref _c) => { false }
    }
  }

  pub fn is_hash(&self) -> bool {
    match *self {
      Subsetting::Single(ref _s) => { false }
      Subsetting::Complex(ref _c) => { true }
    }
  }
}