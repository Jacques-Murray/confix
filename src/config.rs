use crate::error::ConfixError;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// A simple map to hold key-value config pairs.
pub type ConfigMap = HashMap<String, String>;

/// Loads configuration from a given file path.
///
/// Auto-detects the format (.env, .json, .toml) based on the extension.
pub fn load_config_file(path: &Path) -> Result<ConfigMap, ConfixError> {
    if !path.exists() {
        return Err(ConfixError::FileNotFound(path.to_path_buf()));
    }

    match path.extension().and_then(|s| s.to_str()) {
        Some("env") => load_dotenv(path),
        Some("json") => load_json(path),
        Some("toml") => load_toml(path),
        // Handle .env file with no extension
        _ if path.file_name().and_then(|s| s.to_str()) == Some(".env") => load_dotenv(path),
        _ => Err(ConfixError::UnsupportedFormat(path.to_path_buf())),
    }
}

/// Loads a .env file.
fn load_dotenv(path: &Path) -> Result<ConfigMap, ConfixError> {
    // Use dotenvy::from_path_iter to read without modifying the environment
    let iter = dotenvy::from_path_iter(path)?;
    let mut config = ConfigMap::new();
    for item in iter {
        let (key, value) = item?;
        config.insert(key, value);
    }
    Ok(config)
}

/// Loads a .json file.
/// Expects a flat JSON object with string values.
fn load_json(path: &Path) -> Result<ConfigMap, ConfixError> {
    let content = fs::read_to_string(path)?;
    let config: ConfigMap = serde_json::from_str(&content)?;
    Ok(config)
}

/// Loads a .toml file.
/// Expects a flat TOML table with string values.
fn load_toml(path: &Path) -> Result<ConfigMap, ConfixError> {
    let content = fs::read_to_string(path)?;
    let config: ConfigMap = toml::from_str(&content)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_dotenv_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATABASE_URL=postgres://...\nAPI_KEY=12345").unwrap();

        let config = load_config_file(file.path()).unwrap();
        assert_eq!(config.get("DATABASE_URL").unwrap(), "postgres://...");
        assert_eq!(config.get("API_KEY").unwrap(), "12345");
    }

    #[test]
    fn test_load_json_file() {
        let file = NamedTempFile::new_in(".").unwrap();
        // Rename to have .json extension
        let json_file = file.into_temp_path().with_extension("json");
        fs::write(
            &json_file,
            r#"{"DATABASE_URL": "json://...", "API_KEY": "abc"}"#,
        )
        .unwrap();

        let config = load_config_file(&json_file).unwrap();
        assert_eq!(config.get("DATABASE_URL").unwrap(), "json://...");
        assert_eq!(config.get("API_KEY").unwrap(), "abc");

        fs::remove_file(&json_file).unwrap();
    }

    #[test]
    fn test_load_toml_file() {
        let file = NamedTempFile::new_in(".").unwrap();
        // Rename to have .toml extension
        let toml_file = file.into_temp_path().with_extension("toml");
        fs::write(
            &toml_file,
            "DATABASE_URL = \"toml://...\"\nAPI_KEY = \"xyz\"",
        )
        .unwrap();

        let config = load_config_file(&toml_file).unwrap();
        assert_eq!(config.get("DATABASE_URL").unwrap(), "toml://...");
        assert_eq!(config.get("API_KEY").unwrap(), "xyz");

        fs::remove_file(&toml_file).unwrap();
    }

    #[test]
    fn test_load_dotenv_no_extension() {
        let path = Path::new(".env");

        // A simple cleanup guard to ensure the file is deleted
        struct EnvGuard<'a>(&'a Path);
        impl<'a> Drop for EnvGuard<'a> {
            fn drop(&mut self) {
                let _ = fs::remove_file(self.0);
            }
        }
        let _guard = EnvGuard(path);

        // Write the file
        fs::write(path, "SECRET=from-dotenv").unwrap();

        // Run the test
        let config = load_config_file(path).unwrap();
        assert_eq!(config.get("SECRET").unwrap(), "from-dotenv");

        // _guard will automatically delete the file
    }

    #[test]
    fn test_file_not_found() {
        let path = Path::new("nonexistent.file");
        let result = load_config_file(path);
        assert!(matches!(result, Err(ConfixError::FileNotFound(_))));
    }

    #[test]
    fn test_unsupported_format() {
        let file = NamedTempFile::new_in(".").unwrap();
        let path = file.into_temp_path().with_extension("txt");
        fs::write(&path, "hello=world").unwrap();

        let result = load_config_file(&path);
        assert!(matches!(result, Err(ConfixError::UnsupportedFormat(_))));

        fs::remove_file(&path).unwrap();
    }
}
