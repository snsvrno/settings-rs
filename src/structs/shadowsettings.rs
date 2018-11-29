/// The more complex `Settings` style struct that can be used, 
/// designed as a wrapper around `Settings` that allows for 
/// configuration 'Shadowing', which means you can override 
/// global settings with local settings.
/// 
/// `ShadowSettings` looks for two locations for configuration.
/// First the configuration is defined with the 'Configuration'
/// struct. Then `ShadowSettings` looks at the global location
/// (the same location as `Settings` would look) and then looks
/// for the same file in the current working directory. 
/// 
/// For example, if you defined your configuration as a toml at
/// "~/.config/app/settings.toml" then it will look for a local
/// file in the current working directory called "settings.toml".
/// 
/// You can use the "Configuration" struct / `Format` trait to
/// allow hidden files (i.e. ".settings.toml") or even define an
/// alternate name for the local file (".myapp")

use Format;
use Type;
use Settings;

use std::fs::File;
use failure::Error;

#[derive(Serialize,Deserialize,Clone)]
pub struct ShadowSettings<T> where T : Format + Clone {
    ioconfig: T,
    global : Settings<T>,
    local : Option<Settings<T>>,
}

impl<T> ShadowSettings<T> where T : Format + Clone {
    pub fn new(config : T) -> ShadowSettings<T> { 
        ShadowSettings {
            ioconfig : config.clone(),
            global : Settings::new(config.clone()),
            local : None
        }
    }

    pub fn create_from(mut file : &File, config : T) -> Result<ShadowSettings<T>,Error> {
        //! assumse global
        
        Ok(ShadowSettings {
            ioconfig : config.clone(),
            global : Settings::create_from(&mut file,config.clone())?,
            local : None,
        })
    }

    pub fn load(&mut self) -> Result<(),Error> {
        //! attempts to load both local and global
        
        if let Ok(mut file) = File::open(self.ioconfig.get_path()) {
            self.load_global_from(&mut file);
        }

        if let Ok(mut file) = File::open(self.ioconfig.get_local_path()) {
            self.load_local_from(&mut file);
        }

        Ok(())
    }

    pub fn load_global_from(&mut self, file : &mut File) -> Result<(),Error> {
        self.global = Settings::create_from(file, self.ioconfig.clone())?;
        Ok(())
    }

    pub fn load_local_from(&mut self, file : &mut File) -> Result<(),Error> {
        self.local = Some(Settings::create_from(file, self.ioconfig.clone())?);
        Ok(()) 
    }

    pub fn save(&self) -> Result<(),Error> {
        //! saves the setting to a file, uses the `save_to` buffer function
         
        let mut file = File::create(self.ioconfig.get_path())?;
        self.save_global_to(&mut file);

        if let Some(ref local) = self.local {
            let mut local_file  = File::create(self.ioconfig.get_local_path())?;
            self.save_local_to(&mut file);
        }

        Ok(())
    }

    pub fn save_global_to(&self, mut file : &File) -> Result<(),Error> {
        self.global.save_to(file)
    }

    pub fn save_local_to(&self, mut file : &File) -> Result<(), Error> {
        if let Some(ref local) = self.local {
            local.save_to(file)
        } else { Ok(() )}
    }

    pub fn get_value_absolute(&self, key_path : &str) -> Option<Type> {
        
        if let Some(ref local) = self.local {
            match local.get_value_absolute(key_path) {
                None => self.global.get_value_absolute(key_path),
                Some(value) => Some(value),
            }
        } else {
            self.global.get_value_absolute(key_path)
        }
    }

    pub fn get_value(&self, key_path : &str) -> Option<Type> {
        
        if let Some(ref local) = self.local {
            match local.get_value(key_path) {
                None => self.global.get_value(key_path),
                Some(value) => Some(value),
            }
        } else {
            self.global.get_value(key_path)
        }
    }

    // set value
    // delete value
}