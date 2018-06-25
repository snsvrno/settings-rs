use toml;

use std::collections::HashMap;

use structs::error::Error;
use structs::subsetting::Subsetting;

use serde::ser;

pub fn from_str(buffer : &str) -> Result<HashMap<String,Subsetting>,Error> {
  let hash : Result<HashMap<String,Subsetting>,toml::de::Error> = toml::from_str(&buffer);

  match hash {
    Ok(hash) => Ok(hash),
    Err(error) => Err(Error::Error(error.to_string())),
  } 
}

pub fn to_string<T: ?Sized>(value: &T) -> Result<String,Error>
  where T: ser::Serialize,
{
  match toml::to_string(&value) {
    Ok(result) => Ok(result),
    Err(error) => Err(Error::Error(error.to_string())),
  }
}