use Type;
use SupportedType;

use failure::Error;
use std::collections::HashMap;
use serde::ser::Serialize;

pub type SettingsRaw = HashMap<String,Type>;

pub trait Format {

    // need to be implemneted //////////////////////////////////////
    fn filename(&self) -> String;
    fn folder(&self) -> String;
    fn to_string<T>(&self,object:&T) -> Result<String,Error> where T : SupportedType + Serialize;
    fn from_str<T>(&self,buffer:&str) -> Result<SettingsRaw,Error> where T : Format + Clone;

    // have default implemntations ////////////////////////////////
    fn extension(&self) -> Option<String> { None }

    // functions that shouldn't generally need to be implemented //
    fn get_path(&self) -> String {
        if let Some(ext) = self.extension() {
            return format!("{}/{}.{}",self.folder(),self.filename(),ext);
        } else {
            return format!("{}/{}",self.folder(),self.filename());
        }
    }
}