/// ***Settingsfile*** is an easy to use settings file access library.
///
/// This library creates an abstract layer on top of [Serde](https://serde.rs/) for easier
/// read and write access to configuration files encoded in various formats (supports all
/// serde compatible libraries, [some examples here](https://serde.rs/#data-formats)).
///
/// ## Benefits
/// ***Settingsfile*** allows you to quickly read and write configuration files by an easy to
/// use api.
///
/// 
/// settings_file.get_value("user.name") 
/// // Some("snsvrno") or None
/// 
/// // or 
///
/// settings_file.get_value_or("address.update_server","127.0.0.1") 
/// // "www.crates.io" or "127.0.0.1"
/// 
///
/// ***Settingsfile*** also is built around [Serde](https://serde.rs/) so tons of formats are 
/// already supported, all you need to do is find a [compatible library](https://serde.rs/#data-formats)
/// and implement the handful of required traits and you now have a robust, easy to use settings library.
///
/// ## Examples
/// Look at the two examples, one using 
/// [TOML](https://github.com/snsvrno/settingsfile-rs/tree/master/tests/testing_with_toml.rs) 
/// and the other using 
/// [RON](https://github.com/snsvrno/settingsfile-rs/tree/master/tests/testing_with_ron.rs) 
/// to see how to get started.

#[macro_use]
extern crate serde_derive;
extern crate serde;

use std::path::PathBuf;
use types::SupportedType;
use std::collections::HashMap;
use std::env;

pub mod types; use types::Type;
pub mod error; use error::Error;
pub mod settings; use settings::Settings;

pub type PartsPackage = HashMap<String,Type>;

pub struct File<T> where T : Format + Clone {
  ioconfig : T 
}

pub trait Format {

  // need to be implemneted //////////////////////////////////////
  fn filename(&self) -> String;
  fn folder(&self) -> String;
  fn to_string<T>(&self,object:&T) -> Result<String,Error> where T : SupportedType + serde::ser::Serialize;
  fn from_str<T>(&self,buffer:&str) -> Result<PartsPackage,Error> where T : Format + Clone;

  /// have default implemntations ////////////////////////////////
  fn extension(&self) -> Option<String> { None }
}

impl<T> File<T> where T : Format + Clone{
  
  // constructors ////////////////////////////////////////////////

  pub fn new(config : T) -> File<T> {
    File { ioconfig : config }
  }

  // io functions ////////////////////////////////////////////////

  pub fn get_value(&self, key_path : &str) -> Result<Type,Error> {
    //! loads the settings from the file and then reads the value

    match self.get_raw_settings_for_read() {
      Err(error) => { return Err(error); }
      Ok(settings) => {
        match settings.get_value(&key_path) {
          None => return Err(Error::Blank),
          Some(value) => return Ok(value),
        }
      }
    }
  }

  pub fn get_value_or<A>(&self, key_path : &str, default : A) -> Type 
    where A : SupportedType, 
  {
    //! uses get_value() to get the value, but will return return the default value
    //! if no value is returned (for a failure to load and a blank).

    // TODO : add a log message for this failure.
    match self.get_value(&key_path) {
      Err(_) =>  SupportedType::wrap(&default),
      Ok(value) => value,
    }
  }

  pub fn set_value<A>(&self, key_path : &str, value : &A, local : bool) -> Result<(),Error>
    where A : SupportedType,
  {
    //! sets the value to the file, saves it immediately

    // TODO : have this return the previous value once I figure out how to do an unwrap function
    // let old_value = self.get_value_or(&key_path,());

    match self.get_raw_settings_for_write(local) {
      Err(error) => { return Err(error); }
      Ok(mut settings) => {
        match settings.set_value(&key_path,value) {
          Err(error) => { return Err(error); },
          Ok(_) => {
            match local {
              true => {
                match self.path_local() {
                  Err(error) => return Err(error),
                  Ok(path) => settings.save_to(&path)
                }
              },
              false => {
                match self.path_global() {
                  Err(error) => return Err(error),
                  Ok(path) => settings.save_to(&path)
                }
              }
            } 
          }
        }
      }
    }

  }

  //pub fn set_value<B>(&self,key_path:&str,value : &B) -> String
  //  where B : for <'de> serde::de::Deserialize<'de> 
  //{
    // Format::to_string(&self.ioconfig,value)
  //}

  // io functions ////////////////////////////////////////////////

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

  pub fn folder(&self) -> String {
    Format::folder(&self.ioconfig)
  }

  // private functions ///////////////////////////////////////////

  fn path_global(&self) -> Result<PathBuf,Error> {
    //! gets the global path

    match env::home_dir() {
      None => { return Err(Error::wrap("No home directory found?")); }
      Some(mut path) => {
        path.push(self.folder());
        path.push(self.filename());
        return Ok(path);
      }
    }
  }

  fn path_local(&self) -> Result<PathBuf,Error> {
    //! gets the local path, the path for a setting in the working directory

    match env::current_dir() {
      Err(error) => { return Err(Error::wrap(error)); },
      Ok(mut path) => {
        path.push(self.filename());
        return Ok(path);
      }
    }
  }

  fn get_raw_settings_for_read(&self) -> Result<Settings<T>,Error> {
    //! this function is for getting the combination (if enabled) of the local and global locations.

    let settings_local = self.get_raw_settings_for_write(true); 
    let settings_global = self.get_raw_settings_for_write(false);

    // makes sure we didn't have any errors
    if let Err(error) = settings_global { return Err(error); }
    if let Err(error) = settings_local { return Err(error); } 

    if let Ok(settings_local) = settings_local {
      if let Ok(settings_global) = settings_global {
        return Ok(settings_global + settings_local);
      }
    }

    return Err(Error::wrap("General read failure"));
  }

  fn get_raw_settings_for_write(&self,local : bool) -> Result<Settings<T>,Error> {
    //! this function is for writing the processed settings to a file, which means we can't combine settings files
    //! from multiple places because it will mix where the settings are.

    let path : Result<PathBuf,Error> = if local { self.path_local() } else { self.path_global() };
    match path { 
      Err(error) => { return Err(error); },
      Ok(path) => {
        match path.exists() {
          false => Ok(Settings::new(self.ioconfig.clone())),
          true => Settings::load_from(&path,self.ioconfig.clone())
        }
      }
    }
  }

  // functions for testing purposes only /////////////////////////

  pub fn decode_str(&self,buffer : &str) -> Result<PartsPackage,Error> {
    //! for testing only, shouldn't be used normally.
    //!
    //! decodes a string into an [Setting Type](type.SettingResult.html). Can return an [Error](error/enum.Error.html) on failure. 
    Format::from_str::<T>(&self.ioconfig,buffer)
  }

  pub fn encode_to_string<C>(&self,object:&C) -> Result<String,Error> 
    where C : SupportedType + serde::ser::Serialize,
  {
    //! for testing only, shouldn't be used normally.
    //!
    //! encodes the object to a [String] or [Error](error/enum.Error.html).
    Format::to_string::<C>(&self.ioconfig,object)
  }
}
