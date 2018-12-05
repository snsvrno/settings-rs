use Type;
use SupportedType;

use failure::Error;
use std::collections::HashMap;
use serde::ser::Serialize;

use dirs;
use std::env;

/// A convience type that is used to shorten the required return 
/// type for the `Format` trait implemnetations. 
/// 
/// This does not need to be used by the users of this library, 
/// though makes code a little shorter.
pub type SettingsRaw = HashMap<String,Type>;

/// Trait for defining the physical properties of a `Settings`
/// 
/// # Example Usage
/// 
/// This example uses the [ron](https://crates.io/crates/ron) crate
/// and is taken from the test 
/// [testing_with_ron](https://github.com/snsvrno/settingsfile-rs/blob/master/tests/testing_with_ron.rs)
/// 
/// ```rust
/// #[derive(Clone)]
/// struct EmptyConfig { }
/// 
/// // implementing the trait here, only doing the required methods
/// impl Format for EmptyConfig {
///     fn filename(&self) -> String { "config.ron".to_string() }
///     fn folder(&self) -> String { ".config/app".to_string() }
/// 
///     fn from_str<T>(&self,buffer:&str) -> Result<SettingsRaw,Error> 
///         where T : Format + Clone 
///     {
///         ron::de::from_str(&buffer)
///     }
/// 
///     fn to_string<T:Sized>(&self,object:&T) -> Result<String,Error>
///         where T : SupportedType + serde::ser::Serialize, 
///     {
///         ron::ser::to_string(object)
///     }
/// }
/// 
/// fn main() {
///     let settings = Settings::new(EmptyConfig{});
/// }
/// ```
pub trait Format {

    // need to be implemneted //////////////////////////////////////

    /// Should return the file name of the configuration file: 
    /// 
    /// ~/{.application_name}/***{file_name}***.{extension}
    /// 
    /// ```rust,ignore
    /// fn filename(&self) -> String {
    ///     "config".to_string()
    /// }
    /// ```
    /// 
    /// The entire name + extension can be used here as well if 
    /// you don't want to use the [extension()](#method.extension) function.
    /// 
    /// ```rust,ignore
    /// fn filename(&self) -> String {
    ///     "settings.toml".to_string()
    /// }
    /// ``` 
    fn filename(&self) -> String;

    /// Should return the folder name of the configuration file with 
    /// respect to the _%user_directory%_ environmental variable:
    ///    
    /// ~/***{.application_name}***/{file_name}.{extension} 
    /// 
    /// The '.' isn't automatically added, and you would need to include it if desired. 
    /// 
    /// If you want to nest into folders, you can do that too using either '/' or '\\'. 
    /// It will be converted and parsed per platform.
    /// 
    /// ~/***.config/console_app***/config
    /// 
    /// ```rust,ignore
    /// fn folder(&self) -> String {
    ///     ".config/console_app".to_string()
    /// }
    /// ``` 
    fn folder(&self) -> String;

    /// Returns the seralized form of the passed in `object`. 
    /// Because this uses ***Serde.rs*** the `object` must have the 
    /// `serde::de::Serialize` trait, and must also implement the 
    /// `settingsfile::SupportedType` trait.
    /// 
    /// Typically this is just a wrapped passthrough to the serde libray you are using. 
    /// Example using [ron-rs](https://github.com/alexcrichton/ron-rs):
    /// 
    /// ```rust,ignore
    /// fn to_string<T:?Sized>(&self,object:&T) -> Result<String,Error>
    ///   where T : settingsfile::SupportedType + serde::ser::Serialize,
    /// {
    ///   match ron::ser::to_string(object) {
    ///     Ok(string) => Ok(string),
    ///     Err(error) => Err(format_err!("{}",error))
    ///   }
    /// }
    /// ```
    /// 
    /// You can see a working example in the test in the codebase 
    /// [testing_with_ron](https://github.com/snsvrno/settingsfile-rs/blob/master/tests/testing_with_ron.rs)
    fn to_string<T>(&self,object:&T) -> Result<String,Error> where T : SupportedType + Serialize;
    
    /// The decoding function, will return a deserialized form 
    /// of a the `Settings` Rust Struct.
    /// 
    /// Typically this is a wrapped passthrough to the serde library you are using. 
    /// Example using [ron-rs](https://github.com/alexcrichton/ron-rs):
    /// 
    /// ```rust,ignore
    /// fn from_str<T>(&self,buffer:&str) -> Result<SettingsRaw,Error>
    ///   where T : Format + Clone
    /// {
    /// let result : Result<SettingsRaw,ron::de::Error> = ron::de::from_str(&buffer);
    ///   match result {
    ///     Ok(result) => Ok(result),
    ///     Err(error) => Err(format_err!("{}",error)),
    ///   }
    /// }
    /// ```
    /// 
    /// You can see a working example in the test in the codebase 
    /// [testing_with_ron](https://github.com/snsvrno/settingsfile-rs/blob/master/tests/testing_with_ron.rs)
    fn from_str<T>(&self,buffer:&str) -> Result<SettingsRaw,Error> where T : Format + Clone;

    // have default implemntations ////////////////////////////////
    fn extension(&self) -> Option<String> {
        //! Option to allow for an extension separate from filename,
        //! you can always put the extension in the filename if you 
        //! perfer.
        //! 
        //! ~/{.application_name}/{file_name}.{***extension***}
        //!  
        //! ```rust,ignore
        //! fn extension(&self) -> Option<String> {
        //!     Some("toml".to_string())
        //! }
        //! ```
        //! 
        //! If not defined then no extension will be used for the file.
        //! ***Settingsfile*** does not assume a file's format by its
        //! extension so this is just a matter of user preference.
         
        None
    }

    fn local_filename(&self) -> Option<String> {
        //! Option to allow for a different filename for a local
        //! file. only used with `ShadowSetting`. Functions the same 
        //! as `filename`, does not include an extension.
        
        None
    }

    fn local_extension(&self) -> Option<String> {
        //! Option to allow for an extension when using a different
        //! local file name. only used with `ShadowSetting`. Doesn't 
        //! do anything if `local_filename` is `None`
        
        None
    }

    // functions that shouldn't generally need to be implemented //
    fn get_path(&self) -> String {
        //! Will give the correct path depending on what was implemented
        //! in the configuration
  
        match dirs::home_dir() {
            // probably not the most graceful way to do this, but
            // i don't know a likely scenario where you won't have
            //
            None => "".to_string(),
            Some(mut dir) => {
                dir.push(format!("{}",self.folder()));
                dir.display().to_string()
            }
        }
    }

    // functions that shouldn't generally need to be implemented //
    fn get_path_and_file(&self) -> String {
        //! Will give the correct path including file depending on what was implemented
        //! in the configuration
  
        match dirs::home_dir() {
            // probably not the most graceful way to do this, but
            // i don't know a likely scenario where you won't have
            //
            None => "".to_string(),
            Some(mut dir) => {
                dir.push(format!("{}/{}",self.folder(),self.get_filename()));
                dir.display().to_string()
            }
        }
    }

    fn get_filename(&self) -> String {
        //! Returns the complete file name with or without
        //! the extension (if defined)
        
        if let Some(ext) = self.extension() {
            return format!("{}.{}",self.filename(),ext);
        } else {
            return self.filename();
        }
    }

    fn get_local_filename(&self) -> Option<String> {
        //! Assembles the local file name, will return
        //! `None` if local_filename is `None`
         
        match self.local_filename() {
            None => None,
            Some(localname) => {
                if let Some(ext) = self.local_extension() {
                    Some(format!("{}.{}",localname,ext))
                } else {
                    Some(localname)
                }
            }
        }
    }

    fn get_local_path_and_filename(&self) -> String {
        //! Returns the path where the local configuration
        //! would be.
        
        match env::current_dir() {
            // panics on error because I don't think this is a credible scenario
            // the user is in the current directory running this program, so
            // they should have access to it, this will most probably be used
            // for CLI applications and you must have permission to go inside 
            // a directory in order to run it.
            Err(_) => { panic!(); },
            Ok(mut path) => {
                if let Some(local_filename) = self.get_local_filename() {
                    path.push(local_filename);
                } else {
                    path.push(self.get_filename());
                }
                return path.display().to_string();
            }
        }
    }
}