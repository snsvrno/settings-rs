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
  pub fn is_text(&self) -> bool { if let &Type::Text(_) = self { true } else { false } }
  pub fn is_switch(&self) -> bool { if let &Type::Switch(_) = self { true } else { false } }
  pub fn is_number(&self) -> bool { if let &Type::Number(_) = self { true } else { false } }
  pub fn is_complex(&self) -> bool { if let &Type::Complex(_) = self { true } else { false } }
  pub fn is_array(&self) -> bool { if let &Type::Array(_) = self { true } else { false } }
  pub fn is_none(&self) -> bool { if let &Type::None = self { true } else { false } }

  pub fn to_text(&self) -> Option<String> { if let &Type::Text(ref inner) = self { Some(inner.clone()) } else { None } }
  pub fn to_switch(&self) -> Option<bool> { if let &Type::Switch(ref inner) = self { Some(inner.clone()) } else { None } }
  pub fn to_number(&self) -> Option<u32> { if let &Type::Number(ref inner) = self { Some(inner.clone()) } else { None } }
  pub fn to_complex(&self) -> Option<HashMap<String,Type>> { if let &Type::Complex(ref inner) = self { Some(inner.clone()) } else { None } }
  pub fn to_array(&self) -> Option<Vec<Type>> { if let &Type::Array(ref inner) = self { Some(inner.clone()) } else { None } }

  pub fn move_it(self) -> Type { self }

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

pub trait SupportedType {
  fn wrap(&self) -> Type;
}

impl SupportedType for String {
  fn wrap(&self) -> Type { Type::Text(self.clone()) }
}

impl SupportedType for bool {
  fn wrap(&self) -> Type { Type::Switch(self.clone()) }
}

impl SupportedType for u32 {
  fn wrap(&self) -> Type { Type::Number(self.clone()) }
}

impl SupportedType for HashMap<String,Type> {
  fn wrap(&self) -> Type { Type::Complex(self.clone()) }
}

impl SupportedType for Vec<Type> {
  fn wrap(&self) -> Type { Type::Array(self.clone()) }
}

impl SupportedType for Type {
  fn wrap(&self) -> Type { self.clone() }
}

impl<'a> SupportedType for &'a Type {
  fn wrap(&self) -> Type { 
    match *self {
      &Type::Text(ref inner) => Type::Text(inner.clone()),
      &Type::Switch(ref inner) => Type::Switch(inner.clone()),
      &Type::Number(ref inner) => Type::Number(inner.clone()),
      &Type::Array(ref inner) => Type::Array(inner.clone()),
      &Type::Complex(ref inner) => Type::Complex(inner.clone()),
      &Type::None => Type::None,
    }
  }  
}

impl SupportedType for str {
  fn wrap(&self) -> Type { Type::Text(self.to_string()) }
}

impl<'a> SupportedType for &'a str {
  fn wrap(&self) -> Type { Type::Text(self.to_string()) }
}

