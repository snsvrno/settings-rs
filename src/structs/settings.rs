use std::ops::{Add,AddAssign};
use std::io::prelude::*;
use std::path::PathBuf;
use std::collections::HashMap;
use std::fs::{File,create_dir_all};

use structs::subsetting::Subsetting;
use structs::filetype::Filetype;
use structs::error::Error;

use formats::toml;

#[derive(Serialize,Deserialize)]
pub struct Settings {
  parts : HashMap<String,Subsetting>
}

impl Settings {
  pub fn new() -> Settings { Settings { parts : HashMap::new() } }

  pub fn from_flat(flat_hash : &HashMap<String,String>) -> Settings {
    let mut new_hash = Settings::new();

    for (key,value) in flat_hash.iter() {
      new_hash.set_value(&key,&value);
    }

    new_hash
  }

  pub fn load_from(path : &PathBuf, format : &Filetype) -> Result<Settings,Error> {
    //! loads a settings object from a path, returns error if can't

    // loads the raw file into a buffer
    let mut buf : String = String::new();
    match File::open(&path) {
      Err(error) => { 
        let err = Error::Error(format!(
          "Cannot open settings file {}: {}",
          path.display().to_string(),
          error.to_string()
        ));
         return Err(err); 
      },
      Ok(mut file) => { 
        match file.read_to_string(&mut buf) {
          Ok(_) => { },
          Err(error) => { 
            let err = Error::Error(format!(
              "Cannot read file {}: {}",
              path.display().to_string(),
              error.to_string()
            ));
            return Err(err)
          }
        } 
      }
    }

    // parses the string
    if buf.len() > 0 {
      let hash : Result<HashMap<String,Subsetting>,Error> = match format {
        &Filetype::Toml => toml::from_str(&buf),
      };

      match hash {
        Err(error) => return Err(error),
        Ok(parts) => { return Ok(Settings { parts: parts }); }
      }
    } else { 
      let err = Error::Error(format!(
        "File {} is empty?",
        path.display().to_string()
      ));
      Err(err) 
    }
  }
  pub fn load_from_or_empty(path : &PathBuf, format : &Filetype) -> Settings {
    //! loads a new setting object from a path, or creates an empty object it path doesn't exist

    match Settings::load_from(&path,&format) {
      Ok(settings) => { settings }
      Err(_) => { Settings::new() }
    }
  }

  pub fn save_to(&self, path : &PathBuf, format : &Filetype) -> Result<(),Error> {
    //! saves the setting object to a certain path

    let file_format_to_string : Result<String,Error> = match format {
      &Filetype::Toml => toml::to_string(&self.parts),
    };

    match file_format_to_string {
      Err(error) => return Err(error),
      Ok(settings_string) => { 
        // creates folder if doen't exist
        if let Some(path) = path.parent() {
          if !path.exists() { 
            if let Err(error) = create_dir_all(&path) { 
              let err = Error::Error(format!(
                "Cannot create folders for {}: {}",
                path.display().to_string(),
                error.to_string()
              )); 
              return Err(err);
            }
          }
        }

        let file = File::create(path);
        match file {
          Err(error) => { 
            let err = Error::Error(format!(
              "Cannot create file {}: {}",
              path.display().to_string(),
              error.to_string()
            )); 
            return Err(err);
          },
          Ok(mut file) => {
            match file.write_all(settings_string.as_bytes()) {
              Err(error) => { 
                let err = Error::Error(format!(
                  "Cannot write buffer to file {}: {}",
                  path.display().to_string(),
                  error.to_string()
                )); 
                return Err(err)
              }
              Ok(_) => { return Ok(()); }
            }
          }
        }
      }
    }
  }

  pub fn get_flat_hash(&self) -> HashMap<String,String> {
    Settings::flatten(&self.parts,None)
  }

  pub fn flatten(hash_to_flatten : &HashMap<String,Subsetting>, prefix : Option<String>) -> HashMap<String,String> {
    let mut flat_hash : HashMap<String,String> = HashMap::new();

    for (key,value) in hash_to_flatten.iter() {
      match value {
        &Subsetting::Complex(ref hash) => { 
          let sub_flat_hash = Settings::flatten(&hash,Some(key.to_string()));
          for (k2,v2) in sub_flat_hash.iter() {
            let index : String = if let Some(ref prefix) = prefix { format!("{}.{}",prefix,k2) } else { k2.to_string() };
            flat_hash.insert(index,v2.to_string());
          }
        },
        &Subsetting::Single(ref string) => { 
          let index : String = if let Some(ref prefix) = prefix { format!("{}.{}",prefix,key) } else { key.to_string() };
          flat_hash.insert(index,string.to_string());
        }
      }
    }


    flat_hash
  }

  pub fn as_subsetting_consume(self) -> Subsetting {
    Subsetting::Complex(self.parts)
  }

  pub fn get_value(&self, key : &str) -> Option<String> {

    if let Some(raw_part) = self.get_raw(&key) {
      match raw_part {
        Subsetting::Complex(ref _hash) => { return Some("is complex".to_string()); },
        Subsetting::Single(ref string) => { return Some(string.to_string()); }
      }
    }

    None
  }

  pub fn get_value_or(&self, key : &str, or_value : &str) -> String {
    match self.get_value(key) {
      None => { or_value.to_string() }
      Some(value) => { value.to_string() }
    }
  }

  pub fn get_raw(&self, key : &str) -> Option<Subsetting> {

    let path_tree : Vec<&str> = key.split(".").collect();
    let mut subtree : &Subsetting = &Subsetting::Single("Empty".to_string());

    for i in 0..path_tree.len() {
      if i == 0 { 
        if let Some(ref part) = self.parts.get(&path_tree[i].to_string()) {
          subtree = part;
        } else { return None }
      } else {
        match *subtree {
          Subsetting::Complex(ref hash) => { 
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

  pub fn set_value(&mut self, key : &str, value : &str) -> bool {
    let mut parts : Vec<HashMap<String,Subsetting>> = Vec::new();
    let path_tree : Vec<&str> = key.split(".").collect();
    
    for i in 0..path_tree.len()-1 {
        if i == 0 {
            if let Some(part) = self.parts.remove(&path_tree[i].to_string()) {
                if let Subsetting::Complex(hash) = part { parts.push(hash); } else { parts.push(HashMap::new()); }
            } else { parts.push(HashMap::new()); }
        } else {
            let index = parts.len()-1;
            if let Some(part) = parts[index].remove(&path_tree[i].to_string()) {
                if let Subsetting::Complex(hash) = part { parts.push(hash); } else { parts.push(HashMap::new()); }
            } else { parts.push(HashMap::new()); }
        }
    }
    
    parts[path_tree.len()-2].insert(path_tree[path_tree.len()-1].to_string(),Subsetting::Single(value.to_string()));
    
    // rebuilds the tree
    if parts.len() > 1 {
        for i in (1..parts.len()).rev() {
            let temp_part = parts.remove(i);
            parts[i-1].insert(path_tree[i].to_string(),Subsetting::Complex(temp_part));
        }    
    }

    self.parts.insert(path_tree[0].to_string(),Subsetting::Complex(parts.remove(0)));
    
    true
  }
}

impl Add for Settings {
  type Output = Settings;

  fn add(self, other: Settings) -> Settings {
    let mut flat_self = self.get_flat_hash();
    let flat_other = other.get_flat_hash();
    
    for (key,value) in flat_other.iter() {
      flat_self.insert(key.to_string(),value.to_string());
    } 

    Settings::from_flat(&flat_self)
  }
}

impl AddAssign for Settings {
  fn add_assign(&mut self, other:Settings) {
    let flat_other = other.get_flat_hash();

    for (key,value) in flat_other.iter() {
      self.set_value(&key,&value);
    }
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;
  use structs::subsetting::Subsetting;
  use structs::settings::Settings;
  
  #[test]
  fn get_value() {
    let mut test_hash : HashMap<String,Subsetting> = HashMap::new();
    test_hash.insert("test".to_string(),Subsetting::Single("value".to_string()));

    let test_obj = Settings { parts: test_hash };
    
    assert_eq!(Some("value".to_string()),test_obj.get_value("test"));
    assert_eq!(None,test_obj.get_value("tester"));
    assert_eq!("value",test_obj.get_value_or("test","nope"));
    assert_eq!("nope",test_obj.get_value_or("tester","nope"));
  }

  #[test]
  fn set_value() {
    let mut test_obj = Settings::new();
    test_obj.set_value("a.b.c.d","mortan");
    assert_eq!(test_obj.get_value("a.b.c.d"),Some("mortan".to_string()));
  }

  #[test]
  fn flatten() {
    let mut test_obj = Settings::new();
    test_obj.set_value("user.name","snsvrno");
    test_obj.set_value("user.place","space");
    test_obj.set_value("other.thing","nothing");

    let flat = test_obj.get_flat_hash();
    assert_eq!(Some(&"snsvrno".to_string()),flat.get("user.name"));
    assert_eq!(Some(&"space".to_string()),flat.get("user.place"));
    assert_eq!(Some(&"nothing".to_string()),flat.get("other.thing"));
  }

  #[test]
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

  }
}