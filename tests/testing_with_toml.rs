extern crate settingsfile;
use settingsfile::types::SupportedType;
use settingsfile::PartsPackage;
use settingsfile::types::Type;
use settingsfile::Format;
use settingsfile::error::Error;
use settingsfile::settings::Settings;

extern crate toml;
extern crate serde;

#[derive(Clone)]
struct Configuration { }
impl Format for Configuration {
  fn filename(&self) -> String { "settings".to_string() }
  fn folder(&self) -> String { "program_app_folder".to_string() }

  fn from_str<T>(&self,buffer:&str) -> Result<PartsPackage,Error> 
    where T : Format + Clone 
  {
    let result : Result<PartsPackage,toml::de::Error> = toml::from_str(&buffer);
    match result {
      Ok(result) => Ok(result),
      Err(error) => Err(Error::Error(error.to_string())),
    }
  }

  fn to_string<T:?Sized>(&self,object:&T) -> Result<String,Error>
    where T : SupportedType + serde::ser::Serialize, 
  {
    match toml::ser::to_string(object) {
      Ok(string) => Ok(string),
      Err(error) => Err(Error::Error(error.to_string()))
    }
  }
}

#[derive(Clone)]
struct Configuration2 { }
impl Format for Configuration2 {
  fn filename(&self) -> String { "settings".to_string() }
  fn folder(&self) -> String { "program_app_folder".to_string() }
  fn extension(&self) -> Option<String> { Some("toml".to_string()) }

  fn from_str<T>(&self,buffer:&str) -> Result<PartsPackage,Error> 
    where T : Format + Clone 
  {
    let result : Result<PartsPackage,toml::de::Error> = toml::from_str(&buffer);
    match result {
      Ok(result) => Ok(result),
      Err(error) => Err(Error::Error(error.to_string())),
    }
  }

  fn to_string<T:?Sized>(&self,object:&T) -> Result<String,Error> where T : serde::ser::Serialize + SupportedType {
    match toml::ser::to_string(object) {
      Ok(string) => Ok(string),
      Err(error) => Err(Error::Error(error.to_string()))
    }
  }
}

#[test]
fn basic_load_config1() {
  let test = settingsfile::File::new(Configuration{});
  assert_eq!(test.filename(),"settings");
}

#[test]
fn basic_load_config2() {
  let test = settingsfile::File::new(Configuration2{});
  assert_eq!(test.filename(),"settings.toml");
}

#[test]
fn decoding_and_reencoding() {
  let test = settingsfile::File::new(Configuration{});
  let test_string = r#"database = "192.168.1.1"
  other = 12332
  nextone = true"#;

  let decoded_hash = test.decode_str(&test_string).unwrap();
  assert_eq!(decoded_hash.get("database").unwrap().to_text().unwrap(),"192.168.1.1");
  assert_eq!(decoded_hash.get("other").unwrap().to_number().unwrap(),12332);
  assert_eq!(decoded_hash.get("nextone").unwrap().to_switch().unwrap(),true);

  let encoded_hash = test.encode_to_string(&decoded_hash).unwrap();
  let decoded_hash2 = test.decode_str(&encoded_hash).unwrap();
  assert_eq!(decoded_hash,decoded_hash2);

}