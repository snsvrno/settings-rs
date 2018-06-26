use std::collections::HashMap;

#[derive(Serialize,Deserialize,Debug,Clone)]
#[serde(untagged)]
pub enum Type {
  Text(String),
  Switch(bool),
  Number(u32),
  Complex(HashMap<String,Type>)
}

impl Type {
  pub fn is_text(&self) -> bool {
    match *self {
      Type::Text(_) => true,
      _ => false
    }
  }
  pub fn is_switch(&self) -> bool {
    match *self {
      Type::Switch(_) => true,
      _ => false
    }
  }
  pub fn is_number(&self) -> bool {
    match *self {
      Type::Number(_) => true,
      _ => false
    }
  }
  pub fn is_complex(&self) -> bool {
    match *self {
      Type::Complex(_) => true,
      _ => false
    }
  }
  pub fn to_text(&self) -> Option<String> {
    match *self {
      Type::Text(ref text) => Some(text.to_string()),
      _ => None
    }
  }
  pub fn to_switch(&self) -> Option<bool> {
    match *self {
      Type::Switch(ref boolean) => Some(boolean.clone()),
      _ => None
    }
  }
  pub fn to_number(&self) -> Option<u32> {
    match *self {
      Type::Number(ref numb) => Some(numb.clone()),
      _ => None
    }
  }
  pub fn to_complex(&self) -> Option<HashMap<String,Type>> {
    match *self {
      Type::Complex(ref hash) => Some(hash.clone()),
      _ => None
    }
  }
}