
use types::SupportedType;
use PartsPackage;
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
  parts : HashMap<String,Type>,
  ioconfig: T,
}

impl<T> Settings<T> where T : Format + Clone{
  pub fn new(config:T) -> Settings<T> { Settings { parts : HashMap::new(), ioconfig : config } }

  pub fn from_flat(flat_hash : &Settings<T>) -> Settings<T> {
    let mut new_hash = Settings::new(flat_hash.ioconfig.clone());

    for (key,value) in flat_hash.parts.iter() {
      new_hash.set_value(&key,&value);
    } 
    
    new_hash
  }

  /*pub fn load_from(path : &PathBuf, config : T) -> Result<Settings<T>,Error> {
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
      let hash : Result<PartsPackage,Error> = config.from_str(&buf);
      match hash {
        Err(error) => return Err(error),
        Ok(hash) => return Ok(Settings{ parts : hash, ioconfig : config })
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
  }*/

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
    //! returns the flattened form of the ***Setting***, shortcut of `flatten()`

    Settings::flatten(self)
  }

  pub fn is_flat(&self) -> bool {
    //! checks if the settings file is flat
    //!
    //! a flat ***Settings*** is defined by no ***Type*** being `Type::Complex`.
    //! In basic terms it is a `HashMap` that has a depth of '1' and all the 
    //! `keys` are actually `key_paths`.
    //!
    //! example of a flat ***Settings*** in JSON format.
    //!
    //! ```json
    //! // example flat settings
    //! { 
    //!   "user.name" : "snsvrno",
    //!   "user.brightness" : 123,
    //!   "program.default.storage" : "C:",
    //!   "user.path" : [ "~/bin" , "~/.cargo/bin" ]
    //! }
    //! ```
    //!
    //! and the same example in not in a flat form
    //!
    //! ```json
    //! {
    //!   "user" : {
    //!     "name" : "snsvrno",  
    //!     "brightness" : 123,
    //!     "path" : [
    //!       "~/bin",
    //!       "~/.cargo/bin"
    //!     ],
    //!   },  
    //!   "program" : {
    //!     "default" : {
    //!       "storage" : "C:"
    //!     }
    //!   }
    //! }

    for (_,value) in self.parts.iter() {
      if !value.is_complex() { return false; }
    }
    if self.parts.len() > 0 { return true; }
    false
  }

  pub fn flatten(hash_to_flatten : &Settings<T>) -> Settings<T> {
    //! used to flatten a ***Settings***

    let mut flat_hash : HashMap<String,Type> = HashMap::new(); // new hash to return at the end

    // iterates through all the `Types` in the `self.parts` of the ***Settings***,
    // checks if each is a `Type::Complex`, if so then adds it to the flat_hash,
    // and if not then just adds the resulting type from `flatten()` to the 
    // flat_hash returner object.
    for (key,value) in hash_to_flatten.parts.iter() {
      let temp_type : Type = value.flatten(Some(key.to_string()));
      if let Type::Complex(hash) = temp_type {
        for (k2,v2) in hash.iter() {
          flat_hash.insert(k2.to_string(),v2.clone());
        }
      } else {
        flat_hash.insert(key.to_string(),temp_type);
      }
    }

    return Settings { parts : flat_hash, ioconfig : hash_to_flatten.ioconfig.clone() };
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
        if let Some(ref part) = self.parts.get(&path_tree[i].to_string()) {
          subtree = part;
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
        if let Some(part) = self.parts.remove(&path_tree[i].to_string()) {
          if let Type::Complex(hash) = part { 
            parts.push(Type::Complex(hash)); 
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
    println!("{:?}",key_path);
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

    self.parts.insert(path_tree[0].to_string(),parts.remove(0));
    
    Ok(())
  }
}

impl<T> Add for Settings<T> where T : Format + Clone {
  type Output = Settings<T>;

  fn add(self, other: Settings<T>) -> Settings<T> {
    let mut flat_self = self.get_flat_hash();
    let flat_other = other.get_flat_hash();

    for (key,value) in flat_other.parts.iter() {
      flat_self.parts.insert(key.to_string(),value.clone());
    } 

    Settings::from_flat(&flat_self)
  }
}

impl<T> AddAssign for Settings<T> where T : Format + Clone {
  fn add_assign(&mut self, other:Settings<T>) {
    let flat_other = other.get_flat_hash();

    for (key,value) in flat_other.parts.iter() {
      self.set_value(&key,&value);
    }
  }
}

#[cfg(test)]
mod tests {
  use types::SupportedType;
  use PartsPackage;
  use Format;
  use std::collections::HashMap;
  use settings::Settings;
  use types::Type;
  use error::Error;
  extern crate toml;
  
  #[derive(Clone)]
  struct Configuration { }
  impl Format for Configuration {
    fn filename(&self) -> String { "".to_string() }
    fn folder(&self) -> String { "".to_string() }

    fn from_str<T>(&self,_:&str) -> Result<PartsPackage,Error> where T : Format + Clone { Err(Error::unimplemented()) }
    fn to_string<T:?Sized>(&self,_:&T) -> Result<String,Error> where T : SupportedType { Err(Error::unimplemented()) }
  }

  #[test]
  fn get_value() {
    let mut test_hash : HashMap<String,Type> = HashMap::new();
    test_hash.insert("test".to_string(),Type::Text("value".to_string()));
    test_hash.insert("number".to_string(),Type::Number(13223));
    test_hash.insert("switch".to_string(),Type::Switch(false));
    test_hash.insert("array".to_string(),Type::Array(vec![Type::Number(1),Type::Text("other".to_string())]));

    let test_obj = Settings { parts : test_hash, ioconfig : Configuration { } };
    
    assert_eq!(Some(Type::Text("value".to_string())),test_obj.get_value("test")); // testing text
    assert_eq!(Some(Type::Number(13223)),test_obj.get_value("number")); // testing number 
    assert_eq!(Type::Number(1),test_obj.get_value("array").unwrap().to_array().unwrap()[0]); // testing array, 1
    assert_eq!(Type::Text("other".to_string()),test_obj.get_value("array").unwrap().to_array().unwrap()[1]); // testing array, 2
    assert_eq!(Some(Type::Switch(false)),test_obj.get_value("switch")); // testing switch
    assert_eq!(None,test_obj.get_value("tester")); // testing a key that doesn't exist
  }

  #[test]
  fn set_value() {
    let mut test_obj = Settings::new(Configuration{});
    assert_eq!(test_obj.set_value("a.b.c.d","mortan").is_ok(),true);
    assert_eq!(test_obj.set_value("a.b.f",&4453).is_ok(),true);
    assert_eq!(test_obj.set_value("a.is_enabled",&true).is_ok(),true);

    assert_eq!(test_obj.get_value("a.b.c.d"),Some(Type::Text("mortan".to_string())));
    assert_eq!(test_obj.get_value("a.b.f"),Some(Type::Number(4453)));
    assert_eq!(test_obj.get_value("a.is_enabled"),Some(Type::Switch(true)));
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

  #[test]
  fn add() {
    let mut test_obj = Settings::new(Configuration{});
    assert_eq!(test_obj.set_value("user.name","snsvrno").is_ok(),true);
    assert_eq!(test_obj.set_value("other.stuff","what").is_ok(),true);
    assert_eq!(test_obj.set_value("other.thing","nope").is_ok(),true);

    let mut test_obj2 = Settings::new(Configuration{});
    assert_eq!(test_obj2.set_value("user.place","space").is_ok(),true);
    assert_eq!(test_obj2.set_value("other.thing","nothing").is_ok(),true);

    let test_obj3 = test_obj + test_obj2;

    //assert_eq!(test_obj3.get_value("other.thing"),Some(Type::Text("nothing".to_string())));
    //assert_eq!(test_obj3.get_value("other.stuff"),Some(Type::Text("what".to_string())));
    //assert_eq!(test_obj3.get_value("user.place"),Some(Type::Text("space".to_string())));
    //assert_eq!(test_obj3.get_value("user.name"),Some(Type::Text("snsvrno".to_string())));

  }
}