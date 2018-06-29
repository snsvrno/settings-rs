use std::collections::HashMap;

#[derive(Serialize,Deserialize,Debug,Clone,PartialEq)]
#[serde(untagged)]
pub enum Type {
  Text(String),
  Switch(bool),
  Int(i32),
  Float(f32),
  Complex(HashMap<String,Type>),
  Array(Vec<Type>),
  None,
}

impl Type {
  pub fn is_text(&self) -> bool { if let &Type::Text(_) = self { true } else { false } }
  pub fn is_switch(&self) -> bool { if let &Type::Switch(_) = self { true } else { false } }
  pub fn is_int(&self) -> bool { if let &Type::Int(_) = self { true } else { false } }
  pub fn is_float(&self) -> bool { if let &Type::Float(_) = self { true } else { false } }
  pub fn is_complex(&self) -> bool { if let &Type::Complex(_) = self { true } else { false } }
  pub fn is_array(&self) -> bool { if let &Type::Array(_) = self { true } else { false } }
  pub fn is_none(&self) -> bool { if let &Type::None = self { true } else { false } }

  pub fn to_text(&self) -> Option<String> { if let &Type::Text(ref inner) = self { Some(inner.clone()) } else { None } }
  pub fn to_switch(&self) -> Option<bool> { if let &Type::Switch(ref inner) = self { Some(inner.clone()) } else { None } }
  pub fn to_int(&self) -> Option<i32> { if let &Type::Int(ref inner) = self { Some(inner.clone()) } else { None } }
  pub fn to_float(&self) -> Option<f32> { if let &Type::Float(ref inner) = self { Some(inner.clone()) } else { None } }
  pub fn to_complex(&self) -> Option<HashMap<String,Type>> { if let &Type::Complex(ref inner) = self { Some(inner.clone()) } else { None } }
  pub fn to_array(&self) -> Option<Vec<Type>> { if let &Type::Array(ref inner) = self { Some(inner.clone()) } else { None } }

  pub fn move_it(self) -> Type { self }

  pub fn flatten(&self , parent_key : Option<String>) -> Type {
    match self {
      &Type::Text(ref text) => Type::Text(text.clone()),
      &Type::Switch(ref boolean) => Type::Switch(boolean.clone()),
      &Type::Int(ref int) => Type::Int(int.clone()),
      &Type::Float(ref float) => Type::Float(float.clone()),
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

impl SupportedType for () {
  fn wrap(&self) -> Type { Type::None }
}

impl SupportedType for String {
  fn wrap(&self) -> Type { Type::Text(self.clone()) }
}

impl SupportedType for bool {
  fn wrap(&self) -> Type { Type::Switch(self.clone()) }
}

impl SupportedType for i32 {
  fn wrap(&self) -> Type { Type::Int(self.clone()) }
}

impl SupportedType for f32 {
  fn wrap(&self) -> Type { Type::Float(self.clone()) }
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
      &Type::Int(ref inner) => Type::Int(inner.clone()),
      &Type::Float(ref inner) => Type::Float(inner.clone()),
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

