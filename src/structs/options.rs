use structs::filetype::Filetype;

pub struct SettingsOptions {
  pub extension : Option<String>,
  pub filetype : Filetype,
  pub local_enabled : bool
}

impl SettingsOptions {
  pub fn defaults() -> SettingsOptions {
    SettingsOptions {
      extension: None,
      filetype : Filetype::Toml,
      local_enabled : true
    }
  }
}