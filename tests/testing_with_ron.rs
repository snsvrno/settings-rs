extern crate settingsfile;
use settingsfile::{Settings, SupportedType, SettingsRaw, Format};


#[macro_use] extern crate failure; use failure::Error;
extern crate ron;
extern crate serde;
extern crate tempfile;
use std::fs::File;
use std::io::{Seek, SeekFrom};

// setting up the configuration, this will tell `Settingsfile-rs` 
// how you want the physical file saved / setup.

#[derive(Clone)]
struct Configuration { }
impl Format for Configuration {
    fn filename(&self) -> String { "settings".to_string() }
    fn folder(&self) -> String { "program_app_folder".to_string() }

    fn from_str<T>(&self,buffer:&str) -> Result<SettingsRaw,Error> 
        where T : Format + Clone 
    {
        let result : Result<SettingsRaw,ron::de::Error> = ron::de::from_str(&buffer);
        
        println!("from_str result: {:?}",result);

        match result {
                Ok(result) => Ok(result),
                Err(error) => Err(format_err!("{}",error)),
        }
    }

    fn to_string<T:Sized>(&self,object:&T) -> Result<String,Error>
        where T : SupportedType + serde::ser::Serialize, 
    {
        let result : Result<String,ron::ser::Error> = ron::ser::to_string(object);
            
        println!("to_string result: {:?}",result);

        match result {
                Ok(result) => Ok(result),
                Err(error) => Err(format_err!("{}",error)),
        }
    }
}

// testing functionality.

#[test]
fn decoding_and_reencoding() {
    let mut test = Settings::new(Configuration{});
    assert!(test.set_value("user.name", "snsvrno").is_ok());

    // using a tempfile for testing, so we don't have to worry
    // about where or what to create it as.
    let mut tempfile : File = tempfile::tempfile().unwrap();
    // uses `save_to` here so we can tell it what buffer to use,
    // if you use `save` it will automatically save it to the file
    // defined in the configuration
    assert!(test.save_to(&mut tempfile).is_ok());
    // needed to reset the cursor to the front,
    // otherwise we will read from the end of the file 
    // and won't get any data.
    tempfile.seek(SeekFrom::Start(0)).unwrap(); 

    // now loading that temporary file that we just created
    let loaded_setting = Settings::create_from(&mut tempfile,Configuration{}).unwrap();
    assert_eq!(loaded_setting.get_value("user.name"),test.get_value("user.name"));

}