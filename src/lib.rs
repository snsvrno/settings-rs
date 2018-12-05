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