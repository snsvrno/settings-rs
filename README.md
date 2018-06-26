# Settingsfile-rs
A library for easily accessing settings and configuration files.

## Brief 
***Settingsfile-rs*** attempts to abstract away defining, parsing, and saving configuration files for cli and other programs. Currently ***TOML*** is the only supported format but ***JSON*** and ***INI*** is planned.

## Usage
Add ***Settingsfile-rs*** to your `Cargo.toml` file.

```TOML
[dependencies]
...
settingsfile = "0.1.0"
...
```

Then create a new instance of the ***Settings*** struct defining where and how you want to access the settings / configuration.

```rust
extern crate settingsfile; use settingsfile::Settings;

...

  let settings = Settings::new(".config","settings.toml");

  match settings.get_value("user.name") {
    Ok(user_name) => println!("{}",user_name),
    Err(error) => println!("user.name is not defined."),
  }

  // or 

  println!("{}",settings.get_value_or("user.name","username is not defined"));

  // and saving data is just as easy.

  settings.set_value("user.name","snsvrno");

```

## Some Facts about `Settingsfile-rs`

- ***Settingsfile*** reads and write file immediately, it is primarily designed for CLI apps, but can be used for all types. Any `set_value()` operation is handled immediately by writing all the changes to the file.
- ***Settingsfile*** can read files from the defined `folder` (`%user_folder%/%folder%`) and from the local working directory of the CLI app (if option `local_enabled` is true).
- 'Dot notation' is used to read and write settings files, which allows for easy access without worrying about what else is inside the file.

## Dot Notation Rules

Quick primer in the style of `dot notation` used by ***Settingsfile-rs***. Using the following examples

```json
"user" : {
  "name" : "snsvrno"
}
```

```toml
[user]
name = "snsvrno"
```

1. A `value` is access by all the parents connected by '.', called a `key_path`. The value of `snsvrno` is represented with a `key_path` of `user.name`.