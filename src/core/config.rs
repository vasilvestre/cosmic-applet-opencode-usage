//! Configuration management for GitHub Copilot quota tracking
//!
//! This module handles persistent configuration storage and retrieval,
//! following XDG Base Directory specification.

use directories::ProjectDirs;
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Configuration validation errors
#[derive(Debug, Error, PartialEq)]
pub enum ConfigError {
    #[error("Organization name cannot be empty")]
    EmptyOrganizationName,
    
    #[error("Organization name contains invalid characters: {0}")]
    InvalidOrganizationCharacters(String),
    
    #[error("Organization name too long (max 39 characters): {0}")]
    OrganizationNameTooLong(usize),
    
    #[error("Personal Access Token cannot be empty")]
    EmptyPersonalAccessToken,
    
    #[error("Personal Access Token has invalid format")]
    InvalidPersonalAccessTokenFormat,
    
    #[error("Refresh interval must be between 300 and 3600 seconds: {0}")]
    InvalidRefreshInterval(u32),
    
    #[error("IO error: {0}")]
    IoError(String),
    
    #[error("TOML serialization error: {0}")]
    TomlSerializationError(String),
    
    #[error("TOML deserialization error: {0}")]
    TomlDeserializationError(String),
    
    #[error("Failed to determine config directory")]
    ConfigDirectoryNotFound,
    
    #[error("Keyring error: {0}")]
    KeyringError(String),
}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::IoError(err.to_string())
    }
}

impl From<toml::ser::Error> for ConfigError {
    fn from(err: toml::ser::Error) -> Self {
        ConfigError::TomlSerializationError(err.to_string())
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(err: toml::de::Error) -> Self {
        ConfigError::TomlDeserializationError(err.to_string())
    }
}

impl From<keyring::Error> for ConfigError {
    fn from(err: keyring::Error) -> Self {
        ConfigError::KeyringError(err.to_string())
    }
}

/// Application configuration structure
///
/// Stores non-sensitive configuration data that persists between sessions.
/// Sensitive data (like PATs) are stored separately in the system keyring.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    /// GitHub organization name to track
    pub organization_name: String,
    /// Refresh interval in seconds (300-3600)
    pub refresh_interval_seconds: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            organization_name: String::new(),
            refresh_interval_seconds: 900, // 15 minutes
        }
    }
}

impl AppConfig {
    /// Validates all configuration fields
    ///
    /// # Returns
    /// - `Ok(())` if all fields are valid
    /// - `Err(ConfigError)` with the first validation error encountered
    pub fn validate(&self) -> Result<(), ConfigError> {
        validate_organization_name(&self.organization_name)?;
        validate_refresh_interval(self.refresh_interval_seconds)?;
        Ok(())
    }
}

/// Validates a GitHub organization name according to GitHub's naming rules
///
/// # Rules
/// - Must not be empty
/// - Must contain only alphanumeric characters and hyphens
/// - Cannot exceed 39 characters (GitHub's maximum)
///
/// # Returns
/// - `Ok(())` if valid
/// - `Err(ConfigError)` with specific validation failure
pub fn validate_organization_name(name: &str) -> Result<(), ConfigError> {
    if name.is_empty() {
        return Err(ConfigError::EmptyOrganizationName);
    }

    if name.len() > 39 {
        return Err(ConfigError::OrganizationNameTooLong(name.len()));
    }

    // Check for invalid characters (must be alphanumeric or hyphen)
    let invalid_chars: Vec<char> = name
        .chars()
        .filter(|c| !c.is_alphanumeric() && *c != '-')
        .collect();

    if !invalid_chars.is_empty() {
        let invalid_str: String = invalid_chars.iter().collect();
        return Err(ConfigError::InvalidOrganizationCharacters(invalid_str));
    }

    Ok(())
}

/// Validates a GitHub Personal Access Token format
///
/// # Rules
/// - Must not be empty
/// - Classic tokens: start with "ghp_" and are 40 chars total
/// - Fine-grained tokens: start with "github_pat_" and vary in length
///
/// # Returns
/// - `Ok(())` if valid
/// - `Err(ConfigError)` with specific validation failure
pub fn validate_personal_access_token(token: &str) -> Result<(), ConfigError> {
    if token.is_empty() {
        return Err(ConfigError::EmptyPersonalAccessToken);
    }

    // Check for valid GitHub PAT prefixes
    let is_classic = token.starts_with("ghp_") && token.len() == 40;
    let is_fine_grained = token.starts_with("github_pat_");

    if !is_classic && !is_fine_grained {
        return Err(ConfigError::InvalidPersonalAccessTokenFormat);
    }

    Ok(())
}

/// Validates refresh interval is within acceptable range
///
/// # Rules
/// - Minimum: 300 seconds (5 minutes)
/// - Maximum: 3600 seconds (1 hour)
///
/// # Returns
/// - `Ok(())` if valid
/// - `Err(ConfigError)` with specific validation failure
pub fn validate_refresh_interval(seconds: u32) -> Result<(), ConfigError> {
    const MIN_INTERVAL: u32 = 300;
    const MAX_INTERVAL: u32 = 3600;

    if seconds < MIN_INTERVAL || seconds > MAX_INTERVAL {
        return Err(ConfigError::InvalidRefreshInterval(seconds));
    }

    Ok(())
}

/// Configuration file manager
///
/// Handles loading and saving configuration to disk following XDG spec
pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    /// Creates a new ConfigManager with XDG-compliant config directory
    ///
    /// Uses the XDG Base Directory specification to determine config location:
    /// - Linux: `~/.config/cosmic-copilot-quota/config.toml`
    /// - macOS: `~/Library/Application Support/com.cosmic.copilot-quota/config.toml`
    /// - Windows: `%APPDATA%\cosmic\copilot-quota\config.toml`
    pub fn new() -> Result<Self, ConfigError> {
        let proj_dirs = ProjectDirs::from("com", "cosmic", "copilot-quota")
            .ok_or(ConfigError::ConfigDirectoryNotFound)?;

        let config_dir = proj_dirs.config_dir();
        let config_path = config_dir.join("config.toml");

        Ok(ConfigManager { config_path })
    }

    /// Creates a ConfigManager with a custom config path (for testing)
    pub fn with_path<P: AsRef<Path>>(path: P) -> Self {
        ConfigManager {
            config_path: path.as_ref().to_path_buf(),
        }
    }

    /// Returns the config file path
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// Loads configuration from disk
    pub fn load(&self) -> Result<AppConfig, ConfigError> {
        let contents = fs::read_to_string(&self.config_path)?;
        let config: AppConfig = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Saves configuration to disk with secure permissions (0600 on Unix)
    pub fn save(&self, config: &AppConfig) -> Result<(), ConfigError> {
        // Ensure parent directory exists
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let toml_string = toml::to_string_pretty(config)?;
        fs::write(&self.config_path, toml_string)?;

        // Set secure file permissions (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o600);
            fs::set_permissions(&self.config_path, perms)?;
        }

        Ok(())
    }
}

/// Service name for keyring storage
const KEYRING_SERVICE: &str = "cosmic-copilot-quota";

/// Username for PAT storage in keyring
const KEYRING_USERNAME: &str = "github-pat";

/// Manager for secure PAT storage using system keyring
pub struct KeyringManager {
    service: String,
    username: String,
}

impl KeyringManager {
    /// Creates a new KeyringManager with default service and username
    pub fn new() -> Self {
        KeyringManager {
            service: KEYRING_SERVICE.to_string(),
            username: KEYRING_USERNAME.to_string(),
        }
    }

    /// Creates a keyring entry for PAT storage
    fn create_entry(&self) -> Result<Entry, ConfigError> {
        Ok(Entry::new(&self.service, &self.username)?)
    }

    /// Stores a GitHub Personal Access Token securely in the system keyring
    ///
    /// # Arguments
    /// * `pat` - The Personal Access Token to store
    ///
    /// # Returns
    /// * `Ok(())` if the PAT was stored successfully
    /// * `Err(ConfigError::KeyringError)` if storage failed
    pub fn store_pat(&self, pat: &str) -> Result<(), ConfigError> {
        let entry = self.create_entry()?;
        entry.set_password(pat)?;
        Ok(())
    }

    /// Retrieves the stored GitHub Personal Access Token from the system keyring
    ///
    /// # Returns
    /// * `Ok(String)` containing the PAT if found
    /// * `Err(ConfigError::KeyringError)` if retrieval failed or PAT not found
    pub fn retrieve_pat(&self) -> Result<String, ConfigError> {
        let entry = self.create_entry()?;
        let password = entry.get_password()?;
        Ok(password)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_creation() {
        let config = AppConfig {
            organization_name: "my-org".to_string(),
            refresh_interval_seconds: 600,
        };

        assert_eq!(config.organization_name, "my-org");
        assert_eq!(config.refresh_interval_seconds, 600);
    }

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        
        // Default organization should be empty (to be configured by user)
        assert_eq!(config.organization_name, "");
        
        // Default refresh interval should be 900 seconds (15 minutes)
        assert_eq!(config.refresh_interval_seconds, 900);
    }

    #[test]
    fn test_app_config_validate_success() {
        let config = AppConfig {
            organization_name: "valid-org".to_string(),
            refresh_interval_seconds: 600,
        };
        
        let result = config.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_app_config_validate_empty_org() {
        let config = AppConfig {
            organization_name: "".to_string(),
            refresh_interval_seconds: 600,
        };
        
        let result = config.validate();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ConfigError::EmptyOrganizationName);
    }

    #[test]
    fn test_app_config_validate_invalid_org_chars() {
        let config = AppConfig {
            organization_name: "org with spaces".to_string(),
            refresh_interval_seconds: 600,
        };
        
        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::InvalidOrganizationCharacters(_)));
    }

    #[test]
    fn test_app_config_validate_invalid_interval() {
        let config = AppConfig {
            organization_name: "valid-org".to_string(),
            refresh_interval_seconds: 100, // Too low
        };
        
        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::InvalidRefreshInterval(_)));
    }

    #[test]
    fn test_app_config_serialization() {
        let config = AppConfig {
            organization_name: "test-org".to_string(),
            refresh_interval_seconds: 900,
        };

        let serialized = toml::to_string(&config).expect("Failed to serialize");
        assert!(serialized.contains("organization_name"));
        assert!(serialized.contains("test-org"));
        assert!(serialized.contains("refresh_interval_seconds"));
        assert!(serialized.contains("900"));
    }

    #[test]
    fn test_app_config_deserialization() {
        let toml_str = r#"
            organization_name = "deserialize-org"
            refresh_interval_seconds = 1200
        "#;

        let config: AppConfig = toml::from_str(toml_str).expect("Failed to deserialize");
        assert_eq!(config.organization_name, "deserialize-org");
        assert_eq!(config.refresh_interval_seconds, 1200);
    }

    #[test]
    fn test_app_config_clone() {
        let config = AppConfig {
            organization_name: "clone-org".to_string(),
            refresh_interval_seconds: 1800,
        };

        let cloned = config.clone();
        assert_eq!(config, cloned);
    }

    // Validation tests
    #[test]
    fn test_validate_organization_name_valid() {
        assert!(validate_organization_name("my-org").is_ok());
        assert!(validate_organization_name("MyOrg123").is_ok());
        assert!(validate_organization_name("org-with-hyphens").is_ok());
        assert!(validate_organization_name("a").is_ok());
    }

    #[test]
    fn test_validate_organization_name_empty() {
        let result = validate_organization_name("");
        assert_eq!(result, Err(ConfigError::EmptyOrganizationName));
    }

    #[test]
    fn test_validate_organization_name_too_long() {
        let long_name = "a".repeat(40);
        let result = validate_organization_name(&long_name);
        assert_eq!(result, Err(ConfigError::OrganizationNameTooLong(40)));
    }

    #[test]
    fn test_validate_organization_name_invalid_characters() {
        let result = validate_organization_name("my org");
        assert!(matches!(
            result,
            Err(ConfigError::InvalidOrganizationCharacters(_))
        ));

        let result = validate_organization_name("my@org");
        assert!(matches!(
            result,
            Err(ConfigError::InvalidOrganizationCharacters(_))
        ));

        let result = validate_organization_name("my.org");
        assert!(matches!(
            result,
            Err(ConfigError::InvalidOrganizationCharacters(_))
        ));
    }

    #[test]
    fn test_validate_organization_name_max_length() {
        let valid_max = "a".repeat(39);
        assert!(validate_organization_name(&valid_max).is_ok());
    }

    // PAT validation tests
    #[test]
    fn test_validate_pat_classic_valid() {
        let classic_token = "ghp_123456789012345678901234567890123456";
        assert_eq!(classic_token.len(), 40);
        assert!(validate_personal_access_token(classic_token).is_ok());
    }

    #[test]
    fn test_validate_pat_fine_grained_valid() {
        let fine_grained = "github_pat_11ABCDEFG0123456789_abcdefghijklmnopqrstuvwxyz";
        assert!(validate_personal_access_token(fine_grained).is_ok());
    }

    #[test]
    fn test_validate_pat_empty() {
        let result = validate_personal_access_token("");
        assert_eq!(result, Err(ConfigError::EmptyPersonalAccessToken));
    }

    #[test]
    fn test_validate_pat_invalid_format() {
        // Wrong prefix
        assert_eq!(
            validate_personal_access_token("invalid_token"),
            Err(ConfigError::InvalidPersonalAccessTokenFormat)
        );

        // Classic token but wrong length
        assert_eq!(
            validate_personal_access_token("ghp_short"),
            Err(ConfigError::InvalidPersonalAccessTokenFormat)
        );

        // No prefix
        assert_eq!(
            validate_personal_access_token("1234567890123456789012345678901234567890"),
            Err(ConfigError::InvalidPersonalAccessTokenFormat)
        );
    }

    // Refresh interval validation tests
    #[test]
    fn test_validate_refresh_interval_valid() {
        assert!(validate_refresh_interval(300).is_ok()); // Minimum
        assert!(validate_refresh_interval(600).is_ok()); // Middle
        assert!(validate_refresh_interval(1800).is_ok()); // Middle
        assert!(validate_refresh_interval(3600).is_ok()); // Maximum
    }

    #[test]
    fn test_validate_refresh_interval_too_low() {
        assert_eq!(
            validate_refresh_interval(299),
            Err(ConfigError::InvalidRefreshInterval(299))
        );
        assert_eq!(
            validate_refresh_interval(0),
            Err(ConfigError::InvalidRefreshInterval(0))
        );
    }

    #[test]
    fn test_validate_refresh_interval_too_high() {
        assert_eq!(
            validate_refresh_interval(3601),
            Err(ConfigError::InvalidRefreshInterval(3601))
        );
        assert_eq!(
            validate_refresh_interval(10000),
            Err(ConfigError::InvalidRefreshInterval(10000))
        );
    }

    // ConfigManager tests
    #[test]
    fn test_config_manager_new() {
        let manager = ConfigManager::new();
        assert!(manager.is_ok());
        
        let manager = manager.unwrap();
        let path = manager.config_path();
        assert!(path.to_string_lossy().contains("copilot-quota"));
        assert!(path.to_string_lossy().ends_with("config.toml"));
    }

    #[test]
    fn test_config_manager_with_custom_path() {
        let custom_path = PathBuf::from("/tmp/test-config.toml");
        let manager = ConfigManager::with_path(&custom_path);
        assert_eq!(manager.config_path(), custom_path);
    }

    #[test]
    fn test_config_manager_save_and_load() {
        use std::env;
        
        // Create temp file
        let temp_dir = env::temp_dir();
        let test_config_path = temp_dir.join("cosmic-test-config.toml");
        
        // Clean up any existing file
        let _ = fs::remove_file(&test_config_path);

        let manager = ConfigManager::with_path(&test_config_path);

        let config = AppConfig {
            organization_name: "test-org".to_string(),
            refresh_interval_seconds: 600,
        };

        // Save config
        let save_result = manager.save(&config);
        assert!(save_result.is_ok(), "Failed to save: {:?}", save_result);

        // Load config
        let loaded = manager.load();
        assert!(loaded.is_ok(), "Failed to load: {:?}", loaded);
        
        let loaded_config = loaded.unwrap();
        assert_eq!(loaded_config.organization_name, "test-org");
        assert_eq!(loaded_config.refresh_interval_seconds, 600);

        // Clean up
        let _ = fs::remove_file(&test_config_path);
    }

    #[test]
    fn test_config_manager_load_missing_file() {
        let manager = ConfigManager::with_path("/tmp/nonexistent-config-file.toml");
        let result = manager.load();
        assert!(result.is_err());
        assert!(matches!(result, Err(ConfigError::IoError(_))));
    }

    #[test]
    #[cfg(unix)]
    fn test_config_file_permissions() {
        use std::env;
        use std::os::unix::fs::PermissionsExt;
        
        // Create temp file
        let temp_dir = env::temp_dir();
        let test_config_path = temp_dir.join("cosmic-test-perms-config.toml");
        
        // Clean up any existing file
        let _ = fs::remove_file(&test_config_path);

        let manager = ConfigManager::with_path(&test_config_path);

        let config = AppConfig {
            organization_name: "test-org".to_string(),
            refresh_interval_seconds: 600,
        };

        // Save config
        manager.save(&config).expect("Failed to save config");

        // Check file permissions
        let metadata = fs::metadata(&test_config_path).expect("Failed to get metadata");
        let permissions = metadata.permissions();
        let mode = permissions.mode();
        
        // Extract permission bits (last 9 bits)
        let file_perms = mode & 0o777;
        
        assert_eq!(
            file_perms, 0o600,
            "Config file should have 0600 permissions, got {:o}",
            file_perms
        );

        // Clean up
        let _ = fs::remove_file(&test_config_path);
    }

    // Integration Tests
    #[test]
    fn test_full_config_workflow() {
        use std::env;
        
        // Create temp file for integration test
        let temp_dir = env::temp_dir();
        let test_config_path = temp_dir.join("cosmic-integration-test-config.toml");
        
        // Clean up any existing file
        let _ = fs::remove_file(&test_config_path);

        // Step 1: Create default config
        let default_config = AppConfig::default();
        assert_eq!(default_config.organization_name, "");
        assert_eq!(default_config.refresh_interval_seconds, 900);

        // Step 2: Validate default config (should fail - empty org)
        let validation_result = default_config.validate();
        assert!(validation_result.is_err());
        assert!(matches!(
            validation_result,
            Err(ConfigError::EmptyOrganizationName)
        ));

        // Step 3: Create valid config
        let valid_config = AppConfig {
            organization_name: "github".to_string(),
            refresh_interval_seconds: 600,
        };

        // Step 4: Validate valid config (should succeed)
        assert!(valid_config.validate().is_ok());

        // Step 5: Save config to disk
        let manager = ConfigManager::with_path(&test_config_path);
        let save_result = manager.save(&valid_config);
        assert!(save_result.is_ok(), "Failed to save config: {:?}", save_result);

        // Step 6: Verify file exists
        assert!(test_config_path.exists(), "Config file was not created");

        // Step 7: Load config from disk
        let load_result = manager.load();
        assert!(load_result.is_ok(), "Failed to load config: {:?}", load_result);
        
        let loaded_config = load_result.unwrap();
        assert_eq!(loaded_config.organization_name, "github");
        assert_eq!(loaded_config.refresh_interval_seconds, 600);

        // Step 8: Validate loaded config (should succeed)
        assert!(loaded_config.validate().is_ok());

        // Step 9: Test error handling for corrupted files
        // Write invalid TOML to file
        fs::write(&test_config_path, "this is not valid TOML {[}")
            .expect("Failed to write corrupted file");

        let corrupted_load_result = manager.load();
        assert!(corrupted_load_result.is_err());
        assert!(matches!(
            corrupted_load_result,
            Err(ConfigError::TomlDeserializationError(_))
        ));

        // Step 10: Test error handling for missing files
        let _ = fs::remove_file(&test_config_path);
        let missing_load_result = manager.load();
        assert!(missing_load_result.is_err());
        assert!(matches!(
            missing_load_result,
            Err(ConfigError::IoError(_))
        ));

        // Clean up
        let _ = fs::remove_file(&test_config_path);
    }

    // KeyringManager tests
    #[test]
    fn test_keyring_manager_new() {
        let manager = KeyringManager::new();
        assert_eq!(manager.service, "cosmic-copilot-quota");
        assert_eq!(manager.username, "github-pat");
    }

    #[test]
    fn test_keyring_store_and_retrieve_pat() {
        // Use a unique service name to avoid test interference
        let manager = KeyringManager {
            service: "cosmic-copilot-quota-test-store".to_string(),
            username: KEYRING_USERNAME.to_string(),
        };
        let test_pat = "ghp_1234567890123456789012345678901234567890";

        // Clean up first (in case previous test run didn't clean up)
        let entry = Entry::new(&manager.service, &manager.username).unwrap();
        let _ = entry.delete_credential();

        // Store PAT
        let store_result = manager.store_pat(test_pat);
        assert!(store_result.is_ok(), "Failed to store PAT: {:?}", store_result);

        // Retrieve PAT
        let retrieve_result = manager.retrieve_pat();
        assert!(retrieve_result.is_ok(), "Failed to retrieve PAT: {:?}", retrieve_result);
        
        let retrieved_pat = retrieve_result.unwrap();
        assert_eq!(retrieved_pat, test_pat);

        // Clean up - delete the stored PAT
        let _ = entry.delete_credential();
    }

    #[test]
    fn test_keyring_retrieve_pat_not_found() {
        // Use a unique service name to avoid test interference
        let manager = KeyringManager {
            service: "cosmic-copilot-quota-test-notfound".to_string(),
            username: KEYRING_USERNAME.to_string(),
        };
        
        // Ensure no PAT is stored (clean up first)
        let entry = Entry::new(&manager.service, &manager.username).unwrap();
        let _ = entry.delete_credential(); // Ignore result - might not exist

        // Try to retrieve non-existent PAT
        let result = manager.retrieve_pat();
        assert!(result.is_err(), "Should fail when PAT not found");
        assert!(matches!(result, Err(ConfigError::KeyringError(_))));
    }

    #[test]
    fn test_keyring_error_conversion() {
        // Test that keyring::Error converts to ConfigError properly
        // This tests the From<keyring::Error> implementation
        
        // Create a keyring error by trying to access an invalid entry
        let entry = Entry::new("", "").expect("Failed to create invalid entry");
        let result = entry.get_password();
        
        assert!(result.is_err(), "Should fail with empty service name");
        
        // Convert the keyring error to ConfigError using From trait
        let keyring_err = result.unwrap_err();
        let config_err: ConfigError = keyring_err.into();
        
        // Verify it's the right error type
        assert!(matches!(config_err, ConfigError::KeyringError(_)));
        
        // Verify error message is meaningful
        if let ConfigError::KeyringError(msg) = config_err {
            assert!(!msg.is_empty(), "Error message should not be empty");
        }
    }

}
