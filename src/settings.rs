
use types::SupportedType;
use std::ops::{Add,AddAssign};
use std::io::prelude::*;
use std::path::PathBuf;
use std::collections::HashMap;
use std::fs::{File,create_dir_all};

//use Setting;
use Format;
//use SettingResult;
use error::Error;
use types::Type;

#[derive(Serialize,Deserialize)]
pub struct Settings<T> where T : Format + Clone {
  parts : Type,
  ioconfig: T
}

impl<T> Settings<T> where T : Format + Clone{
  pub fn new(config:T) -> Settings<T> { Settings { parts : Type::None, ioconfig : config } }

  pub fn from_flat(flat_hash : &Settings<T>) -> Result<Settings<T>,Error> {
    let mut new_hash = Settings::new(flat_hash.ioconfig.clone());

    if let Type::Complex(ref flat_hash) = flat_hash.parts {
      for (key,value) in flat_hash.iter() {
        new_hash.set_value(&key,&value);
      } 
      Ok(new_hash)
    } else {
      Err(Error::Error("input Type is not a flat_hash style / Complex Setting Block".to_string()))
    }
  }

  pub fn load_from(path : &PathBuf, config : T) -> Result<Settings<T>,Error> {
    //! loads a settings object from a path, returns error if can't

    // loads the raw file into a buffer
    let mut buf : String = String::new();
    match File::open(&path) {
      Err(error) => return Err(Error::Error(error.to_string())),
      Ok(mut file) => { 
        match file.read_to_string(&mut buf) {
          Err(error) => return Err(Error::Error(error.to_string())),
          Ok(_) => { },
        } 
      }
    }

    // parses the string
    if buf.len() > 0 {
      let hash : Result<Type,Error> = config.from_str(&buf);
      match hash {
        Err(error) => return Err(error),
        Ok(parts) => { return Ok(Settings { parts: parts, ioconfig: config }); }
      }
    } else { 
      Err(Error::Error(format!("file {} is empty?",path.display().to_string())))
    }
  }
  pub fn load_from_or_empty(path : &PathBuf, config : T) -> Settings<T> {
    //! loads a new setting object from a path, or creates an empty object it path doesn't exist

    match Settings::load_from(&path,config.clone()) {
      Ok(settings) => { settings }
      Err(_) => { Settings::new(config.clone()) }
    }
  }

  pub fn save_to(&self, path : &PathBuf) -> Result<(),Error> {
    //! saves the setting object to a certain path

    match self.ioconfig.to_string(&self.parts){
      Err(error) => return Err(error),
      Ok(settings_string) => { 
        // creates folder if doen't exist
        if let Some(path) = path.parent() {
          if !path.exists() { 
            if let Err(error) = create_dir_all(&path) {
              return Err(Error::Error(error.to_string()));
            }
          }
        }

        let file = File::create(path);
        match file {
          Err(error) => return Err(Error::Error(error.to_string())),
          Ok(mut file) => {
            match file.write_all(settings_string.as_bytes()) {
              Err(error) => return Err(Error::Error(error.to_string())),
              Ok(_) => { return Ok(()); }
            }
          }
        }
      }
    }
  }

  pub fn get_flat_hash(&self) -> Settings<T> {
    Settings::flatten(self)
  }

  pub fn is_flat(&self) -> bool {
    if let Type::Complex(ref hash) = self.parts {
      for (_,value) in hash.iter() {
        if !value.is_complex() { return false; }
      }
      return true;
    }

    false
  }

  pub fn flatten(hash_to_flatten : &Settings<T>) -> Settings<T> {
    return Settings { parts : hash_to_flatten.parts.flatten(None), ioconfig : hash_to_flatten.ioconfig.clone() };
  }

  pub fn as_type(&self) -> Type {
    self.parts.clone()
  }

  pub fn get_value(&self, key_path : &str) -> Option<Type>{
    if let Some(raw_part) = self.get_raw(&key_path) {
      return Some(raw_part.clone());
    }
    None
  }

  pub fn get_raw(&self, key_path : &str) -> Option<Type> {

    let path_tree : Vec<&str> = key_path.split(".").collect();
    let mut subtree : &Type = &Type::Text("Empty".to_string());

    // TODO : need to fix this in order to have full unicode support. need to use .chars() instead of slice.
    for i in 0..path_tree.len() {
      if i == 0 { 
        if let Type::Complex(ref parts) = self.parts {
          if let Some(ref part) = parts.get(&path_tree[i].to_string()) {
            subtree = part;
          } else { return None }
        } else { return None }
      } else {
        match *subtree {
          Type::Complex(ref hash) => { 
            if let Some(ref part) = hash.get(&path_tree[i].to_string()) {
              subtree = part;
            } else { return None }
          },
          _ => { return None }
        }
      }
    }

    return Some(subtree.clone());
  }

  pub fn set_value<A:?Sized>(&mut self, key_path : &str, value : &A) -> Result<(),Error> 
    where A : SupportedType ,
  {
    let mut parts : Vec<Type> = Vec::new();
    let path_tree : Vec<&str> = key_path.split(".").collect();

    for i in 0..path_tree.len()-1 {
      if i == 0 {
        if let Type::Complex(ref mut self_parts) = self.parts {
          if let Some(part) = self_parts.remove(&path_tree[i].to_string()) {
            if let Type::Complex(hash) = part { 
              parts.push(Type::Complex(hash)); 
            } else { parts.push(Type::Complex(HashMap::new())); }
          } else { parts.push(Type::Complex(HashMap::new())); }
        } else { parts.push(Type::Complex(HashMap::new())); }
      } else {
        let index = parts.len()-1;
        let mut push_me : Option<Type> = None;
        if let Type::Complex(ref mut mut_parts) = parts[index] {
          if let Some(part) = mut_parts.remove(&path_tree[i].to_string()) {
            if let Type::Complex(hash) = part { 
              push_me = Some(Type::Complex(hash));
            }
          }
        }
        match push_me {
          None => parts.push(Type::Complex(HashMap::new())),
          Some(push_me) => parts.push(push_me)
        }
      }
    }
       
    if let Type::Complex(ref mut parts_two) = parts[path_tree.len()-2] {
      parts_two.insert(path_tree[path_tree.len()-1].to_string(),value.wrap());
    }

    
    // rebuilds the tree
    if parts.len() > 1 {
        for i in (1..parts.len()).rev() {
            let temp_part = parts.remove(i);
            if let Type::Complex(ref mut parts_minus_1) = parts[i-1] {
              parts_minus_1.insert(path_tree[i].to_string(),temp_part);
            }
        }    
    }

    match self.parts {
      Type::Complex(ref mut self_parts) => { self_parts.insert(path_tree[0].to_string(),parts.remove(0)); }
      _ => {
        let mut hash : HashMap<String,Type> = HashMap::new();
        hash.insert(path_tree[0].to_string(),parts.remove(0));
        self.parts = Type::Complex(hash);
      }
    }
    
    Ok(())
  }
}

impl<T> Add for Settings<T> where T : Format + Clone {
  type Output = Settings<T>;

  fn add(self, other: Settings<T>) -> Settings<T> {
    let mut flat_self = self.get_flat_hash();
    let flat_other = other.get_flat_hash();
    
    if !flat_self.is_flat() || !flat_other.is_flat() { 
      // TODO : need to fix this better somehow!
      return Settings::new(self.ioconfig);
    }

    if let Type::Complex(ref mut flat_self) = flat_self.parts {
      for (key,value) in flat_other.parts.to_complex().unwrap().iter() {
        flat_self.insert(key.to_string(),value.clone());
      } 
    }

    Settings::from_flat(&flat_self).unwrap()
  }
}

impl<T> AddAssign for Settings<T> where T : Format + Clone {
  fn add_assign(&mut self, other:Settings<T>) {
    let flat_other = other.get_flat_hash();

    if let Type::Complex(ref flat_other) = flat_other.parts {
      for (key,value) in flat_other.iter() {
        self.set_value(&key,&value);
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use Format;
  use std::collections::HashMap;
  use settings::Settings;
  use types::Type;
  use serde;
  use error::Error;
  extern crate toml;
  
  #[derive(Clone)]
  struct Configuration { }
  impl Format for Configuration {
    fn filename(&self) -> String { "".to_string() }
    fn folder(&self) -> String { "".to_string() }

    fn from_str(&self,_:&str) -> Result<Type,Error> { Err(Error::unimplemented()) }
    fn to_string<T:?Sized>(&self,_:&T) -> Result<String,Error> where T : serde::ser::Serialize { Err(Error::unimplemented()) }
  }

  #[test]
  fn get_value() {
    let mut test_hash : HashMap<String,Type> = HashMap::new();
    test_hash.insert("test".to_string(),Type::Text("value".to_string()));

    let test_obj = Settings { parts : Type::Complex(test_hash), ioconfig : Configuration { } };
    
    assert_eq!(Some(Type::Text("value".to_string())),test_obj.get_value("test"));
    assert_eq!(None,test_obj.get_value("tester"));
  }

  #[test]
  fn set_value() {
    let mut test_obj = Settings::new(Configuration{});
    assert_eq!(test_obj.set_value("a.b.c.d","mortan").is_ok(),true);
    match test_obj.as_type() {
      Type::Complex(hash) => { assert_eq!(hash.get("a.b.c.d").is_some(),true); }
      _ => { assert_eq!("doesn't have key","true"); }
    }
    assert_eq!(test_obj.get_value("a.b.c.d"),Some(Type::Text("mortan".to_string())));
  }

  /*#[test]
  fn flatten() {
    let mut test_obj = Settings::new();
    test_obj.set_value("user.name","snsvrno");
    test_obj.set_value("user.place","space");
    test_obj.set_value("other.thing","nothing");

    if let Ok(flat) = test_obj.get_flat_hash() {
      if let Type::Complex(flat) = flat {
        assert_eq!(Some(&"snsvrno".to_string()),flat.get("user.name"));
        assert_eq!(Some(&"space".to_string()),flat.get("user.place"));
        assert_eq!(Some(&"nothing".to_string()),flat.get("other.thing"));
      } else { assert_eq!(true,false); }
    } else { assert_eq!(true,false); }
  }*/

  /*#[test]
  fn add() {
    let mut test_obj = Settings::new();
    test_obj.set_value("user.name","snsvrno");
    test_obj.set_value("other.stuff","what");
    test_obj.set_value("other.thing","nope");

    let mut test_obj2 = Settings::new();
    test_obj2.set_value("user.place","space");
    test_obj2.set_value("other.thing","nothing");

    let test_obj3 = test_obj + test_obj2;

    assert_eq!(test_obj3.get_value("other.thing"),Some("nothing".to_string()));
    assert_eq!(test_obj3.get_value("other.stuff"),Some("what".to_string()));
    assert_eq!(test_obj3.get_value("user.place"),Some("space".to_string()));
    assert_eq!(test_obj3.get_value("user.name"),Some("snsvrno".to_string()));

  }*/
}