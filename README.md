# Settingsfile-rs
A library for easily accessing settings and configuration files using [Serde](https://serde.rs/).

## Brief 
***Settingsfile-rs*** attempts to abstract away defining, parsing, and saving configuration files for cli and other programs. Taking the heavy lifting away so you can focus on making an easy to use program with a robust set of organized configuration options.

## Features
***Settigsfile-rs*** uses a 'tree' method for organizing settings files. This means you can organise different options in groups, allowing you to 'go down the rabit hole' with options and configurations and have it easy for your users to tweak, customize, and have everything exactly as you like it.

### Nesting
Each configuration can store properties in a tree, with multiple branches, here is basic example (in JSON)
```json
{
  "user" : {
    "name" : "the user's name",
    "email" : "the user's email",
    "key" : "SDH23UIRWDFHIJSDJF234IOF"
  },
  "display" : {
    "tab-spaces": 2,
    "options" : [ "green", "auto-break" ]
  }
}
```

As you can see, you can organize different settings by their function, and you can have multiple types used, so numbers, lists, or booleans.

### Shadowing
When looking for a settings, ***Settingsfile-rs*** looks in 2 places by default. First is the local working directory, and second is the defined place in the configuration. This allows the user to set per folder / per project settings. A good example with git could be a project might be personal so you need your personal key, but if its a work project then you need to use your work key for that organization.

***Settingsfile-rs*** automatically shadows the global setting (defined in the configuration) with the local file (in the working folder) so those will override your global settings. This way the user can define specific settings they may want in this working folder, but not effect the global settings. 


## Usage
Add ***Settingsfile-rs*** to your `Cargo.toml` file.

```TOML
[dependencies]
settingsfile = "^0.2"
```

Then create a dummy struct and implement the `Clone` + `Format` trait and then create a new `File`, in most cases you can use `#[derive(Clone)]` for `Clone`.

```rust
extern crate settingsfile;
// ...

struct Configuration { }
impl Format for Configuration {
 // implement the functions you need ...
}

fn main() {
  // ...

  // create the `File`
  let settings = Settings::new(Configuration{});

  // reading a value from the settings.
  match settings.get_value("user.name") {
    Ok(user_name) => println!("{}",user_name),
    Err(error) => println!("user.name is not defined."),
  }

  // reading a value and supplying a default in case it doesn't exist or 
  println!("{}",settings.get_value_or("user.name","username is not defined"));

  // and saving data is just as easy.
  settings.set_value("user.name","snsvrno");
  
  // ...
}
```

Keys are accessed by using a 'dot notation', so this means that you can't include '.' in your key names, but you can nest values and work with trees easily.

```rust
// using the previous initalization

if let Ok(user_tree) = settings.get_value("user") {
  // this will give you all the settings nested under user,
  // so you can access or save them as a different file / or 
  // you can easily just remove them all by using `delete_key`.

  settings.get_value("name") // equivalent to "user.name" because we are inside "user"
  ...
}
``` 

One note is that these are _copies_ so manipulating content from `get_value` will not work. You will need to use `set_value` to do any permanent manipulation.

## Types

#### `pub type PartsPackage = HashMap<String,Type>`

A convience type that is used to shorten the required return type for the `Format` trait implemnetations. This does not need to be used by the users of this library.

## Traits

***Settingsfile-rs*** requires an empty struct with the `Format` trait implemented in order to create a `Settingsfile::File`.

### Required Functions

#### `filename()`

```rust
fn filename(&self) -> String
```

Should return the file name of the configuration file; ~/.application_name/***file_name***.extension

The entire name + extension can be used here as well if you don't want to use the [extension()](#extension) function.

####  `folder()`

```rust
fn folder(&self) -> String
```

Should return the folder name of the configuration file with respect to the %user_directory%; ~/.***application_name***/file_name.extension

The '.' isn't automatically added, and you would need to include it if desired. If you want to nest into folders, you can do that too using either '/' or '\\'; it will be converted and parsed per platform.

#### `to_string()`

```rust
fn to_string<T>(&self, object:T) -> Result<String,Error> 
  where T : settingsfile::SupportedType + serde::de::Serialize
```

Returns the seralized form of the passed in `object`. Because this uses ***Serde.rs*** the `object` must have the `serde::de::Serialize` trait, and must also implement the `settingsfile::SupportedType` trait.

Typically this is just a wrapped passthrough to the serde libray you are using. Example using [toml-rs](https://github.com/alexcrichton/toml-rs):

```rust
fn to_string<T:?Sized>(&self,object:&T) -> Result<String,Error>
  where T : settingsfile::SupportedType + serde::ser::Serialize,
{
  match toml::ser::to_string(object) {
    Ok(string) => Ok(string),
    Err(error) => Err(Error::Error(error.to_string()))
  }
}
```

#### `from_str()`

```rust
fn from_str<T>(&self, buffer:&str) -> Result<PartsPackage,Error>
  where T : Clone + settingsfile::Format
```

Returns a deserialized form of a string buffer into a Rust object.

Typically this is just a wrapped passthrough to the serde library you are using. Example using [toml-rs](https://github.com/alexcrichton/toml-rs):

```rust
fn from_str<T>(&self,buffer:&str) -> Result<PartsPackage,Error>
  where T : Format + Clone
{
let result : Result<PartsPackage,toml::de::Error> = toml::from_str(&buffer);
  match result {
    Ok(result) => Ok(result),
    Err(error) => Err(Error::Error(error.to_string())),
  }
}
```


### Optional Functions

#### `extension()`

```rust
fn extension(&self) -> Option<String>
```

Should return the extension of the configuration file; ~/.application_name/file_name.***extension*** 

If not defined then no extension will be used for the file. ***Settingsfile*** does not assume a file's format by its extension so this is just a matter of user preference.
