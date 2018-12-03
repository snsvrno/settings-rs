#[macro_use] extern crate serde_derive;
#[macro_use] extern crate failure;
extern crate serde;

//use std::fs::{File,create_dir_all};
//use std::path::PathBuf;
//use std::collections::HashMap;
//use std::env;

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