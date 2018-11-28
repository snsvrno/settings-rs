
use Format;
use Type;
use SupportedType;

use std::ops::{Add,AddAssign};
use std::io::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use failure::Error;


#[derive(Serialize,Deserialize,Clone)]
pub struct Settings<T> where T : Format + Clone {
    // contains all the data. a hashmap of Type(s)
    parts : HashMap<String,Type>,
    // a non-editable array to allow for easy iteration, 
    // is regenerated on manipulation, contains the dot
    // location keys of everything in the array
    keys : Vec<String>,
    // the information of IO, where this file is located
    // and general details about the format.
    ioconfig: T,
}

impl<T> Settings<T> where T : Format + Clone {

    // initalizers //////////////////////////////////////////////////////////////////////////////////
    // functions for create new instances of Settings

    pub fn new(config : T) -> Settings<T> { 
        //! Creates an empty `Settings` from a configuration
        Settings { 
            parts : HashMap::new(), 
            keys : Vec::new(),
            ioconfig : config
        } 
    }

    pub fn from_flat(flat_hash : &Settings<T>) -> Settings<T> {
        //! creates a settings from a flatten `Settings`. A flat settings is a 
        //! `Settings` that doesn't have any `Type::Complex`, so there is only
        //! one level of depth.

        let mut new_hash = Settings::new(flat_hash.ioconfig.clone());

        for (key,value) in flat_hash.parts.iter() {
            // FIXME: do something with error / results when
            // setting a value here from a flat settings.
            let _ = new_hash.set_value(&key,&value);
        } 
        
        new_hash
    }

    fn generate_keys(hash : &HashMap<String,Type>) -> Vec<String> {
        //! creates a vector array of strings which are all the keys
        //! inside the input hash. The intent of this is to have a vec
        //! of keys along with the data so we can easiy iterate and keep
        //! track of where we are without having to worry about looking 
        //! track and regenerating the hash every iterator used.
        
        let mut keys : Vec<String> = Vec::new();

        let complex_type = Type::Complex(hash.clone());

        // assuming this will always work because we are creating a complex, 
        // and flattening a complex should always result in another complex
        // so this should be 100% safe to do (unwrap).
        for (k,_) in complex_type.flatten(None).to_complex().unwrap() {
            keys.push(k);
        }

        keys
    }

    // io - filesystem functions //////////////////////////////////////////////////////////////////
    // accessing stored versions of the Settings that isn't in memory.

    pub fn load_from(mut file : File, config : T) -> Result<Settings<T>,Error> {
        //! Loads the content of a `File` using the configuration. Doesn't use
        //! a path or doesn't infer the path from the config because this method
        //! is easier to do testing on to ensure everything behaves as expected

        // loads the raw file into a buffer
        let mut buf : String = String::new();
        file.read_to_string(&mut buf)?;

        // parses the string
        if buf.len() > 0 {
            let hash = Format::from_str::<T>(&config,&buf)?;
            Ok(Settings{ 
                parts : hash, 
                // TODO: fix the problem with referencing trates causing error E0283
                //  keys : Settings::generate_keys(&hash), 
                keys : Vec::new(), 
                ioconfig : config 
            })
        } else { 
            Ok(Settings{ parts: HashMap::new(), keys: Vec::new(), ioconfig : config })
        }
    }

    pub fn load_from_or_empty(file : File, config : T) -> Settings<T> {
        //! Wrapper around `load_from` so the return value isn't an result.
        //! loads a new setting object from a path, or creates an empty object 
        //! if file doesn't exist or errors in any way.

        match Settings::load_from(file,config.clone()) {
            Ok(settings) => { settings }
            Err(_) => { Settings::new(config.clone()) }
        }
    }

    pub fn save_to(&self, mut file : File) -> Result<(),Error> {
        //! saves the setting to a file buffer. Maybe done this way because 
        //! of ease of writing good tests?

        match self.ioconfig.to_string(&self.parts){
            Err(error) => return Err(error),
            Ok(settings_string) => {
                file.write_all(settings_string.as_bytes())?;
                Ok(())
            }
        }
    }

    // io - object functions ///////////////////////////////////////////////////////////////////
    // interactions with the `Settings` struct data

    pub fn get_value_absolute(&self, key_path : &str) -> Option<Type> {
        //! normally you should always use `get_value`, as it properly splits
        //! the key_path to get the correct value in the tree.
        //! if you are working with a flattend `Settings` then `get_value`
        //! will not work as it will attempt to split the key and it will find 
        //! nothing, this function will _NEVER_ split the key
        
        if let Some(result) = self.parts.get(key_path) {
            return Some(result.clone());
        } else {
            return None;
        }
    }

    pub fn get_value(&self, key_path : &str) -> Option<Type> {
        //! looks for a `key_path` in dot notation and returns an `Option` 
        //! containing the value if it exists.
        
        let path_tree : Vec<&str> = key_path.split(".").collect();
        let mut subtree : &Type = &Type::Text("Empty".to_string());

        // TODO: need to fix this in order to have full unicode support. need to use .chars() instead of slice.
        for i in 0..path_tree.len() {
            if i == 0 { 
                if let Some(ref part) = self.parts.get(&path_tree[i].to_string()) {
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

    pub fn set_value<A:?Sized>(&mut self, key_path : &str, value : &A) -> Result<(),Error> 
        where A : SupportedType ,
    {
        //! sets the value of a key, uses a generic that must implement the `SupportedType` trait
        
        let mut parts : Vec<Type> = Vec::new();
        let path_tree : Vec<&str> = key_path.split(".").collect();

        for i in 0..path_tree.len()-1 {
            if i == 0 {
                if let Some(part) = self.parts.remove(&path_tree[i].to_string()) {
                    if let Type::Complex(hash) = part { 
                        parts.push(Type::Complex(hash)); 
                    } else { parts.push(Type::Complex(HashMap::new())); }
                } else { parts.push(Type::Complex(HashMap::new())); }
            } else {
                let index = parts.len()-1;
                let mut push_me : Option<Type> = None;
                if let Type::Complex(ref mut mut_parts) = parts[index] {
                    if let Some(part) = mut_parts.remove(&path_tree[i].to_string()) {
                        if let Type::Complex(hash) = part { 
                            push_me = Some(Type::Complex(hash));
                        }
                    }
                }
                match push_me {
                    None => parts.push(Type::Complex(HashMap::new())),
                    Some(push_me) => parts.push(push_me)
                }
            }
        }

        if let Type::Complex(ref mut parts_two) = parts[path_tree.len()-2] {
            parts_two.insert(path_tree[path_tree.len()-1].to_string(),value.wrap());
        }

        
        // rebuilds the tree
        if parts.len() > 1 {
                for i in (1..parts.len()).rev() {
                        let temp_part = parts.remove(i);
                        if let Type::Complex(ref mut parts_minus_1) = parts[i-1] {
                            parts_minus_1.insert(path_tree[i].to_string(),temp_part);
                        }
                }        
        }

        self.parts.insert(path_tree[0].to_string(),parts.remove(0));

        // needs to recalculate the keys, so that if we are adding
        // a new value to a new key location it can be iterated about
        // TODO: fix the issue with Trait reference
        // self.keys = Settings::generate_keys(&self.parts);
        
        Ok(())
    }

    pub fn delete_key(&mut self, key_path : &str) -> Option<Type> {
        //! deletes the key and returns the current value, 
        //! returns none if the key didn't exist.
        
        let mut parts : Vec<Type> = Vec::new();
        let path_tree : Vec<&str> = key_path.split(".").collect();
        let mut returned_value : Option<Type> = None;

        for i in 0..path_tree.len()-1 {
            if i == 0 {
                if let Some(part) = self.parts.remove(&path_tree[i].to_string()) {
                    if let Type::Complex(hash) = part { 
                        parts.push(Type::Complex(hash)); 
                    } else { parts.push(Type::Complex(HashMap::new())); }
                } else { parts.push(Type::Complex(HashMap::new())); }
            } else {
                let index = parts.len()-1;
                let mut push_me : Option<Type> = None;
                if let Type::Complex(ref mut mut_parts) = parts[index] {
                    if let Some(part) = mut_parts.remove(&path_tree[i].to_string()) {
                        if let Type::Complex(hash) = part { 
                            push_me = Some(Type::Complex(hash));
                        }
                    }
                }
                match push_me {
                    None => parts.push(Type::Complex(HashMap::new())),
                    Some(push_me) => parts.push(push_me)
                }
            }
        }

        // if the parts length is one, then there was nothing to split
        // so we should just treat the key as an absolute path key
        // and go directly to the `HashMap<_,_>::remove()` function
        // to delete the key.
        if path_tree.len() == 1 {
            returned_value = self.parts.remove(key_path);
        } else {
            if let Type::Complex(ref mut parts_two) = parts[path_tree.len()-2] {
                returned_value = parts_two.remove(path_tree[path_tree.len()-1]);
            }
        }
        
        // rebuilds the tree, if there is a tree to rebuild (parts.len() > 1)
        if parts.len() > 1 {
                for i in (1..parts.len()).rev() {
                        let temp_part = parts.remove(i);
                        if let Type::Complex(ref mut parts_minus_1) = parts[i-1] {
                            parts_minus_1.insert(path_tree[i].to_string(),temp_part);
                        }
                }
            self.parts.insert(path_tree[0].to_string(),parts.remove(0));
        }

        // needs to recalculate the keys, so that if we are adding
        // a new value to a new key location it can be iterated about
        // TODO: fix the issue with Trait reference
        // self.keys = Settings::generate_keys(&self.parts);
        
        returned_value
    }

    // flatten related functions //////////////////////////////////////////////////////

    pub fn get_flat_hash(&self) -> Settings<T> {
        //! returns the flattened form of the ***Setting***, shortcut of `flatten()`
        //! and a member function

        Settings::flatten(self)
    }

    pub fn is_flat(&self) -> bool {
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
        for (_,value) in self.parts.iter() {
            if value.is_complex() { return false; }
        }
        
        // if we are still going, then look at the length, if there aren't
        // any parts then it shouldn't be considered flat because its empty.
        if self.parts.len() > 0 { 
            true
        } else {
            false
        }
    }

    pub fn flatten(hash_to_flatten : &Settings<T>) -> Settings<T> {
        //! used to flatten a `Settings`. Takes a `Settings` and removes all 
        //! `Type::Complex` into a noncomplex with a key using dot notation. 
        //! Refer to the explaination at `is_flat` to see what a flat `Settings` is

        let mut flat_hash : HashMap<String,Type> = HashMap::new(); // new hash to return at the end

        // iterates through all the `Types` in the `self.parts` of the `Settings`,
        // checks if each is a `Type::Complex`, if so then adds it to the flat_hash,
        // and if not then just adds the resulting type from `flatten()` to the 
        // flat_hash returner object.
        for (key,value) in hash_to_flatten.parts.iter() {
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
            parts : flat_hash, 
            // TODO: another issue with traits, fix and then return the bottom line
            //   keys : Settings::generate_keys(&flat_hash),
            keys : Vec::new(),
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

        for (key,value) in flat_other.parts.iter() {
            flat_self.parts.insert(key.to_string(),value.clone());
        } 

        Settings::from_flat(&flat_self)
    }
}

impl<T> AddAssign for Settings<T> where T : Format + Clone {
    
    fn add_assign(&mut self, other:Settings<T>) {
        //! `AddAssign` follows the same logic as `Add`, and allows you to use += with
        //! `Settings`
        
        let flat_other = other.get_flat_hash();

        for (key,value) in flat_other.parts.iter() {
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

        assert_eq!(test_obj.get_value("a.b.c.d"),Some(Type::Text("mortan".to_string())));
        assert_eq!(test_obj.get_value("a.b.f"),Some(Type::Int(4453)));
        assert_eq!(test_obj.get_value("a.is_enabled"),Some(Type::Switch(true)));
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
    fn deleting() {
        let mut setting = Settings::new(Configuration{});
        assert!(setting.set_value("user.name","the username").is_ok());
        assert!(setting.set_value("user.email","someone@someplace.com").is_ok());
        assert!(setting.set_value("software.version",&23).is_ok());
        assert!(setting.set_value("software.update_available",&false).is_ok());

        assert_eq!(setting.get_value("software.version"),Some(Type::Int(23)));

        assert_eq!(setting.delete_key("software.version"),Some(Type::Int(23)));
        setting.delete_key("user");
        assert_eq!(None,setting.get_value("software.version"));
        assert_eq!(None,setting.get_value("user.name"));
        assert_eq!(None,setting.get_value("user.email"));
    }


}

