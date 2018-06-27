//! ***Settingsfile*** is an easy to use settings file access library.
//!
//! This library creates an abstract layer on top of [Serde](https://serde.rs/) for easier
//! read and write access to configuration files encoded in various formats (supports all
//! serde compatible libraries, [some examples here](https://serde.rs/#data-formats)).
//!
//! ## Benefits
//! ***Settingsfile*** allows you to quickly read and write configuration files by an easy to
//! use api.
//!
//! ```rust
//! settings_file.get_value("user.name") 
//! // Some("snsvrno") or None
//! 
//! // or 
//!
//! settings_file.get_value_or("address.update_server","127.0.0.1") 
//! // "www.crates.io" or "127.0.0.1"
//! ```
//!
//! ***Settingsfile*** also is built around [Serde](https://serde.rs/) so tons of formats are 
//! already supported, all you need to do is find a [compatible library](https://serde.rs/#data-formats)
//! and implement the handful of required traits and you now have a robust, easy to use settings library.
//!
//! ## Examples
//! Look at the two examples, one using 
//! [TOML](https://github.com/snsvrno/settingsfile-rs/tree/master/tests/testing_with_toml.rs) 
//! and the other using 
//! [RON](https://github.com/snsvrno/settingsfile-rs/tree/master/tests/testing_with_ron.rs) 
//! to see how to get started.

#[macro_use]
extern crate serde_derive;
extern crate serde;

pub mod error; use error::Error;
pub mod types; use types::Type;
pub mod settings;

//pub type Setting = HashMap<String,Type>;
//pub type SettingResult = Result<Setting,Error>;

pub struct File<T> where T : Format + Clone {
  ioconfig : T 
}

pub trait Format {
  // need to be implemented
  fn filename(&self) -> String;
  fn folder(&self) -> String;
  fn from_str(&self,buffer:&str) -> Result<Type,Error>;
  fn to_string<T:?Sized>(&self,object:&T) -> Result<String,Error> where T : serde::ser::Serialize;

  // have default implementations
  fn extension(&self) -> Option<String> { None }
  fn transcode<T:?Sized>(&self,object:&T) -> Result<Type,Error> 
    where T : serde::ser::Serialize, 
  {
    match self.to_string(object) {
      Err(error) => Err(error),
      Ok(string) => self.from_str(&string)
    }
  }
}

impl<T> File<T> where T : Format + Clone{
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
    //! returns the filename for the configuration file
    //!
    //! this will either only be [Format::filename()](trait.Format.html) 
    //! or [Format::filename()](trait.Format.html) + [Format::extension()](trait.Format.html)
    //! depending on if [Format::extension()](trait.Format.html) is implemented or not.
    match Format::extension(&self.ioconfig) {
      Some(extension) => format!("{}.{}",
        Format::filename(&self.ioconfig),
        extension
      ),
      None => Format::filename(&self.ioconfig)
    }
  }

  pub fn decode_str(&self,buffer : &str) -> Result<Type,Error> {
    //! for testing only, shouldn't be used normally.
    //!
    //! decodes a string into an [Setting Type](type.SettingResult.html). Can return an [Error](error/enum.Error.html) on failure. 
    Format::from_str(&self.ioconfig,buffer)
  }

  pub fn encode_to_string<C>(&self,object:&C) -> Result<String,Error> 
    where C : serde::ser::Serialize,
  {
    //! for testing only, shouldn't be used normally.
    //!
    //! encodes the object to a [String] or [Error](error/enum.Error.html).
    Format::to_string(&self.ioconfig,object)
  }
}