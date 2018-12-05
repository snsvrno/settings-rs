use std::collections::HashMap;
use Type;

/// Trait for data types that can be inserted into a `Settings`.
/// 
/// Implementing this trait for a custom struct will allow this 
/// struct to be used with `Settings` directly.
/// 
/// # Implemented Types
/// 
/// - String
/// - bool
/// - i32
/// - f32
/// - Vec<Type>
/// - HashMap<String,Type>
/// 
pub trait SupportedType {

    /// Function to wrap the type into a [Type](enum.Type.html)
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