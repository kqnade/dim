use std::path::Path;

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Parse(toml::de::Error),
    Serialize(toml::ser::Error),
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Config {
    #[serde(default = "default_tab_width")]
    pub tab_width: usize,
    #[serde(default = "default_skk_enabled")]
    pub skk_enabled: bool,
    pub skk_system_dictionary_path: Option<String>,
    pub skk_user_dictionary_path: Option<String>,
    #[serde(default = "default_show_line_numbers")]
    pub show_line_numbers: bool,
    #[serde(default = "default_show_relative_line_numbers")]
    pub show_relative_line_numbers: bool,
}

fn default_tab_width() -> usize {
    4
}

fn default_skk_enabled() -> bool {
    false
}

fn default_show_line_numbers() -> bool {
    true
}

fn default_show_relative_line_numbers() -> bool {
    false
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tab_width: 4,
            skk_enabled: false,
            skk_system_dictionary_path: None,
            skk_user_dictionary_path: None,
            show_line_numbers: true,
            show_relative_line_numbers: false,
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let contents = std::fs::read_to_string(path).map_err(ConfigError::Io)?;
        let config: Config = toml::from_str(&contents).map_err(ConfigError::Parse)?;
        Ok(config)
    }

    pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
        let contents = toml::to_string_pretty(self).map_err(ConfigError::Serialize)?;
        std::fs::write(path, contents).map_err(ConfigError::Io)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;
    use std::fs;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.tab_width, 4);
        assert!(!config.skk_enabled);
        assert_eq!(config.skk_system_dictionary_path, None);
        assert_eq!(config.skk_user_dictionary_path, None);
        assert!(config.show_line_numbers); // opt-out: default on
        assert!(!config.show_relative_line_numbers);
    }

    #[test]
    fn test_config_save_and_load() {
        let path = temp_dir().join("dim_test_config.toml");
        let config = Config {
            tab_width: 8,
            skk_enabled: false,
            skk_system_dictionary_path: Some("/usr/share/skk/SKK-JISYO.L".to_string()),
            skk_user_dictionary_path: Some("~/.skk-jisyo".to_string()),
            show_line_numbers: true,
            show_relative_line_numbers: true,
        };
        config.save(&path).unwrap();
        let loaded = Config::load(&path).unwrap();
        assert_eq!(config, loaded);
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_config_load_not_found() {
        let path = temp_dir().join("dim_test_config_missing.toml");
        let result = Config::load(&path);
        assert!(matches!(result, Err(ConfigError::Io(_))));
    }

    #[test]
    fn test_config_load_partial_toml() {
        let path = temp_dir().join("dim_test_config_partial.toml");
        fs::write(&path, "tab_width = 2\n").unwrap();
        let loaded = Config::load(&path).unwrap();
        assert_eq!(loaded.tab_width, 2);
        assert!(!loaded.skk_enabled);
        assert_eq!(loaded.skk_system_dictionary_path, None);
        assert_eq!(loaded.skk_user_dictionary_path, None);
        assert!(loaded.show_line_numbers); // opt-out: default on
        assert!(!loaded.show_relative_line_numbers);
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_config_load_line_numbers_from_toml() {
        let path = temp_dir().join("dim_test_config_lines.toml");
        fs::write(
            &path,
            "show_line_numbers = true\nshow_relative_line_numbers = true\n",
        )
        .unwrap();
        let loaded = Config::load(&path).unwrap();
        assert!(loaded.show_line_numbers);
        assert!(loaded.show_relative_line_numbers);
        fs::remove_file(&path).unwrap();
    }
}
