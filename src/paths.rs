use std::path::PathBuf;
use std::env;
use std::ffi;

use structs::error::Error;

pub fn check_if_a_path(string : &str) -> bool {
  string.contains("/") || string.contains("\\")
}

pub fn get_absolute_path_from_str(string : &str) -> Result<PathBuf,Error> {
  let path = PathBuf::from(string);
  if path.is_absolute() { return Ok(path); }

  match env::current_dir() {
    Err(error) => return Err(Error::Error(error.to_string())),
    Ok(mut path) => {
      path.push(string);
      return Ok(compress_path(&path));
    }
  }
}

fn compress_path(path : &PathBuf) -> PathBuf{
  //! removes the ../../../ sections in a path

  let mut new_string : String = String::new();
  let mut parts : Vec<ffi::OsString> = Vec::new();
  
  for part in path.iter() {
    if part == ".." { parts.pop(); }
    else if part == "\\" { }
    else { parts.push(part.to_os_string()); }
  }

  for part in parts { new_string = format!("{}{}\\",new_string,part.to_str().unwrap()); }

  PathBuf::from(new_string)
}