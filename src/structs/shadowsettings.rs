use Format;
use Type;
use Settings;
use SupportedType;

use std::fs;
use std::fs::File;
use failure::Error;

/// Complex settings that pulls data from 2 locations
/// 
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
/// `ShadowSettings` works the same as [Settings](struct.Settings.html)
/// except for the additional functions to `get` and `set` locally or 
/// globally. See [Settings](struct.Settings.html) or the 
/// [ron test](https://github.com/snsvrno/settingsfile-rs/blob/master/tests/testing_with_ron.rs)
/// to see how its used.
/// 
/// # Example
/// 
/// If you defined your configuration as a toml at
/// `~/.config/app/settings.toml` then it will look for a local
/// file in the current working directory called `settings.toml`.
/// 
/// You can use the `Format` trait to
/// allow hidden files (i.e. ".settings.toml") or even define an
/// alternate name for the local file (".myapp"). See 
/// [Format](trait.format.html) for more information.
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
        
        let global_path = self.ioconfig.get_path_and_file();
        if let Ok(mut file) = File::open(&global_path) {
            info!("Using {} for global file",global_path);
            self.load_global_from(&mut file)?;
        }

        let local_path = self.ioconfig.get_local_path_and_filename();
        if let Ok(mut file) = File::open(&local_path) {
            info!("Using {} for local file",local_path);
            self.load_local_from(&mut file)?;
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
         
        let global_path = self.ioconfig.get_path_and_file();
        // first makes sure all the directories exist before attempting to create
        // the file, so it has a place to make it
        fs::create_dir_all(self.ioconfig.get_path())?;
        // creates the file, now that we know the directory exists
        info!("Saving global to {}",global_path);
        let mut file = File::create(global_path)?;
        self.save_global_to(&mut file)?;

        if self.local.is_some() {
            let local_path = self.ioconfig.get_local_path_and_filename();
            info!("Saving local to {}",local_path);
            let mut local_file  = File::create(local_path)?;
            self.save_local_to(&mut local_file)?;
        }

        Ok(())
    }

    pub fn save_global_to(&self, file : &File) -> Result<(),Error> {
        self.global.save_to(file)
    }

    pub fn save_local_to(&self, file : &File) -> Result<(), Error> {
        if let Some(ref local) = self.local {
            local.save_to(file)
        } else { Ok(() )}
    }

    pub fn get_value(&self, key_path : &str) -> Option<Type> {
        
        if let Some(ref local) = self.local {
            match local.get_value(key_path) {
                None => self.global.get_value(key_path),
                Some(value) => { 
                    // here we are creating a new complex that is a 
                    // composite of the other two complexs (global,local)
                    match value {
                        Type::Complex(mut value) => {
                            if let Some(Type::Complex(global)) = self.global.get_value(key_path) {
                                for (k,v) in global {
                                    if value.get(&k).is_none() {
                                        value.insert(k, v);
                                    }
                                }
                            }
                            Some(Type::Complex(value))
                        },
                        _ => Some(value),
                    } 
                },
            }
        } else {
            self.global.get_value(key_path)
        }
    }

    pub fn get_value_or<A:?Sized>(&self, key_path : &str, default_value : &A) -> Type
        where A : SupportedType,
    {
        match self.get_value(key_path) {
            Some(value) => value,
            None => default_value.wrap(),
        }
    }
    
    pub fn get_value_local(&self, key_path : &str) -> Option<Type> {
        match self.local {
            Some(ref local) => local.get_value(key_path),
            None => None,
        } 
    }

    pub fn get_value_global(&self, key_path : &str) -> Option<Type> {
        self.global.get_value(key_path)
    }

    pub fn set_value_local<A:?Sized>(&mut self, key_path : &str, value : &A) -> Result<(),Error> 
        where A : SupportedType ,
    {
        match self.local {
            Some(ref mut local) => local.set_value(key_path,value),
            None => {
                // needs to make the local settings if they don't exist.
                // the user doesn't care if it doesn't exist yet, they
                // obviously want to use it because they are using this function
                let mut local = Settings::new(self.ioconfig.clone());
                let result = local.set_value(key_path,value);
                self.local = Some(local);
                result
            },
        } 
    }

    pub fn set_value_global<A:?Sized>(&mut self, key_path : &str, value : &A) -> Result<(),Error> 
        where A : SupportedType ,
    {
        self.global.set_value(key_path,value)
    }
    
    pub fn delete_key_local(&mut self, key_path : &str) -> Option<Type> {
        match self.local {
            Some(ref mut local) => local.delete_key(key_path),
            None => None,
        }
    }

    pub fn delete_key_global(&mut self, key_path : &str) -> Option<Type> {
        self.global.delete_key(key_path)
    }

    pub fn delete_file_global(&self) -> bool {
        self.global.delete_file()
    }

    pub fn delete_file_local(&self) -> bool {
        match fs::remove_file(self.ioconfig.get_local_path_and_filename()){
            Err(_) => false,
            Ok(_) => true,
        }
    }
}

// tests ////////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
mod tests {
    use SupportedType;
    use Format;
    use SettingsRaw;
    use Type;
    use ShadowSettings;

    use failure::Error;
    use std::collections::HashMap;

    // Dummy configuration, just enough to get it working.
    #[derive(Clone)]
    struct Configuration { }
    impl Format for Configuration {
        fn filename(&self) -> String { "".to_string() }
        fn folder(&self) -> String { "".to_string() }

        fn from_str<T>(&self,_:&str) -> Result<SettingsRaw,Error> where T : Format + Clone { 
            Ok(HashMap::<String,Type>::new())
        }
        fn to_string<T:?Sized>(&self,_:&T) -> Result<String,Error> where T : SupportedType { 
            Ok("unimplemented".to_string())
        }
    }

    #[test]
    fn set_and_get_value() {
        //! confirms set and get functionality, that basic reading and writing works
        
        let mut test_obj = ShadowSettings::new(Configuration{});
        // setting global settings
        assert_eq!(test_obj.set_value_global("a.b.c.d","mortan").is_ok(),true);
        assert_eq!(test_obj.set_value_global("a.b.c.e","bobby lee").is_ok(),true);
        assert_eq!(test_obj.set_value_global("a.b.f",&4453).is_ok(),true);
        assert_eq!(test_obj.set_value_global("a.is_enabled",&true).is_ok(),true);
        //setting local settings to override the global settings
        assert_eq!(test_obj.set_value_local("a.b.c.d",&false).is_ok(),true);
        // checking all the global settings are accessible
        assert_eq!(test_obj.get_value_global("a.b.c.d"),Some(Type::Text("mortan".to_string())));
        assert_eq!(test_obj.get_value_global("a.b.c.e"),Some(Type::Text("bobby lee".to_string())));
        assert_eq!(test_obj.get_value_global("a.b.f"),Some(Type::Int(4453)));
        assert_eq!(test_obj.get_value_global("a.is_enabled"),Some(Type::Switch(true)));
        // checking the local setting is accessible
        assert_eq!(test_obj.get_value_local("a.b.c.d"),Some(Type::Switch(false)));
        // checks shadowing.
        assert_eq!(test_obj.get_value("a.b.c.d"),Some(Type::Switch(false)));
        assert_eq!(test_obj.get_value("a.b.c.e"),Some(Type::Text("bobby lee".to_string())));
        assert_eq!(test_obj.get_value("a.b.f"),Some(Type::Int(4453)));
        assert_eq!(test_obj.get_value("a.is_enabled"),Some(Type::Switch(true)));
    }

    #[test]
    fn get_value_or() {
        let mut test_obj = ShadowSettings::new(Configuration{});

        assert_eq!(test_obj.set_value_global("a.b.c.d","mortan").is_ok(),true);
        assert_eq!(test_obj.set_value_global("a.b.c.e","bobby lee").is_ok(),true);

        assert_eq!(test_obj.get_value_or("a.b.c.d", "not going to be used"),Type::Text("mortan".to_string()));
        assert_eq!(test_obj.get_value_or("a.b.c.f", "will be used"),Type::Text("will be used".to_string()));
    }

    #[test]
    fn get_value_shadow_complex() {
        //! confirms that when you pull out a section of the settings from a 
        //! ShadowSettings you will get a shadowed version of the settings, 
        //! so it should be a SimpleSettings (Settings) with the correct 
        //! shadowing.

        let mut test_obj = ShadowSettings::new(Configuration{});
        // setting global settings
        assert_eq!(test_obj.set_value_global("a.b.c.d","mortan").is_ok(),true);
        assert_eq!(test_obj.set_value_global("a.b.c.e","bobby lee").is_ok(),true);
        // setting local settings
        assert_eq!(test_obj.set_value_local("a.b.c.e","lee bo").is_ok(),true);

        let other_setting = test_obj.get_value("a.b.c").unwrap().to_complex().unwrap();
        assert_eq!(other_setting.get("d"), Some(&Type::Text("mortan".to_string())));
        assert_eq!(other_setting.get("e"), Some(&Type::Text("lee bo".to_string())));
    }
}