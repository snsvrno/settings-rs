
use Format;
use Type;
use SupportedType;

use std::ops::{Add,AddAssign};
use std::io::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::fs;
use failure::Error;

/// Basic one file settings
///
/// The main guts of `Settingsfile-rs`. The `Settings` struct
/// interacts with the file system to serialize / deserialize
/// configurations / settings and then allow easy navigation
/// and maniulation.
/// 
/// `Settings` only reads data from one source and doesn't do
/// any 'shadowing', so if you want to override settings based
/// on a local user configuration, use the 
/// [ShadowSettings](struct.ShadowSettings.html)
/// struct instead.
/// 
/// # Examples
/// 
/// ## General Use 
/// 
/// Say a file looked like this, and was defined in the Format Config
/// ```json
/// {
///     "user" : {
///         "name" : "user's name",
///         "email" : "user's email"
///     }
/// }
/// ```
/// We could access the data like this.
/// 
/// ```rust
/// # extern crate settingsfile;
/// use settingsfile::Settings;
/// use settingsfile::EmptyConfig; // dumb config for examples and testing
/// 
/// let app_settings = Settings::new(EmptyConfig{});
/// if let Some(value) = app_settings.get_value("user.name") {
///     // prints "username is user's name"
///     println!("username is {}",value); 
/// }
/// ```
/// 
/// And if we weren't sure if the user would set a certain value, but would need
/// something during running we can define a default value.
/// 
/// ```rust
/// # extern crate settingsfile;
/// use settingsfile::Settings;
/// use settingsfile::EmptyConfig; // dumb config for examples and testing
/// 
/// let app_settings = Settings::new(EmptyConfig{});
/// let font_size = app_settings.get_value_or("font.size",&12);
/// println!("Font size is {}",font_size); // will either be 12 or whatever the user sets.
/// ```
/// 
/// ## Data Validation
/// 
/// Rust is heavy on type safe, and so is `settingsfile`. By default a settings file can only 
/// contain the following types (defined in the [Type](enum.Type.html) enum)
/// 
/// - Text
/// - Int
/// - Float
/// - Switch
/// - Array (of any combination of above)
/// - HashMap (of any combination of above)
/// 
/// Every result of a `get` will return a [Type](enum.Type.html) that you will need to match in
/// order to use the data. The only except is when you print / format it, as `Type` implements
/// display.
/// 
/// Using the fontsize example above.
/// 
/// ```rust
/// # extern crate settingsfile;
/// use settingsfile::Settings;
/// use settingsfile::Type;
/// use settingsfile::EmptyConfig; // dumb config for examples and testing
/// 
/// let app_settings = Settings::new(EmptyConfig{});
/// if let Type::Int(size) = app_settings.get_value_or("font.size",&12) {
///     println!("Setting font size to {}",size);
/// } else {
///     println!("Setting font.size must be an int!");
/// }
/// ```
#[derive(Serialize,Deserialize,Clone)]
pub struct Settings<T> where T : Format + Clone {
    // contains all the data. a hashmap of Type(s)
    global : HashMap<String,Type>,
    // the information of IO, where this file is located
    // and general details about the format.
    ioconfig: T,
}

impl<T> Settings<T> where T : Format + Clone {

    // initalizers //////////////////////////////////////////////////////////////////////////////////
    // functions for create new instances of Settings

    pub fn new(config : T) -> Settings<T> { 
        //! Creates an empty `Settings` from a configuration.
        //! 
        //! Initally the settings doesn't have any data and needs
        //! to have data inserted, `set` or loaded, `::load()`.

        Settings { 
            global : HashMap::new(),
            ioconfig : config
        } 
    }

    pub fn new_and_load(config : T) -> Settings<T> {
        //! A onliner to create a `Settings` and load from the config location.
        //! 
        //! Basically the same thing as `.new()` and `.load()`. The main difference
        //! is you don't need to give your Settings mutability if you don't need it.
        //! 
        //! Will return an empty setting if it fails loading the file for some reason.
        //! A `warn!()` (from the [log](https://crates.io/crates/log) crate) will be
        //! used if the loading fails.
        
        let mut setting = Settings::new(config);
        if let Err(error) = setting.load() {
            warn!("{}",error);
        }
        setting
    }

    fn from_flat(flat_hash : &Settings<T>) -> Settings<T> {
        //! Creates a settings from a flatten `Settings`. A flat settings is a 
        //! `Settings` that doesn't have any `Type::Complex`, so there is only
        //! one level of depth.
        //! 
        //! Only used in `+` and `+=`

        let mut new_hash = Settings::new(flat_hash.ioconfig.clone());

        for (key,value) in flat_hash.global.iter() {
            if let Err(error) = new_hash.set_value(&key,&value) {
                warn!("Error setting {}:{}, {}",key,value,error);
            }
        } 
        
        new_hash
    }

    // io - filesystem functions //////////////////////////////////////////////////////////////////
    // accessing stored versions of the Settings that isn't in memory.

    pub fn create_from(mut file : &File, config : T) -> Result<Settings<T>,Error> {
        //! Loads the content of a `File` using the configuration, but doesn't use
        //! a path or doesn't infer the path from the config.
        //! 
        //! Primariy made for testing serializing and deserializing the struture, but
        //! can also because to force the `setting` to load from a file buffer

        // loads the raw file into a buffer
        let mut buf : String = String::new();
        file.read_to_string(&mut buf)?;

        // parses the string
        if buf.len() > 0 {
            let hash = Format::from_str::<T>(&config,&buf)?;
            Ok(Settings{ 
                global : hash,
                ioconfig : config
            })
        } else { 
            Ok(Settings{ global: HashMap::new(), ioconfig : config })
        }
    }

    pub fn create_from_or_empty(file : &File, config : T) -> Settings<T> {
        //! Wrapper around `load_from` so the return value isn't an result.
        //! loads a new setting object from a path, or creates an empty object 
        //! if file doesn't exist or errors in any way.

        match Settings::create_from(file,config.clone()) {
            Ok(settings) => { settings }
            Err(_) => { Settings::new(config) }
        }
    }

    pub fn load(&mut self) -> Result<(),Error> {
        //! Loads `Setting` data from the file defined in the configuration.
        //! 
        //! If nothing exists it throws an error. This shouldn't
        //! be used for initalizing a new `Settings`, look at `create` and 
        //! `create_from` for that.
        //! 
        //! _Will override the existing data of a `Setting`_
        
        let path = self.ioconfig.get_path_and_file();
        info!("Loading from {}",path);

        let mut file = File::open(&path)?;
        info!("{} loaded.",path);
        
        self.load_from(&mut file)
    }

    pub fn load_from(&mut self, file : &mut File) -> Result<(),Error> {
        //! Loads into the current `Setting` from a file.
        //! 
        //! _Will override the existing data of a `Setting`_

        // loads the raw file into a buffer
        let mut buf : String = String::new();
        file.read_to_string(&mut buf)?;

        // parses the string
        if buf.len() > 0 {
            let hash = Format::from_str::<T>(&self.ioconfig,&buf)?;
            self.global = hash;
            Ok(())
        } else {
            Err(format_err!("Error loading from buffer"))
        }
    }

    pub fn save(&self) -> Result<(),Error> {
        //! Saves the setting to a file defined in the configuraton.

        let path = self.ioconfig.get_path_and_file();
        info!("Saving to {}",path);
        // first makes sure all the directories exist before attempting to create
        // the file, so it has a place to make it
        fs::create_dir_all(self.ioconfig.get_path())?;
        // creates the file, now that we know the directory exists
        let mut file = File::create(path)?;
        self.save_to(&mut file)
    }

    pub fn save_to(&self, mut file : &File) -> Result<(),Error> {
        //! saves the setting to a file buffer.

        match self.ioconfig.to_string(&self.global){
            Err(error) => return Err(error),
            Ok(settings_string) => {
                match file.write(settings_string.as_bytes()){
                    Ok(_) => Ok(()),
                    Err(error) => Err(format_err!("{}",error)),
                }
            }
        }
    }

    // io - object functions ///////////////////////////////////////////////////////////////////
    // interactions with the `Settings` struct data

    
    #[allow(dead_code)]
    fn get_value_absolute(&self, key_path : &str) -> Option<Type> {
        //! Get the value of the `key_path`, but doesn't split the `key_path`.
        //! 
        //! Normally you should always use `get_value`, as it properly splits
        //! the key_path to get the correct value in the tree.
        //! if you are working with a flattend `Settings` then `get_value`
        //! will not work as it will attempt to split the key and it will find 
        //! nothing, this function will _NEVER_ split the key
        
        if let Some(result) = self.global.get(key_path) {
            return Some(result.clone());
        } else {
            return None;
        }
    }

    pub fn get_value(&self, key_path : &str) -> Option<Type> {
        //! Get the saved value inside of a `Setting`
        //! 
        //! Looks for a `key_path` in dot notation and returns an `Option` 
        //! containing the value if it exists.
        
        let path_tree : Vec<&str> = key_path.split(".").collect();
        let mut subtree : &Type = &Type::Text("Empty".to_string());

        // TODO: need to fix this in order to have full unicode support. 
        // need to use .chars() instead of slice.
        for i in 0..path_tree.len() {
            if i == 0 { 
                if let Some(ref part) = self.global.get(&path_tree[i].to_string()) {
                    subtree = part;
                } else { return None }
            } else {
                match *subtree {
                    Type::Complex(ref hash) => { 
                        if let Some(ref part) = hash.get(&path_tree[i].to_string()) {
                            subtree = part;
                        } else { return None }
                    },
                    _ => { return None }
                }
            }
        }

        return Some(subtree.clone());
    }

    pub fn get_value_or<A:?Sized>(&self, key_path : &str, default_value : &A) -> Type
        where A : SupportedType, 
    {
        //! Wraps `get_value` so instead of an `Option` the result will always be a type.
        //! 
        //! The default value can be any type that implements [SupportedType](traits.SupportedType.html)
        
        match self.get_value(key_path) {
            Some(value) => value,
            None => default_value.wrap(),
        }
    }

    pub fn set_value<A:?Sized>(&mut self, key_path : &str, value : &A) -> Result<(),Error> 
        where A : SupportedType ,
    {
        //! sets the value of a key, uses a generic that must implement
        //! the [SupportedType](traits.SupportedType.html) trait
        
        let mut global : Vec<Type> = Vec::new();
        let path_tree : Vec<&str> = key_path.split(".").collect();

        // goes through the split up key_path
        // will run even if there is only 1 element in the split
        // path.
        for i in 0..path_tree.len()-1 {
            // if this is the first part then we want to initalize
            // all the elements because we will be going into this element
            // deeper with each step down the key_path 
            if i == 0 {
                // checks if this is part of an existing setting,
                // if it is then it will add it with pull it out of the setting
                // and adde it to the global vector
                if let Some(part) = self.global.remove(&path_tree[i].to_string()) {
                    if let Type::Complex(hash) = part { 
                        global.push(Type::Complex(hash)); 
                    } else { global.push(Type::Complex(HashMap::new())); }
                // if this doesn't exist then we will create a new item.
                } else { global.push(Type::Complex(HashMap::new())); }
            // now for the rest we can work with the existing object
            // we pulled out where `i == 0`
            } else {
                let index = global.len()-1; // the last element
                let mut push_me : Option<Type> = None;
                // checks if its a complex object, because then we need to add to it
                //, if it isn't a complex then we will override whatever is there with
                // a new blank complex.
                if let Type::Complex(ref mut mut_parts) = global[index] {
                    if let Some(part) = mut_parts.remove(&path_tree[i].to_string()) {
                        if let Type::Complex(hash) = part { 
                            push_me = Some(Type::Complex(hash));
                        }
                    }
                }
                // the above section pulled out the hashmap that exists (if one exists)
                // and places it in the `push_me` variable, i believe I did this because
                // of access rights / borrow checker.
                match push_me {
                    None => global.push(Type::Complex(HashMap::new())),
                    Some(push_me) => global.push(push_me)
                }
            }
        }

        // inserts the desired value into the tree, so we can rebuild it and insert it
        global.push(value.wrap());
        
        // rebuilds the tree
        if global.len() > 1 {
            for i in (1..global.len()).rev() {
                let temp_part = global.remove(i);
                if let Type::Complex(ref mut parts_minus_1) = global[i-1] {
                    parts_minus_1.insert(path_tree[i].to_string(),temp_part);
                }
            }        
        }

        // inserts the last part of the global list into the 
        // settings
        self.global.insert(path_tree[0].to_string(),global.remove(0));

        Ok(())
    }

    pub fn delete_key(&mut self, key_path : &str) -> Option<Type> {
        //! Deletes the key and returns the current value, 
        //! returns none if the key didn't exist.
        
        let mut global : Vec<Type> = Vec::new();
        let path_tree : Vec<&str> = key_path.split(".").collect();
        let mut returned_value : Option<Type> = None;

        for i in 0..path_tree.len()-1 {
            if i == 0 {
                if let Some(part) = self.global.remove(&path_tree[i].to_string()) {
                    if let Type::Complex(hash) = part { 
                        global.push(Type::Complex(hash)); 
                    } else { global.push(Type::Complex(HashMap::new())); }
                } else { global.push(Type::Complex(HashMap::new())); }
            } else {
                let index = global.len()-1;
                let mut push_me : Option<Type> = None;
                if let Type::Complex(ref mut mut_parts) = global[index] {
                    if let Some(part) = mut_parts.remove(&path_tree[i].to_string()) {
                        if let Type::Complex(hash) = part { 
                            push_me = Some(Type::Complex(hash));
                        }
                    }
                }
                match push_me {
                    None => global.push(Type::None),
                    Some(push_me) => global.push(push_me)
                }
            }
        }

        // if the global length is one, then there was nothing to split
        // so we should just treat the key as an absolute path key
        // and go directly to the `HashMap<_,_>::remove()` function
        // to delete the key.
        if path_tree.len() == 1 {
            returned_value = self.global.remove(key_path);
        } else if global.len() > 0 && path_tree.len() > 0 {
            let index = global.len()-1;
            if let Type::Complex(ref mut parts_two) = global[index] {
                returned_value = parts_two.remove(path_tree[path_tree.len()-1]);
            }
        }

        if global.len() > 0 {
            self.global.insert(path_tree[0].to_string(),global.remove(0));
        }
        
        returned_value
    }

    pub fn delete_file(&self) -> bool {
        //! Deletes the physical file from the disk
        
        let path = self.ioconfig.get_path_and_file();
        info!("Deleting {}",path);
        match fs::remove_file(path){
            Err(_) => false,
            Ok(_) => true,
        }
    }

    pub fn keys(&self) -> Vec<String> {
        let mut keys : Vec<String> = Vec::new();
        let flat = Settings::flatten(&self);
        
        for k in flat.global.keys() {
            keys.push(k.to_string());
        }

        keys
    }

    // flatten related functions //////////////////////////////////////////////////////

    fn get_flat_hash(&self) -> Settings<T> {
        //! returns the flattened form of the ***Setting***, shortcut of `flatten()`
        //! and a member function

        Settings::flatten(self)
    }

    #[allow(dead_code)]
    fn is_flat(&self) -> bool {
        //! checks if the settings file is flat
        //!
        //! a flat ***Settings*** is defined by no ***Type*** being `Type::Complex`.
        //! In basic terms it is a `HashMap` that has a depth of '1' and all the 
        //! `keys` are actually `key_paths`.
        //!
        //! example of a flat ***Settings*** in JSON format.
        //!
        //! ```json
        //! // example flat settings
        //! { 
        //!     "user.name" : "snsvrno",
        //!     "user.brightness" : 123,
        //!     "program.default.storage" : "C:",
        //!     "user.path" : [ "~/bin" , "~/.cargo/bin" ]
        //! }
        //! ```
        //!
        //! and the same example in not in a flat form
        //!
        //! ```json
        //! {
        //!     "user" : {
        //!         "name" : "snsvrno",    
        //!         "brightness" : 123,
        //!         "path" : [
        //!             "~/bin",
        //!             "~/.cargo/bin"
        //!         ],
        //!     },    
        //!     "program" : {
        //!         "default" : {
        //!             "storage" : "C:"
        //!         }
        //!     }
        //! }

        // checks if any of the part values are a complex,
        // which then means this can't be flat because a flat
        // has a depth of 1 and a complex has its own depth + current
        // depth which is > 1.
        for (_,value) in self.global.iter() {
            if value.is_complex() { return false; }
        }
        
        // if we are still going, then look at the length, if there aren't
        // any global then it shouldn't be considered flat because its empty.
        if self.global.len() > 0 { 
            true
        } else {
            false
        }
    }

    fn flatten(hash_to_flatten : &Settings<T>) -> Settings<T> {
        //! used to flatten a `Settings`. Takes a `Settings` and removes all 
        //! `Type::Complex` into a noncomplex with a key using dot notation. 
        //! Refer to the explaination at `is_flat` to see what a flat `Settings` is

        let mut flat_hash : HashMap<String,Type> = HashMap::new(); // new hash to return at the end

        // iterates through all the `Types` in the `self.global` of the `Settings`,
        // checks if each is a `Type::Complex`, if so then adds it to the flat_hash,
        // and if not then just adds the resulting type from `flatten()` to the 
        // flat_hash returner object.
        for (key,value) in hash_to_flatten.global.iter() {
            let temp_type : Type = value.flatten(Some(key.to_string()));
            if let Type::Complex(hash) = temp_type {
                for (k2,v2) in hash.iter() {
                    flat_hash.insert(k2.to_string(),v2.clone());
                }
            } else {
                flat_hash.insert(key.to_string(),temp_type);
            }
        }

        Settings { 
            global : flat_hash,
            ioconfig : hash_to_flatten.ioconfig.clone() 
        }
    }
}

// other implementations /////////////////////////////////////////////////////////////////

impl<T> Add for Settings<T> where T : Format + Clone {
    type Output = Settings<T>;

    fn add(self, other: Settings<T>) -> Settings<T> {
        //! implementing `add` so you should be able to use '+' on two settings, useful
        //! because you may want to combine settings from different locations.
        //! 
        //! Adding to `Settings` works by overlaying the two `Settings` on top of each
        //! other, if the same "key" has multiple "values", the "self" is overwritten with
        //! the "other". So Adding a `Settings` means you are overlaying it ontop of the
        //! existing data.
    
        let mut flat_self = self.get_flat_hash();
        let flat_other = other.get_flat_hash();

        for (key,value) in flat_other.global.iter() {
            flat_self.global.insert(key.to_string(),value.clone());
        } 

        Settings::from_flat(&flat_self)
    }
}

impl<T> AddAssign for Settings<T> where T : Format + Clone {
    
    fn add_assign(&mut self, other:Settings<T>) {
        //! `AddAssign` follows the same logic as `Add`, and allows you to use += with
        //! `Settings`
        
        let flat_other = other.get_flat_hash();

        for (key,value) in flat_other.global.iter() {
            let _ = self.set_value(&key,&value);
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
    use Settings;

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
        
        let mut test_obj = Settings::new(Configuration{});
        assert_eq!(test_obj.set_value("a.b.c.d","mortan").is_ok(),true);
        assert_eq!(test_obj.set_value("a.b.f",&4453).is_ok(),true);
        assert_eq!(test_obj.set_value("a.is_enabled",&true).is_ok(),true);
        assert_eq!(test_obj.set_value("single",&true).is_ok(),true);

        assert_eq!(test_obj.get_value("a.b.c.d"),Some(Type::Text("mortan".to_string())));
        assert_eq!(test_obj.get_value("a.b.f"),Some(Type::Int(4453)));
        assert_eq!(test_obj.get_value("a.is_enabled"),Some(Type::Switch(true)));
        assert_eq!(test_obj.get_value("single"),Some(Type::Switch(true)));
    }

    #[test]
    fn get_value_or() {
        let mut test_obj = Settings::new(Configuration{});

        assert_eq!(test_obj.set_value("a.b.c.d","mortan").is_ok(),true);
        assert_eq!(test_obj.set_value("a.b.c.e","bobby lee").is_ok(),true);

        assert_eq!(test_obj.get_value_or("a.b.c.d", "not going to be used"),Type::Text("mortan".to_string()));
        assert_eq!(test_obj.get_value_or("a.b.c.f", "will be used"),Type::Text("will be used".to_string()));
    }

    #[test]
    fn add() {
        //! confirms addition of two settings works.
        //! the only current (maybe) issues with this is that it consumes the 
        //! `Settings` during the operations
        
        let mut test_obj = Settings::new(Configuration{});
        assert_eq!(test_obj.set_value("other.count",&23).is_ok(),true);
        assert_eq!(test_obj.set_value("other.thing",&false).is_ok(),true);

        let mut test_obj2 = Settings::new(Configuration{});
        assert!(test_obj2.set_value("user.place","space").is_ok());
        assert!(test_obj2.set_value("other.thing",&132.23).is_ok());

        let test_obj3 = test_obj.clone() + test_obj2.clone();

        assert_eq!(test_obj3.get_value("other.thing"),Some(Type::Float(132.23)));
        assert_eq!(test_obj3.get_value("other.count"),Some(Type::Int(23)));
        assert_eq!(test_obj3.get_value("user.place"),Some(Type::Text("space".to_string())));

        let test_obj3 = test_obj2.clone() + test_obj.clone();

        assert_eq!(test_obj3.get_value("other.thing"),Some(Type::Switch(false)));
        assert_eq!(test_obj3.get_value("other.count"),Some(Type::Int(23)));
        assert_eq!(test_obj3.get_value("user.place"),Some(Type::Text("space".to_string())));

        test_obj += test_obj2;

        assert_eq!(test_obj.get_value("other.thing"),Some(Type::Float(132.23)));
        assert_eq!(test_obj.get_value("other.count"),Some(Type::Int(23)));
        assert_eq!(test_obj.get_value("user.place"),Some(Type::Text("space".to_string())));

    }

    #[test]
    fn flattening() {
        //! test flattening and flattening detection
        let mut flat_gen = Settings::new(Configuration{});
        assert!(flat_gen.set_value("user.name","the username").is_ok());
        assert!(flat_gen.set_value("user.email","someone@someplace.com").is_ok());
        assert!(flat_gen.set_value("software.version",&23).is_ok());
        assert!(flat_gen.set_value("software.update_available",&false).is_ok());
        assert!(flat_gen.is_flat() == false);

        let flat = flat_gen.get_flat_hash();
        assert!(flat.is_flat() == true);
        assert_eq!(flat.get_value("user"),None);
        assert_eq!(flat.get_value("software"),None);
        // using this function to get the absolute key, so we don't try and split it
        assert_eq!(flat.get_value_absolute("user.name"),Some(Type::Text("the username".to_string())));
        assert_eq!(flat.get_value_absolute("software.version"),Some(Type::Int(23)));

        let fluff = Settings::from_flat(&flat);
        assert!(fluff.is_flat() == false);
        assert_eq!(fluff.get_value_absolute("user.name"),None);
        assert_eq!(fluff.get_value("user.name"),Some(Type::Text("the username".to_string())));
        assert_eq!(fluff.get_value_absolute("software.version"),None);
        assert_eq!(fluff.get_value("software.version"),Some(Type::Int(23)));


    }

    #[test]
    fn keys() {
        let mut flat_gen = Settings::new(Configuration{});
        assert!(flat_gen.set_value("user.name","the username").is_ok());
        assert!(flat_gen.set_value("user.email","someone@someplace.com").is_ok());
        assert!(flat_gen.set_value("software.version",&23).is_ok());
        assert!(flat_gen.set_value("software.update_available",&false).is_ok());

        let mut count = 0;
        let mut total_count = 0;
        for k in flat_gen.keys() {
            if k == "user.name" { count +=1; }
            if k == "user.email" { count +=1; }
            if k == "software.version" { count +=1; }
            if k == "software.update_available" { count +=1; }
            total_count += 1;
        }
        assert!(count == 4);
        assert!(total_count == 4);
    }

    #[test]
    fn deleting() {
        let mut setting = Settings::new(Configuration{});
        assert!(setting.set_value("user.name","the username").is_ok());
        assert!(setting.set_value("user.email","someone@someplace.com").is_ok());
        assert!(setting.set_value("software.version",&23).is_ok());
        assert!(setting.set_value("software.update_available",&false).is_ok());
        // checks one key is set properly
        assert_eq!(setting.get_value("software.version"),Some(Type::Int(23)));
        // deletes
        assert_eq!(setting.delete_key("software.version"),Some(Type::Int(23)));
        // checks only the delete key is gone
        assert_eq!(None,setting.get_value("software.version"));
        assert_eq!(setting.get_value("software.update_available"),Some(Type::Switch(false)));
        assert_eq!(setting.get_value("user.email"),Some(Type::Text("someone@someplace.com".to_string())));
        assert_eq!(setting.get_value("user.name"),Some(Type::Text("the username".to_string())));
        // deleting a subgroup
        setting.delete_key("user");
        // checking if the right stuff is gone and the rest is still there.
        assert_eq!(None,setting.get_value("software.version"));
        assert_eq!(None,setting.get_value("user.name"));
        assert_eq!(None,setting.get_value("user.email"));
        assert_eq!(setting.get_value("software.update_available"),Some(Type::Switch(false)));
    }

}

