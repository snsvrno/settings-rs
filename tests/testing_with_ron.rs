extern crate settingsfile;
use settingsfile::types::SupportedType;
use settingsfile::PartsPackage;
use settingsfile::types::Type;
use settingsfile::Format;
use settingsfile::error::Error;
use settingsfile::settings::Settings;
use std::collections::HashMap;

extern crate ron;
extern crate serde;
extern crate tempfile;

#[derive(Clone)]
struct Configuration { }
impl Format for Configuration {
  fn filename(&self) -> String { "settings".to_string() }
  fn folder(&self) -> String { "program_app_folder".to_string() }

  fn from_str<T>(&self,buffer:&str) -> Result<PartsPackage,Error> 
    where T : Format + Clone 
  {
    let result : Result<PartsPackage,ron::de::Error> = ron::de::from_str(&buffer);
    
    println!("from_str result: {:?}",result);

    match result {
      Ok(result) => Ok(result),
      Err(error) => Err(Error::Error(error.to_string())),
    }
  }

  fn to_string<T:Sized>(&self,object:&T) -> Result<String,Error>
    where T : SupportedType + serde::ser::Serialize, 
  {
    let result : Result<String,ron::ser::Error> = ron::ser::to_string(object);
      
    println!("to_string result: {:?}",result);

    match result {
      Ok(string) => Ok(string),
      Err(error) => Err(Error::Error(error.to_string()))
    }
  }
}

#[test]
fn decoding_and_reencoding() {
  let test = settingsfile::Settingsfile::new(Configuration{});
  let test_string = r#"{
    "boolean": true,
    "float": 8.2
  }"#;

  let mut test_map : HashMap<String,Type> = HashMap::new();
  test_map.insert("integer".to_string(),Type::Int(32));
  test_map.insert("switch".to_string(),Type::Switch(true));
  println!("{:?}",test_map);

  let decoded_hash = test.decode_str(&test_string).unwrap();
  assert_eq!(decoded_hash.get("boolean").unwrap().to_switch().unwrap(),true);
  assert_eq!(decoded_hash.get("float").unwrap().to_float().unwrap(),8.2);

  let encoded_hash = test.encode_to_string(&decoded_hash).unwrap();
  let decoded_hash2 = test.decode_str(&encoded_hash).unwrap();
  assert_eq!(decoded_hash,decoded_hash2);

}
