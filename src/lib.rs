use std::collections::HashMap;

#[macro_use]
extern crate serde_derive;
extern crate serde;

pub mod error; use error::Error;
pub mod types; use types::Type;

pub type Setting = HashMap<String,Type>;
pub type SettingResult = Result<Setting,Error>;

pub struct File<T> where T : Format {
  ioconfig : T 
}

pub trait Format {

  // need to be implemented
  fn filename(&self) -> String;
  fn folder(&self) -> String;
  fn from_str(&self,buffer:&str) -> SettingResult;
  fn to_string<T>(&self,object:&T) -> Result<String,Error> where T : serde::ser::Serialize;
  // have default implementations
  fn extension(&self) -> Option<String> { None }
}

impl<T> File<T> where T : Format {
  pub fn new(config : T) -> File<T> {
    File { ioconfig : config }
  }

  pub fn get_value(&self,key_path:&str) {
    println!("{}",key_path);
  }

  //pub fn set_value<B>(&self,key_path:&str,value : &B) -> String
  //  where B : for <'de> serde::de::Deserialize<'de> 
  //{
    // Format::to_string(&self.ioconfig,value)
  //}

  pub fn filename(&self) -> String {
    match Format::extension(&self.ioconfig) {
      Some(extension) => format!("{}.{}",
        Format::filename(&self.ioconfig),
        extension
      ),
      None => Format::filename(&self.ioconfig)
    }
  }

  pub fn decode_str(&self,buffer : &str) -> SettingResult {
    //! for testing only, shouldn't be used normally.
    Format::from_str(&self.ioconfig,buffer)
  }

  pub fn encode_to_string<C>(&self,object:&C) -> Result<String,Error> 
    where C : serde::ser::Serialize,
  {
    //! for testing only, shouldn't be used normally.
    Format::to_string(&self.ioconfig,object)
  }
}