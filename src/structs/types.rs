use std::collections::HashMap;
use std::fmt;

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
    // Checking types to see if `Type` is what you think it is, or want it to be.
    pub fn is_text(&self) -> bool { if let &Type::Text(_) = self { true } else { false } }
    pub fn is_switch(&self) -> bool { if let &Type::Switch(_) = self { true } else { false } }
    pub fn is_int(&self) -> bool { if let &Type::Int(_) = self { true } else { false } }
    pub fn is_float(&self) -> bool { if let &Type::Float(_) = self { true } else { false } }
    pub fn is_complex(&self) -> bool { if let &Type::Complex(_) = self { true } else { false } }
    pub fn is_array(&self) -> bool { if let &Type::Array(_) = self { true } else { false } }
    pub fn is_none(&self) -> bool { if let &Type::None = self { true } else { false } }

    // Casts to get the inner value of the type. If you cast to the wrong thing you will get a None.
    // These don't "use" the original data but instead clone it.
    pub fn to_text(&self) -> Option<String> { if let &Type::Text(ref inner) = self { Some(inner.clone()) } else { None } }
    pub fn to_switch(&self) -> Option<bool> { if let &Type::Switch(ref inner) = self { Some(inner.clone()) } else { None } }
    pub fn to_int(&self) -> Option<i32> { if let &Type::Int(ref inner) = self { Some(inner.clone()) } else { None } }
    pub fn to_float(&self) -> Option<f32> { if let &Type::Float(ref inner) = self { Some(inner.clone()) } else { None } }
    pub fn to_complex(&self) -> Option<HashMap<String,Type>> { if let &Type::Complex(ref inner) = self { Some(inner.clone()) } else { None } }
    pub fn to_array(&self) -> Option<Vec<Type>> { if let &Type::Array(ref inner) = self { Some(inner.clone()) } else { None } }

    // pub fn move_it(self) -> Type { self }

    pub fn flatten(&self , parent_key : Option<String>) -> Type {
        //! Flattens the `Type`. 
        //! 
        //! If the type is anything but a `Type::Complex` it just 
        //! returns a copy of the original `Type`.
        //! 
        //! If the type is a `Type::Complex` it returns a 1D Complex,
        //! thus flattening it. This is usually used recursively to
        //! flatten an entire `Settings`

        match self {
            &Type::Text(ref text) => Type::Text(text.clone()),
            &Type::Switch(ref boolean) => Type::Switch(boolean.clone()),
            &Type::Int(ref int) => Type::Int(int.clone()),
            &Type::Float(ref float) => Type::Float(float.clone()),
            &Type::Array(ref array) => Type::Array(array.clone()),
            &Type::None => Type::None,
            &Type::Complex(ref numb) => {
                let mut flat : HashMap<String,Type> = HashMap::new();

                for (key,value) in numb {
                    let parent = if let Some(ref parent_key) = parent_key { 
                        format!("{}.{}",parent_key,key) 
                    } else { 
                        key.to_string() 
                    };

                    let flattened = value.flatten(Some(parent.to_string()));

                    if flattened.is_complex() {
                        for (key,value) in flattened.to_complex().unwrap() {
                            flat.insert(key,value);
                        }
                    } else {
                        flat.insert(parent.to_string(),flattened);
                    }
                }

                Type::Complex(flat)
            }
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Int(ref value) => write!(f,"{}",value),
            Type::Switch(ref value) => write!(f,"{}",value),
            Type::Float(ref value) => write!(f,"{}",value),
            Type::Text(ref value) => write!(f,"{}",value),
            Type::None => write!(f,"[BLANK]"),
            Type::Array(ref value) => {
                write!(f,"[ ");
                for i in 0..value.len() {
                    write!(f,"{}",value[i]);
                    if i < value.len() - 1 { 
                        write!(f,", ");
                    }
                }
                write!(f," ]")
            },
            Type::Complex(ref value) => {
                write!(f,"{{ ");
                for (k,v) in value {
                    write!(f,"{} : {}, ", k,v);
                }
                write!(f," }}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use Type;
    use std::collections::HashMap;

    #[test]
    fn flatten() {
        //! Testing if flattening works correctly, something very basic.
        
        let mut hash : HashMap<String,Type> = HashMap::new();
        let mut hash2 : HashMap<String,Type> = HashMap::new();
        hash2.insert("a".to_string(), Type::Switch(true));
        hash2.insert("float".to_string(), Type::Float(10.23));
        hash2.insert("int".to_string(), Type::Int(10));
        hash2.insert("array".to_string(), Type::Array(vec![Type::Int(1), Type::Switch(false), Type::Text("testing here".to_string())]));
        
        hash.insert("b".to_string(),Type::Complex(hash2));

        let complex = Type::Complex(hash).flatten(None);

        if let Type::Complex(ref stuff) = complex {
            for (k,v) in stuff {
                println!("{} : {:?}",k,v);
            }
        }

        assert!(complex.to_complex().unwrap().get("b.a").unwrap().to_switch().unwrap());
        assert!(complex.to_complex().unwrap().get("a") == None);
        assert!(complex.to_complex().unwrap().get("b.int").unwrap().to_int().unwrap() == 10);
        assert!(complex.to_complex().unwrap().get("b.float").unwrap().to_float().unwrap() == 10.23);
    }

    #[test]
    fn display_print() {
        let test1 = Type::Int(12);
        let test2 = Type::Switch(false);
        let test3 = Type::Float(12.01);
        let test4 = Type::Text("Wjat os tjos".to_string());
        let test5 = Type::Array(vec![ Type::Int(1),Type::Float(2.2) ]);

        let mut hash : HashMap<String,Type> = HashMap::new();
        hash.insert("1".to_string(),test1.clone());
        hash.insert("2".to_string(),test2.clone());
        hash.insert("3".to_string(),test3.clone());
        hash.insert("4".to_string(),test4.clone());
        hash.insert("5".to_string(),test5.clone());
        let test6 = Type::Complex(hash);

        println!("{}",test1);
        println!("{}",test2);
        println!("{}",test3);
        println!("{}",test4);
        println!("{}",test5);
        println!("{}",test6);

        assert!(true);
    }
}