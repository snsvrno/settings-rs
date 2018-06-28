# Settingsfile-rs
A library for easily accessing settings and configuration files using [Serde](https://serde.rs/).

## Brief 
***Settingsfile-rs*** attempts to abstract away defining, parsing, and saving configuration files for cli and other programs.

## Usage
Add ***Settingsfile-rs*** to your `Cargo.toml` file.

```TOML
[dependencies]
settingsfile = "^0.2"
```

Then create a dummy struct and implement the `Clone` + `Format` trait and then create a new `File`, you can use `#[derive(Clone)]` for `Clone`.

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

Should return the folder name of the configuration file with respect to the %user_directory%; ~/***.application_name***/file_name.extension

#### `to_string()`

```rust
fn to_string<T>(&self, object:T) -> Result<String,Error> 
  where T : settingsfile::SupportedType + serde::de::Serialize
```

Returns the seralized form of the passed in `object`. Because this uses ***Serde.rs*** the `object` must have the `serde::de::Serialzie` trait, and must also implement the `settingsfile::SupportedType` trait.

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

Typically this is just aw rapped passthrough to the serde library you are using. Example using [toml-rs](https://github.com/alexcrichton/toml-rs):

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

If not defined then no extension will be used for the file.
