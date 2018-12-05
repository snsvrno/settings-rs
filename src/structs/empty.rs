use Format;
use SettingsRaw;
use failure::Error;
use SupportedType;

#[derive(Clone)]
struct EmptyConfig { }
impl Format for EmptyConfig {
    fn filename(&self) -> String { "settingsfile.text".to_string() }
    fn folder(&self) -> String { "settingsfile".to_string() }

    fn from_str<T>(&self,_:&str) -> Result<SettingsRaw,Error> 
        where T : Format + Clone 
    {
        Err(format_err!("Not Implemented"))
    }

    fn to_string<T:Sized>(&self,_:&T) -> Result<String,Error>
        where T : SupportedType + serde::ser::Serialize, 
    {
        Ok("Not Implemented".to_string())
    }
}