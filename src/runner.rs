use crate::config::{ConfigMap, load_config_file};
use crate::error::ConfixError;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Loads all provided config files and merges them.
/// Later files override earlier ones.
fn merge_configs(paths: &[PathBuf]) -> Result<ConfigMap, ConfixError> {
    let mut merged_config = ConfigMap::new();
    for path in paths {
        let config = load_config_file(path)?;
        merged_config.extend(config);
    }
    Ok(merged_config)
}

/// Executes a command with the given configuration as environment variables.
pub fn run_command(config_paths: &[PathBuf], cmd_args: &[String]) -> Result<i32, ConfixError> {
    // 1. Load and merge configurations
    let config = merge_configs(config_paths)?;

    // 2. Separate command and its arguments
    let (command, args) = cmd_args
        .split_first()
        .ok_or_else(|| ConfixError::CommandFailed("No command provided.".to_string()))?;

    // 3. Build the command
    let mut child = Command::new(command)
        .args(args)
        .envs(&config) // Inject the merged config as env vars
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| {
            ConfixError::CommandFailed(format!("Failed to spawn command '{}': {}", command, e))
        })?;

    // 4. Wait for the command to finish
    let status = child.wait().map_err(|e| {
        ConfixError::CommandFailed(format!("Command '{}' failed to run: {}", command, e))
    })?;

    // Return the exit code of the child process
    Ok(status.code().unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{Builder, NamedTempFile};

    // Helper to create a temp config file
    fn temp_file(content: &str, ext: &str) -> NamedTempFile {
        let mut file = Builder::new()
            .suffix(&format!(".{}", ext))
            .tempfile_in(".") // Create in current dir
            .unwrap();

        file.write_all(content.as_bytes()).unwrap();
        file // Return the file
    }

    #[test]
    fn test_merge_configs() {
        let file1 = temp_file("KEY1=value1\nOVERRIDE=from_env", "env");
        let file2 = temp_file(r#"{"OVERRIDE": "from_json", "KEY2": "value2"}"#, "json");

        let paths = vec![file1.path().to_path_buf(), file2.path().to_path_buf()];
        let config = merge_configs(&paths).unwrap();

        assert_eq!(config.get("KEY1").unwrap(), "value1");
        assert_eq!(config.get("KEY2").unwrap(), "value2");
        assert_eq!(config.get("OVERRIDE").unwrap(), "from_json");
    }

    #[test]
    fn test_run_command_success() {
        let file = temp_file("TEST_VAR=confix_test_ran", "env");
        let paths = vec![file.path().to_path_buf()];

        // Use `sh -c 'env | grep ...'` or `cmd /C "set | findstr ..."`
        let (cmd, arg1, arg2) = if cfg!(target_os = "windows") {
            ("cmd", "/C", "set | findstr TEST_VAR")
        } else {
            ("sh", "-c", "env | grep TEST_VAR")
        };

        let cmd_args = vec![cmd.to_string(), arg1.to_string(), arg2.to_string()];

        let exit_code = run_command(&paths, &cmd_args).unwrap();
        assert_eq!(exit_code, 0);
    }

    #[test]
    fn test_run_command_no_cmd() {
        let cmd_args = Vec::<String>::new();
        let result = run_command(&[], &cmd_args);
        assert!(matches!(result, Err(ConfixError::CommandFailed(_))));
    }
}
