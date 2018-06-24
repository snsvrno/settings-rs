#[derive(PartialEq,Debug)]
enum Filetype {
  Toml,
}

#[allow(dead_code)]
struct Settings {
  user_folder: String,
  settings_filename: String,
  settings_extension: Option<String>,
  settings_filetype: Filetype,
}

#[allow(dead_code)]
impl Settings {

  fn new(folder:&str, filename:&str, extension:Option<&str>, filetype:Option<Filetype>) -> Settings {
    Settings {
      user_folder : folder.to_string(),
      settings_filename: filename.to_string(),
      settings_extension: if let Some(ext) = extension { Some(ext.to_string()) } else { None }, // defaults to None
      settings_filetype: if let Some(filetype) = filetype { filetype } else { Filetype::Toml }, // defaults to TOML
    }
  }

  fn get_value(&self,key_path:&str) -> Option<String> {
    None
  }

  fn get_value_or(&self,key_path:&str,default_value:&str) -> String {
    match self.get_value(&key_path) {
      Some(value) => value.to_string(),
      None => default_value.to_string(),
    }
  }

  fn set_value(&self,key_path:&str,value:&str) -> bool {
    false
  }

  fn has_value(&self,key_path:&str) -> bool {
    match self.get_value(&key_path) {
      Some(_) => true,
      None => false
    }
  }

}



#[cfg(test)]
mod tests{
  #[test]
  fn basic_test() {
    use super::Settings;

    let test_settings = Settings::new(".test_project","settingsfile",None,None);
    assert_eq!(test_settings.settings_filetype,super::Filetype::Toml);
  }
}