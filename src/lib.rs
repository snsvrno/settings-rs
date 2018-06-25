#[macro_use]
extern crate serde_derive;
extern crate serde;

extern crate toml;

mod paths;
mod structs;
mod formats;

use structs::filetype::Filetype;
use structs::options::SettingsOptions;
use structs::error::Error;


use std::path::PathBuf;
use std::env;

pub struct Settings {
  user_folder: String,
  settings_filename: String,
  
  settings_extension: Option<String>,
  settings_filetype: Filetype,
  option_look_locally: bool
}

impl Settings {

  pub fn new(folder:&str, filename:&str, options : SettingsOptions) -> Settings {
    Settings {
      user_folder : folder.to_string(),
      settings_filename: filename.to_string(),

      // optionals
      option_look_locally : options.local_enabled,
      settings_extension: if let Some(ext) = options.extension { Some(ext.to_string()) } else { None }, // defaults to None
      settings_filetype: options.filetype, // defaults to TOML
    }
  }

  pub fn get_value(&self,key_path:&str) -> Option<String> {
    let setting_dat = self.for_read();
    setting_dat.get_value(&key_path)
  }

  pub fn get_value_specific(&self,key_path:&str,local:&bool) -> Option<String> {
    let setting_dat = self.for_write(&local);
    setting_dat.get_value(&key_path)
  }

  pub fn get_value_or(&self,key_path:&str,default_value:&str) -> String {
    match self.get_value(&key_path) {
      Some(value) => value.to_string(),
      None => default_value.to_string(),
    }
  }

  pub fn set_value(&self,key_path:&str,value:&str,local:&bool) -> Result<(),Error> {
    let mut setting_dat = self.for_write(&local);

    let mut value = value.to_string();

    // checks if value a directory, and then makes sure it is absolute path.
    if paths::check_if_a_path(&value) {
      if let Ok(path) = paths::get_absolute_path_from_str(&value) {
        value = path.display().to_string();
      }
    }

    setting_dat.set_value(&key_path, &value);

    match setting_dat.save_to(&self.for_write_path(&local),&self.settings_filetype) {
      Ok(_) => Ok(()),
      Err(error) => Err(error)
    }

  }

  pub fn has_value(&self,key_path:&str) -> bool {
    match self.get_value(&key_path) {
      Some(_) => true,
      None => false
    }
  }

  pub fn filename(&self) -> String {
    return self.settings_filename.to_string();
  }

  fn for_read(&self) -> structs::settings::Settings {
    match self.option_look_locally {
      false => structs::settings::Settings::load_from_or_empty(&self.get_path_global_file(),&self.settings_filetype),
      true => {
        let local = structs::settings::Settings::load_from_or_empty(&self.get_path_global_file(),&self.settings_filetype);
        let global = structs::settings::Settings::load_from_or_empty(&self.get_path_global_file(),&self.settings_filetype);
        global + local
      },
    }
  }

  fn for_write(&self,local:&bool) -> structs::settings::Settings {
    structs::settings::Settings::load_from_or_empty(&self.for_write_path(&local),&self.settings_filetype)
  }

  fn for_write_path(&self,local:&bool) -> PathBuf {
    match local {
      &false => self.get_path_global_file(),
      &true => self.get_path_local_file()
    }
  }

  fn get_path_global_file(&self) -> PathBuf {
    //! determines what the *global* file path should be.

    let mut home_dir = self.get_path_global_folder();
    home_dir.push(self.filename());
    home_dir
  }

  fn get_path_global_folder(&self) -> PathBuf {
    //! builds the *global* settings folder.

    let mut home_dir = env::home_dir().unwrap();
    home_dir.push(&self.user_folder);
    home_dir
  }

  fn get_path_local_file(&self) -> PathBuf {
    //! determines what the *local* file path should be.
    
    if let Ok(mut dir) = env::current_dir() {
      dir.push(self.filename());
      dir
    } else {
      let mut path = PathBuf::from(".");
      path.push(self.filename());
      path
    }
  }

}


#[cfg(test)]
mod tests{
  #[test]
  fn basic_test() {
    use super::Settings;

    let test_settings = Settings::new(".test_project","settingsfile",super::SettingsOptions::defaults());
    assert_eq!(test_settings.settings_filetype,super::Filetype::Toml);
  }
}