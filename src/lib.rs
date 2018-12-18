//! An easy to use local settings library.
//! 
//! The `settingsfile` crate provides a simple and easy to use API that abstracts away
//! most OS and serializing / deserializing functions so you can focus on saving and reading
//! settings and configurations.
//! 
//! The user of this crate only needs to define a struct that implements [Format](trait.format.html), 
//! which defines the location and format of the configuration file. Then the user can call
//! set and get functions to interact with that file and the data in that file.
//! 
//! `settingsfile` works primarily in memory and does not automatically save and load data from 
//! the disk, so ::load() and ::save() will need to manually be called.

#[macro_use] extern crate serde_derive;
#[macro_use] extern crate failure;
#[macro_use] extern crate log;
extern crate serde;
extern crate dirs;

// public traits
mod traits;
pub use traits::supportedtype::SupportedType;
pub use traits::format::Format;
pub use traits::format::SettingsRaw;
// public structs
mod structs;
pub use structs::settings::Settings;
pub use structs::shadowsettings::ShadowSettings;
pub use structs::types::Type;
pub use structs::empty::EmptyConfig;