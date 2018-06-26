use structs::filetype::Filetype;

pub struct SettingsOptions {
  pub extension : Option<String>,
  pub filetype : Filetype,
  pub local_enabled : bool,
  pub use_default_extension : bool,
}

impl SettingsOptions {
  pub fn defaults() -> SettingsOptions {
    SettingsOptions {
      extension: None,
      use_default_extension: true,
      filetype : Filetype::Toml,
      local_enabled : false
    }
  }
}