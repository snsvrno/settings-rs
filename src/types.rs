use std::collections::HashMap;

#[derive(Serialize,Deserialize,Debug,Clone,PartialEq)]
#[serde(untagged)]
pub enum Type {
  Text(String),
  Switch(bool),
  Number(u32),
  Complex(HashMap<String,Type>),
  Array(Vec<Type>),
  None,
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
  pub fn is_array(&self) -> bool {
    match *self {
      Type::Array(_) => true,
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
  pub fn to_array(&self) -> Option<Vec<Type>> {
    match *self {
      Type::Array(ref array) => Some(array.clone()),
      _ => None
    }
  }

  pub fn flatten(&self , parent_key : Option<String>) -> Type {
    match self {
      &Type::Text(ref text) => Type::Text(text.clone()),
      &Type::Switch(ref boolean) => Type::Switch(boolean.clone()),
      &Type::Number(ref numb) => Type::Number(numb.clone()),
      &Type::Array(ref array) => Type::Array(array.clone()),
      &Type::None => Type::None,
      &Type::Complex(ref numb) => {
        let mut flat : HashMap<String,Type> = HashMap::new();

        for (index,key) in numb {
          let parent = if let Some(ref parent_key) = parent_key { format!("{}.{}",parent_key,index) } else { index.to_string() };
          flat.insert(
            parent.to_string(),
            key.flatten(Some(parent))
          );
        }

        Type::Complex(flat)
      }
    }
  }
}