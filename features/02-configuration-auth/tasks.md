# Implementation Tasks: Configuration & Authentication

## Overview
This document breaks down Feature 02 into implementable TDD tasks following Kent Beck's Red-Green-Refactor cycle. Each task follows the pattern: write failing test → implement minimum code → refactor if needed.

## Testing Strategy
- Write unit tests for all validation logic
- Write integration tests for file I/O operations
- Use temporary directories for config file tests
- Mock keyring operations where system keyring unavailable
- Run all tests after each task completion

---

## Phase 1: Core Configuration Module

### Task 1.1: Add Dependencies
**Type**: Setup  
**Estimated Time**: 5 minutes

#### Steps:
1. Add to `Cargo.toml`:
   ```toml
   serde = { version = "1.0", features = ["derive"] }
   toml = "0.8"
   directories = "5.0"
   thiserror = "1.0"
   ```
2. Run `cargo build` to verify dependencies resolve
3. Commit: "Add dependencies for configuration management"

**Acceptance Criteria**:
- [ ] All dependencies added
- [ ] `cargo build` succeeds
- [ ] No dependency conflicts

---

### Task 1.2: Create AppConfig Struct (RED)
**Type**: Test-Driven Development  
**Estimated Time**: 10 minutes

#### RED: Write Failing Test
```rust
// src/core/config.rs

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_appconfig_can_be_created() {
        let config = AppConfig {
            organization_name: "test-org".to_string(),
            refresh_interval_minutes: 15,
        };
        assert_eq!(config.organization_name, "test-org");
        assert_eq!(config.refresh_interval_minutes, 15);
    }
    
    #[test]
    fn test_appconfig_serializes_to_toml() {
        let config = AppConfig {
            organization_name: "test-org".to_string(),
            refresh_interval_minutes: 15,
        };
        let toml = toml::to_string(&config).unwrap();
        assert!(toml.contains("organization_name"));
        assert!(toml.contains("test-org"));
    }
    
    #[test]
    fn test_appconfig_deserializes_from_toml() {
        let toml_str = r#"
            organization_name = "test-org"
            refresh_interval_minutes = 15
        "#;
        let config: AppConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.organization_name, "test-org");
        assert_eq!(config.refresh_interval_minutes, 15);
    }
    
    #[test]
    fn test_appconfig_equality() {
        let config1 = AppConfig {
            organization_name: "test-org".to_string(),
            refresh_interval_minutes: 15,
        };
        let config2 = AppConfig {
            organization_name: "test-org".to_string(),
            refresh_interval_minutes: 15,
        };
        assert_eq!(config1, config2);
    }
}
```

Run: `cargo test --lib` → Tests should fail (AppConfig doesn't exist)

#### GREEN: Implement Minimum Code
```rust
// src/core/config.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    pub organization_name: String,
    pub refresh_interval_minutes: u32,
}
```

Update `src/core/mod.rs`:
```rust
pub mod config;
pub mod localization;
pub mod models;
```

Run: `cargo test --lib` → All tests should pass

#### REFACTOR: Clean Up
- No refactoring needed at this stage
- Code is simple and clear

**Acceptance Criteria**:
- [x] Tests written and initially failing
- [x] AppConfig struct implemented
- [x] All tests passing
- [x] Serialization/deserialization works

**Commit**: "Add AppConfig struct with serde support"

---

### Task 1.3: Implement ConfigError (RED)
**Type**: Test-Driven Development  
**Estimated Time**: 15 minutes

#### RED: Write Failing Test
```rust
// src/core/config.rs

#[cfg(test)]
mod tests {
    // ... previous tests ...
    
    #[test]
    fn test_config_error_display() {
        let err = ConfigError::EmptyOrgName;
        assert_eq!(err.to_string(), "Organization name cannot be empty");
        
        let err = ConfigError::InvalidRefreshInterval;
        assert!(err.to_string().contains("1 and 60"));
    }
    
    #[test]
    fn test_config_error_is_error_trait() {
        let err = ConfigError::FileNotFound;
        let _: &dyn std::error::Error = &err;  // Should compile
    }
}
```

Run: `cargo test --lib` → Tests should fail

#### GREEN: Implement Minimum Code
```rust
// src/core/config.rs

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum ConfigError {
    #[error("Configuration file not found")]
    FileNotFound,
    
    #[error("Failed to read configuration file: {0}")]
    FileReadError(String),
    
    #[error("Failed to write configuration file: {0}")]
    FileWriteError(String),
    
    #[error("Permission denied accessing configuration file")]
    PermissionError,
    
    #[error("Invalid configuration format: {0}")]
    InvalidFormat(String),
    
    #[error("Organization name cannot be empty")]
    EmptyOrgName,
    
    #[error("Organization name is too long (max 39 characters)")]
    OrgNameTooLong,
    
    #[error("Invalid organization name format")]
    InvalidOrgNameFormat,
    
    #[error("Personal Access Token cannot be empty")]
    EmptyPat,
    
    #[error("Invalid Personal Access Token format")]
    InvalidPatFormat,
    
    #[error("Refresh interval must be between 1 and 60 minutes")]
    InvalidRefreshInterval,
    
    #[error("System keyring not available")]
    KeyringNotAvailable,
    
    #[error("Failed to access keyring: {0}")]
    KeyringAccessError(String),
    
    #[error("Personal Access Token not found in keyring")]
    PatNotFound,
}
```

Run: `cargo test --lib` → All tests should pass

#### REFACTOR: Clean Up
- Error messages are clear and user-friendly
- thiserror handles Error trait implementation

**Acceptance Criteria**:
- [x] ConfigError enum defined
- [x] All error variants have descriptive messages
- [x] Implements Error trait via thiserror
- [x] Tests passing

**Commit**: "Add ConfigError enum with descriptive messages"

---

### Task 1.4: Implement Organization Name Validation (RED)
**Type**: Test-Driven Development  
**Estimated Time**: 20 minutes

#### RED: Write Failing Tests
```rust
#[cfg(test)]
mod tests {
    // ... previous tests ...
    
    #[test]
    fn test_validate_org_name_valid() {
        assert!(validate_org_name("my-org").is_ok());
        assert!(validate_org_name("MyOrg123").is_ok());
        assert!(validate_org_name("a").is_ok());
        assert!(validate_org_name("a-b-c").is_ok());
    }
    
    #[test]
    fn test_validate_org_name_empty() {
        let result = validate_org_name("");
        assert_eq!(result, Err(ConfigError::EmptyOrgName));
    }
    
    #[test]
    fn test_validate_org_name_too_long() {
        let long_name = "a".repeat(40);
        let result = validate_org_name(&long_name);
        assert_eq!(result, Err(ConfigError::OrgNameTooLong));
    }
    
    #[test]
    fn test_validate_org_name_invalid_chars() {
        assert_eq!(
            validate_org_name("my org"),
            Err(ConfigError::InvalidOrgNameFormat)
        );
        assert_eq!(
            validate_org_name("my_org"),
            Err(ConfigError::InvalidOrgNameFormat)
        );
        assert_eq!(
            validate_org_name("my@org"),
            Err(ConfigError::InvalidOrgNameFormat)
        );
    }
    
    #[test]
    fn test_validate_org_name_invalid_hyphens() {
        assert_eq!(
            validate_org_name("-myorg"),
            Err(ConfigError::InvalidOrgNameFormat)
        );
        assert_eq!(
            validate_org_name("myorg-"),
            Err(ConfigError::InvalidOrgNameFormat)
        );
        assert_eq!(
            validate_org_name("my--org"),
            Err(ConfigError::InvalidOrgNameFormat)
        );
    }
}
```

Run: `cargo test --lib` → Tests should fail

#### GREEN: Implement Minimum Code
```rust
// src/core/config.rs

pub fn validate_org_name(name: &str) -> Result<(), ConfigError> {
    if name.is_empty() {
        return Err(ConfigError::EmptyOrgName);
    }
    
    if name.len() > 39 {
        return Err(ConfigError::OrgNameTooLong);
    }
    
    // Check for valid characters: alphanumeric and hyphens only
    let valid_chars = name.chars().all(|c| c.is_alphanumeric() || c == '-');
    if !valid_chars {
        return Err(ConfigError::InvalidOrgNameFormat);
    }
    
    // Check for hyphen rules: no leading/trailing, no consecutive
    if name.starts_with('-') || name.ends_with('-') {
        return Err(ConfigError::InvalidOrgNameFormat);
    }
    
    if name.contains("--") {
        return Err(ConfigError::InvalidOrgNameFormat);
    }
    
    Ok(())
}
```

Run: `cargo test --lib` → All tests should pass

#### REFACTOR: Clean Up
- Consider extracting character validation to separate function if needed
- Current implementation is clear and readable

**Acceptance Criteria**:
- [x] Validates empty names
- [x] Validates length (max 39 chars)
- [x] Validates character set (alphanumeric + hyphens)
- [x] Validates hyphen placement rules
- [x] All tests passing

**Commit**: "Add organization name validation"

---

### Task 1.5: Implement PAT Validation (RED)
**Type**: Test-Driven Development  
**Estimated Time**: 20 minutes

#### RED: Write Failing Tests
```rust
#[cfg(test)]
mod tests {
    // ... previous tests ...
    
    #[test]
    fn test_validate_pat_classic_format() {
        let classic = "ghp_1234567890123456789012345678901234567890";
        assert!(validate_pat(classic).is_ok());
    }
    
    #[test]
    fn test_validate_pat_fine_grained_format() {
        let fine_grained = "github_pat_11ABCDEFG_abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        assert!(validate_pat(fine_grained).is_ok());
    }
    
    #[test]
    fn test_validate_pat_empty() {
        let result = validate_pat("");
        assert_eq!(result, Err(ConfigError::EmptyPat));
    }
    
    #[test]
    fn test_validate_pat_invalid_prefix() {
        let result = validate_pat("invalid_1234567890123456789012345678901234567890");
        assert_eq!(result, Err(ConfigError::InvalidPatFormat));
    }
    
    #[test]
    fn test_validate_pat_classic_wrong_length() {
        let short = "ghp_123";
        assert_eq!(validate_pat(short), Err(ConfigError::InvalidPatFormat));
        
        let long = "ghp_12345678901234567890123456789012345678901";
        assert_eq!(validate_pat(long), Err(ConfigError::InvalidPatFormat));
    }
    
    #[test]
    fn test_validate_pat_fine_grained_too_short() {
        let short = "github_pat_123";
        assert_eq!(validate_pat(short), Err(ConfigError::InvalidPatFormat));
    }
}
```

Run: `cargo test --lib` → Tests should fail

#### GREEN: Implement Minimum Code
```rust
// src/core/config.rs

pub fn validate_pat(token: &str) -> Result<(), ConfigError> {
    if token.is_empty() {
        return Err(ConfigError::EmptyPat);
    }
    
    // GitHub Classic PAT: ghp_ + 40 characters = 44 total
    let is_classic = token.starts_with("ghp_") && token.len() == 44;
    
    // GitHub Fine-grained PAT: github_pat_ + 82+ characters = 93+ total
    let is_fine_grained = token.starts_with("github_pat_") && token.len() >= 93;
    
    if !is_classic && !is_fine_grained {
        return Err(ConfigError::InvalidPatFormat);
    }
    
    Ok(())
}
```

Run: `cargo test --lib` → All tests should pass

#### REFACTOR: Clean Up
- Code is clear and follows GitHub's token format rules
- No refactoring needed

**Acceptance Criteria**:
- [x] Validates classic PAT format (ghp_ + 40 chars)
- [x] Validates fine-grained PAT format (github_pat_ + 82+ chars)
- [x] Rejects empty tokens
- [x] Rejects invalid formats
- [x] All tests passing

**Commit**: "Add GitHub PAT format validation"

---

### Task 1.6: Implement Refresh Interval Validation (RED)
**Type**: Test-Driven Development  
**Estimated Time**: 10 minutes

#### RED: Write Failing Tests
```rust
#[cfg(test)]
mod tests {
    // ... previous tests ...
    
    #[test]
    fn test_validate_refresh_interval_valid() {
        assert!(validate_refresh_interval(1).is_ok());
        assert!(validate_refresh_interval(15).is_ok());
        assert!(validate_refresh_interval(60).is_ok());
    }
    
    #[test]
    fn test_validate_refresh_interval_too_low() {
        let result = validate_refresh_interval(0);
        assert_eq!(result, Err(ConfigError::InvalidRefreshInterval));
    }
    
    #[test]
    fn test_validate_refresh_interval_too_high() {
        let result = validate_refresh_interval(61);
        assert_eq!(result, Err(ConfigError::InvalidRefreshInterval));
    }
}
```

Run: `cargo test --lib` → Tests should fail

#### GREEN: Implement Minimum Code
```rust
// src/core/config.rs

pub fn validate_refresh_interval(minutes: u32) -> Result<(), ConfigError> {
    if minutes < 1 || minutes > 60 {
        return Err(ConfigError::InvalidRefreshInterval);
    }
    Ok(())
}
```

Run: `cargo test --lib` → All tests should pass

#### REFACTOR: Clean Up
- Simple validation, no refactoring needed

**Acceptance Criteria**:
- [x] Validates range 1-60 minutes
- [x] Rejects 0 and values > 60
- [x] All tests passing

**Commit**: "Add refresh interval validation"

---

### Task 1.7: Implement ConfigManager - Default Config (RED)
**Type**: Test-Driven Development  
**Estimated Time**: 25 minutes

#### RED: Write Failing Tests
```rust
#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    
    fn get_test_config_path() -> PathBuf {
        let temp_dir = std::env::temp_dir();
        temp_dir.join("cosmic-copilot-test").join("config.toml")
    }
    
    fn cleanup_test_config() {
        let path = get_test_config_path();
        if let Some(parent) = path.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }
    
    #[test]
    fn test_config_manager_creates_default_config() {
        cleanup_test_config();
        
        let config_path = get_test_config_path();
        let manager = ConfigManager::new_with_path(config_path.clone()).unwrap();
        
        let config = manager.get_config();
        assert_eq!(config.organization_name, "");
        assert_eq!(config.refresh_interval_minutes, 15);
        
        // Verify file was created
        assert!(config_path.exists());
        
        cleanup_test_config();
    }
    
    #[test]
    fn test_config_manager_loads_existing_config() {
        cleanup_test_config();
        
        let config_path = get_test_config_path();
        
        // Create config file manually
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        
        let test_config = AppConfig {
            organization_name: "my-org".to_string(),
            refresh_interval_minutes: 30,
        };
        
        let toml_str = toml::to_string(&test_config).unwrap();
        fs::write(&config_path, toml_str).unwrap();
        
        // Load with ConfigManager
        let manager = ConfigManager::new_with_path(config_path.clone()).unwrap();
        let config = manager.get_config();
        
        assert_eq!(config.organization_name, "my-org");
        assert_eq!(config.refresh_interval_minutes, 30);
        
        cleanup_test_config();
    }
    
    #[test]
    fn test_config_manager_default_values() {
        let config = AppConfig::default();
        assert_eq!(config.organization_name, "");
        assert_eq!(config.refresh_interval_minutes, 15);
    }
}
```

Run: `cargo test --lib` → Tests should fail

#### GREEN: Implement Minimum Code
```rust
// src/core/config.rs

use std::fs;
use std::path::PathBuf;

pub struct ConfigManager {
    config: AppConfig,
    config_path: PathBuf,
}

impl AppConfig {
    pub fn default() -> Self {
        Self {
            organization_name: String::new(),
            refresh_interval_minutes: 15,
        }
    }
}

impl ConfigManager {
    pub fn new_with_path(config_path: PathBuf) -> Result<Self, ConfigError> {
        let config = if config_path.exists() {
            // Load existing config
            let contents = fs::read_to_string(&config_path)
                .map_err(|e| ConfigError::FileReadError(e.to_string()))?;
            
            toml::from_str(&contents)
                .map_err(|e| ConfigError::InvalidFormat(e.to_string()))?
        } else {
            // Create default config
            let default_config = AppConfig::default();
            
            // Create parent directories
            if let Some(parent) = config_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| ConfigError::FileWriteError(e.to_string()))?;
            }
            
            // Write default config to file
            let toml_str = toml::to_string(&default_config)
                .map_err(|e| ConfigError::InvalidFormat(e.to_string()))?;
            
            fs::write(&config_path, toml_str)
                .map_err(|e| ConfigError::FileWriteError(e.to_string()))?;
            
            default_config
        };
        
        Ok(Self {
            config,
            config_path,
        })
    }
    
    pub fn get_config(&self) -> &AppConfig {
        &self.config
    }
}
```

Run: `cargo test --lib` → All tests should pass

#### REFACTOR: Clean Up
- Consider extracting file I/O to helper methods
- Current implementation is functional

**Acceptance Criteria**:
- [x] Creates default config on first run
- [x] Loads existing config from file
- [x] Creates parent directories if needed
- [x] Default values are correct
- [x] All tests passing

**Commit**: "Add ConfigManager with default config creation and loading"

---

### Task 1.8: Implement ConfigManager - Save Config (RED)
**Type**: Test-Driven Development  
**Estimated Time**: 20 minutes

#### RED: Write Failing Tests
```rust
#[cfg(test)]
mod tests {
    // ... previous tests ...
    
    #[test]
    fn test_config_manager_saves_config() {
        cleanup_test_config();
        
        let config_path = get_test_config_path();
        let mut manager = ConfigManager::new_with_path(config_path.clone()).unwrap();
        
        let new_config = AppConfig {
            organization_name: "new-org".to_string(),
            refresh_interval_minutes: 45,
        };
        
        manager.save_config(&new_config).unwrap();
        
        // Verify in-memory config updated
        assert_eq!(manager.get_config().organization_name, "new-org");
        assert_eq!(manager.get_config().refresh_interval_minutes, 45);
        
        // Verify file updated
        let contents = fs::read_to_string(&config_path).unwrap();
        assert!(contents.contains("new-org"));
        assert!(contents.contains("45"));
        
        cleanup_test_config();
    }
    
    #[test]
    fn test_config_manager_validates_on_save() {
        cleanup_test_config();
        
        let config_path = get_test_config_path();
        let mut manager = ConfigManager::new_with_path(config_path.clone()).unwrap();
        
        // Try to save invalid config
        let invalid_config = AppConfig {
            organization_name: "".to_string(),  // Empty - should fail
            refresh_interval_minutes: 15,
        };
        
        let result = manager.save_config(&invalid_config);
        assert_eq!(result, Err(ConfigError::EmptyOrgName));
        
        cleanup_test_config();
    }
    
    #[test]
    fn test_config_manager_persists_across_instances() {
        cleanup_test_config();
        
        let config_path = get_test_config_path();
        
        // Create first instance and save
        {
            let mut manager = ConfigManager::new_with_path(config_path.clone()).unwrap();
            let new_config = AppConfig {
                organization_name: "persisted-org".to_string(),
                refresh_interval_minutes: 20,
            };
            manager.save_config(&new_config).unwrap();
        }
        
        // Create second instance and verify
        {
            let manager = ConfigManager::new_with_path(config_path.clone()).unwrap();
            assert_eq!(manager.get_config().organization_name, "persisted-org");
            assert_eq!(manager.get_config().refresh_interval_minutes, 20);
        }
        
        cleanup_test_config();
    }
}
```

Run: `cargo test --lib` → Tests should fail

#### GREEN: Implement Minimum Code
```rust
// src/core/config.rs

impl ConfigManager {
    pub fn save_config(&mut self, config: &AppConfig) -> Result<(), ConfigError> {
        // Validate before saving
        validate_org_name(&config.organization_name)?;
        validate_refresh_interval(config.refresh_interval_minutes)?;
        
        // Serialize to TOML
        let toml_str = toml::to_string(config)
            .map_err(|e| ConfigError::InvalidFormat(e.to_string()))?;
        
        // Write to file
        fs::write(&self.config_path, toml_str)
            .map_err(|e| ConfigError::FileWriteError(e.to_string()))?;
        
        // Update in-memory config
        self.config = config.clone();
        
        Ok(())
    }
}
```

Run: `cargo test --lib` → All tests should pass

#### REFACTOR: Clean Up
- Consider atomic write (write to temp, then rename)
- Current implementation is functional for MVP

**Acceptance Criteria**:
- [x] Validates config before saving
- [x] Writes to file successfully
- [x] Updates in-memory config
- [x] Persists across instances
- [x] All tests passing

**Commit**: "Add ConfigManager save functionality with validation"

---

### Task 1.9: Add XDG Directory Support (RED)
**Type**: Test-Driven Development  
**Estimated Time**: 15 minutes

#### RED: Write Failing Tests
```rust
#[cfg(test)]
mod tests {
    // ... previous tests ...
    
    #[test]
    fn test_config_manager_uses_xdg_path() {
        // Note: This test verifies the path structure, not actual XDG behavior
        let manager = ConfigManager::new().unwrap();
        let path = manager.get_config_path();
        
        // Path should contain cosmic-copilot-quota-tracker
        assert!(path.to_string_lossy().contains("cosmic-copilot-quota-tracker"));
        assert!(path.to_string_lossy().ends_with("config.toml"));
    }
}
```

Run: `cargo test --lib` → Tests should fail

#### GREEN: Implement Minimum Code
```rust
// src/core/config.rs

use directories::ProjectDirs;

impl ConfigManager {
    pub fn new() -> Result<Self, ConfigError> {
        let config_path = Self::get_default_config_path()?;
        Self::new_with_path(config_path)
    }
    
    fn get_default_config_path() -> Result<PathBuf, ConfigError> {
        let proj_dirs = ProjectDirs::from("com", "cosmic", "copilot-quota-tracker")
            .ok_or(ConfigError::FileNotFound)?;
        
        let config_dir = proj_dirs.config_dir();
        Ok(config_dir.join("config.toml"))
    }
    
    pub fn get_config_path(&self) -> &PathBuf {
        &self.config_path
    }
}
```

Run: `cargo test --lib` → All tests should pass

#### REFACTOR: Clean Up
- XDG path construction is clean
- ProjectDirs handles XDG spec automatically

**Acceptance Criteria**:
- [x] Uses XDG_CONFIG_HOME or ~/.config
- [x] Path includes app identifier
- [x] new() method uses XDG path by default
- [x] All tests passing

**Commit**: "Add XDG directory support for config file location"

---

### Task 1.10: Add File Permissions (Unix) (RED)
**Type**: Test-Driven Development  
**Estimated Time**: 15 minutes

#### RED: Write Failing Tests
```rust
#[cfg(test)]
mod tests {
    // ... previous tests ...
    
    #[test]
    #[cfg(unix)]
    fn test_config_file_has_secure_permissions() {
        cleanup_test_config();
        
        let config_path = get_test_config_path();
        let _manager = ConfigManager::new_with_path(config_path.clone()).unwrap();
        
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(&config_path).unwrap();
        let permissions = metadata.permissions();
        
        // Should be 0600 (rw-------)
        assert_eq!(permissions.mode() & 0o777, 0o600);
        
        cleanup_test_config();
    }
}
```

Run: `cargo test --lib` → Tests should fail

#### GREEN: Implement Minimum Code
```rust
// src/core/config.rs

impl ConfigManager {
    #[cfg(unix)]
    fn set_secure_permissions(path: &PathBuf) -> Result<(), ConfigError> {
        use std::os::unix::fs::PermissionsExt;
        
        let mut perms = fs::metadata(path)
            .map_err(|e| ConfigError::PermissionError)?
            .permissions();
        
        perms.set_mode(0o600);  // rw-------
        
        fs::set_permissions(path, perms)
            .map_err(|_| ConfigError::PermissionError)?;
        
        Ok(())
    }
    
    #[cfg(not(unix))]
    fn set_secure_permissions(_path: &PathBuf) -> Result<(), ConfigError> {
        // Windows/other platforms: skip permission setting
        Ok(())
    }
    
    // Update new_with_path to set permissions after file creation
    pub fn new_with_path(config_path: PathBuf) -> Result<Self, ConfigError> {
        let config = if config_path.exists() {
            // Load existing config
            let contents = fs::read_to_string(&config_path)
                .map_err(|e| ConfigError::FileReadError(e.to_string()))?;
            
            toml::from_str(&contents)
                .map_err(|e| ConfigError::InvalidFormat(e.to_string()))?
        } else {
            // Create default config
            let default_config = AppConfig::default();
            
            // Create parent directories
            if let Some(parent) = config_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| ConfigError::FileWriteError(e.to_string()))?;
            }
            
            // Write default config to file
            let toml_str = toml::to_string(&default_config)
                .map_err(|e| ConfigError::InvalidFormat(e.to_string()))?;
            
            fs::write(&config_path, toml_str)
                .map_err(|e| ConfigError::FileWriteError(e.to_string()))?;
            
            // Set secure permissions
            Self::set_secure_permissions(&config_path)?;
            
            default_config
        };
        
        Ok(Self {
            config,
            config_path,
        })
    }
    
    // Update save_config to set permissions after write
    pub fn save_config(&mut self, config: &AppConfig) -> Result<(), ConfigError> {
        // Validate before saving
        validate_org_name(&config.organization_name)?;
        validate_refresh_interval(config.refresh_interval_minutes)?;
        
        // Serialize to TOML
        let toml_str = toml::to_string(config)
            .map_err(|e| ConfigError::InvalidFormat(e.to_string()))?;
        
        // Write to file
        fs::write(&self.config_path, toml_str)
            .map_err(|e| ConfigError::FileWriteError(e.to_string()))?;
        
        // Set secure permissions
        Self::set_secure_permissions(&self.config_path)?;
        
        // Update in-memory config
        self.config = config.clone();
        
        Ok(())
    }
}
```

Run: `cargo test --lib` → All tests should pass

#### REFACTOR: Clean Up
- Permission setting is properly isolated
- Works cross-platform (no-op on non-Unix)

**Acceptance Criteria**:
- [x] Config file set to 0600 on Unix systems
- [x] Permissions set on creation and save
- [x] No-op on non-Unix platforms
- [x] All tests passing

**Commit**: "Add secure file permissions (0600) for config file"

---

## Phase 2: Secure Storage with Keyring

### Task 2.1: Add Keyring Dependency
**Type**: Setup  
**Estimated Time**: 5 minutes

#### Steps:
1. Add to `Cargo.toml`:
   ```toml
   keyring = { version = "2.3", features = ["linux-secret-service"] }
   ```
2. Run `cargo build` to verify
3. Commit: "Add keyring dependency for secure PAT storage"

**Acceptance Criteria**:
- [ ] keyring dependency added
- [ ] Build succeeds

---

### Task 2.2: Implement SecureStorage (RED)
**Type**: Test-Driven Development  
**Estimated Time**: 30 minutes

#### RED: Write Failing Tests
```rust
// src/core/config.rs (add to end of file)

pub struct SecureStorage {
    service_name: String,
}

#[cfg(test)]
mod secure_storage_tests {
    use super::*;
    
    const TEST_SERVICE: &str = "cosmic-copilot-test";
    
    fn cleanup_test_keyring() {
        let storage = SecureStorage::new(TEST_SERVICE);
        let _ = storage.delete_pat();
    }
    
    #[test]
    fn test_store_and_retrieve_pat() {
        cleanup_test_keyring();
        
        let storage = SecureStorage::new(TEST_SERVICE);
        let test_pat = "ghp_1234567890123456789012345678901234567890";
        
        storage.store_pat(test_pat).unwrap();
        let retrieved = storage.get_pat().unwrap();
        
        assert_eq!(retrieved, test_pat);
        
        cleanup_test_keyring();
    }
    
    #[test]
    fn test_retrieve_nonexistent_pat() {
        cleanup_test_keyring();
        
        let storage = SecureStorage::new(TEST_SERVICE);
        let result = storage.get_pat();
        
        assert_eq!(result, Err(ConfigError::PatNotFound));
        
        cleanup_test_keyring();
    }
    
    #[test]
    fn test_delete_pat() {
        cleanup_test_keyring();
        
        let storage = SecureStorage::new(TEST_SERVICE);
        let test_pat = "ghp_1234567890123456789012345678901234567890";
        
        storage.store_pat(test_pat).unwrap();
        storage.delete_pat().unwrap();
        
        let result = storage.get_pat();
        assert_eq!(result, Err(ConfigError::PatNotFound));
        
        cleanup_test_keyring();
    }
    
    #[test]
    fn test_update_pat() {
        cleanup_test_keyring();
        
        let storage = SecureStorage::new(TEST_SERVICE);
        let pat1 = "ghp_1111111111111111111111111111111111111111";
        let pat2 = "ghp_2222222222222222222222222222222222222222";
        
        storage.store_pat(pat1).unwrap();
        storage.store_pat(pat2).unwrap();  // Update
        
        let retrieved = storage.get_pat().unwrap();
        assert_eq!(retrieved, pat2);
        
        cleanup_test_keyring();
    }
}
```

Run: `cargo test --lib` → Tests should fail

#### GREEN: Implement Minimum Code
```rust
// src/core/config.rs

use keyring::Entry;

impl SecureStorage {
    pub fn new(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
        }
    }
    
    pub fn store_pat(&self, token: &str) -> Result<(), ConfigError> {
        let entry = Entry::new(&self.service_name, "github_pat")
            .map_err(|e| ConfigError::KeyringAccessError(e.to_string()))?;
        
        entry.set_password(token)
            .map_err(|e| ConfigError::KeyringAccessError(e.to_string()))?;
        
        Ok(())
    }
    
    pub fn get_pat(&self) -> Result<String, ConfigError> {
        let entry = Entry::new(&self.service_name, "github_pat")
            .map_err(|e| ConfigError::KeyringAccessError(e.to_string()))?;
        
        entry.get_password()
            .map_err(|e| {
                if e.to_string().contains("not found") {
                    ConfigError::PatNotFound
                } else {
                    ConfigError::KeyringAccessError(e.to_string())
                }
            })
    }
    
    pub fn delete_pat(&self) -> Result<(), ConfigError> {
        let entry = Entry::new(&self.service_name, "github_pat")
            .map_err(|e| ConfigError::KeyringAccessError(e.to_string()))?;
        
        entry.delete_password()
            .map_err(|e| {
                if e.to_string().contains("not found") {
                    Ok(())  // Already deleted, not an error
                } else {
                    Err(ConfigError::KeyringAccessError(e.to_string()))
                }
            })?;
        
        Ok(())
    }
}
```

Run: `cargo test --lib` → All tests should pass (if keyring available)

**Note**: Tests may fail in CI/headless environments without keyring. Consider:
- Conditional test compilation with `#[ignore]` attribute
- Mock keyring for testing

#### REFACTOR: Clean Up
- Add helper method for entry creation
- Handle keyring unavailability gracefully

**Acceptance Criteria**:
- [x] Can store PAT in keyring
- [x] Can retrieve PAT from keyring
- [x] Can delete PAT from keyring
- [x] Can update PAT (overwrite)
- [x] All tests passing (with keyring available)

**Commit**: "Add SecureStorage for keyring-based PAT management"

---

### Task 2.3: Add PAT Masking Utility (RED)
**Type**: Test-Driven Development  
**Estimated Time**: 10 minutes

#### RED: Write Failing Tests
```rust
#[cfg(test)]
mod tests {
    // ... previous tests ...
    
    #[test]
    fn test_mask_pat_empty() {
        assert_eq!(mask_pat(""), "(not set)");
    }
    
    #[test]
    fn test_mask_pat_classic() {
        let pat = "ghp_1234567890123456789012345678901234567890";
        let masked = mask_pat(pat);
        assert!(masked.starts_with("ghp_123"));
        assert!(masked.contains("●"));
        assert!(!masked.contains("1234567890"));  // Shouldn't show digits
    }
    
    #[test]
    fn test_mask_pat_fine_grained() {
        let pat = "github_pat_11ABCDEFG1234567890";
        let masked = mask_pat(pat);
        assert!(masked.starts_with("github_"));
        assert!(masked.contains("●"));
    }
    
    #[test]
    fn test_mask_pat_short() {
        let pat = "short";
        let masked = mask_pat(pat);
        assert_eq!(masked, "●●●●●");
    }
}
```

Run: `cargo test --lib` → Tests should fail

#### GREEN: Implement Minimum Code
```rust
// src/core/config.rs

pub fn mask_pat(token: &str) -> String {
    if token.is_empty() {
        return "(not set)".to_string();
    }
    
    // Show first 7 characters, mask the rest
    if token.len() > 7 {
        let prefix = &token[..7];
        let mask = "●".repeat(token.len() - 7);
        format!("{}{}", prefix, mask)
    } else {
        "●".repeat(token.len())
    }
}
```

Run: `cargo test --lib` → All tests should pass

#### REFACTOR: Clean Up
- Simple and effective masking
- No refactoring needed

**Acceptance Criteria**:
- [x] Empty tokens show "(not set)"
- [x] Shows first 7 chars of token
- [x] Masks remaining chars with ●
- [x] Handles short tokens
- [x] All tests passing

**Commit**: "Add PAT masking utility for UI display"

---

## Phase 3: UI Integration

### Task 3.1: Update Message Enum
**Type**: Structural Change  
**Estimated Time**: 5 minutes

#### Steps:
1. Update `src/app.rs`:
   ```rust
   pub enum Message {
       // ... existing messages
       OpenSettings,
       CloseSettings,
       UpdateOrgName(String),
       UpdatePat(String),
       UpdateRefreshInterval(u32),
       SaveConfig,
   }
   ```
2. Run `cargo build` to verify
3. Commit: "Add settings-related messages to Message enum"

**Acceptance Criteria**:
- [ ] New messages added
- [ ] Build succeeds

---

### Task 3.2: Update App Struct with Config Fields
**Type**: Structural Change  
**Estimated Time**: 10 minutes

#### Steps:
1. Update `src/app.rs`:
   ```rust
   use crate::core::config::{ConfigManager, SecureStorage, ConfigError};
   
   pub struct CosmicCopilotQuotaTracker {
       core: Core,
       popup: Option<Id>,
       // Config management
       config_manager: ConfigManager,
       secure_storage: SecureStorage,
       // Settings UI state
       settings_dialog_open: bool,
       temp_org_name: String,
       temp_pat: String,
       temp_refresh_interval: u32,
       config_error: Option<ConfigError>,
   }
   ```
2. Update `init()` method to initialize config
3. Run `cargo build` to verify
4. Commit: "Add config management fields to app state"

**Acceptance Criteria**:
- [ ] Config fields added to struct
- [ ] Config initialized in init()
- [ ] Build succeeds

---

### Task 3.3: Implement Settings Dialog UI
**Type**: Feature Implementation  
**Estimated Time**: 45 minutes

#### Steps:
1. Add settings button to popup menu
2. Create settings dialog layout using libcosmic widgets
3. Add input fields for org name, PAT, refresh interval
4. Add masked PAT display with toggle visibility
5. Add save/cancel buttons
6. Test UI rendering

**Implementation**:
```rust
fn view_window(&self, _id: Id) -> Element<Self::Message> {
    let content_list = widget::list_column()
        .padding(5)
        .spacing(0);
    
    if self.settings_dialog_open {
        // Settings dialog content
        let org_input = widget::text_input("Organization name", &self.temp_org_name)
            .on_input(Message::UpdateOrgName);
        
        let pat_input = widget::text_input("Personal Access Token", &mask_pat(&self.temp_pat))
            .on_input(Message::UpdatePat);
        
        let interval_input = widget::text_input(
            "Refresh interval",
            &self.temp_refresh_interval.to_string()
        )
        .on_input(|s| {
            s.parse::<u32>()
                .map(Message::UpdateRefreshInterval)
                .unwrap_or(Message::UpdateRefreshInterval(15))
        });
        
        // Error display
        let error_text = if let Some(err) = &self.config_error {
            widget::text(format!("Error: {}", err)).into()
        } else {
            widget::text("").into()
        };
        
        let buttons = widget::row()
            .push(widget::button("Cancel").on_press(Message::CloseSettings))
            .push(widget::button("Save").on_press(Message::SaveConfig));
        
        content_list
            .add(settings::item("Organization", org_input))
            .add(settings::item("PAT", pat_input))
            .add(settings::item("Refresh Interval", interval_input))
            .add(error_text)
            .add(buttons)
    } else {
        // Normal popup with settings button
        content_list
            .add(widget::button("Settings").on_press(Message::OpenSettings))
    }
    
    self.core.applet.popup_container(content_list).into()
}
```

**Acceptance Criteria**:
- [ ] Settings button appears in popup
- [ ] Settings dialog opens on button click
- [ ] Input fields render correctly
- [ ] PAT is masked in UI
- [ ] Save/cancel buttons work

**Commit**: "Add settings dialog UI with input fields"

---

### Task 3.4: Implement Settings Message Handlers
**Type**: Feature Implementation  
**Estimated Time**: 30 minutes

#### Steps:
1. Implement `OpenSettings` handler
2. Implement `UpdateOrgName`, `UpdatePat`, `UpdateRefreshInterval` handlers
3. Implement `SaveConfig` handler with validation
4. Implement `CloseSettings` handler
5. Test message flow

**Implementation**: (See design.md for detailed update() implementation)

**Acceptance Criteria**:
- [ ] Settings open with current values
- [ ] Input changes update temp state
- [ ] Save validates and persists config
- [ ] Errors displayed on validation failure
- [ ] Dialog closes on save or cancel

**Commit**: "Implement settings message handlers with validation"

---

## Phase 4: Integration & Testing

### Task 4.1: Manual Testing
**Type**: Testing  
**Estimated Time**: 30 minutes

#### Test Scenarios:
1. **First Launch**:
   - [ ] Config file created at XDG location
   - [ ] Default values loaded
   - [ ] File permissions set to 0600

2. **Settings Flow**:
   - [ ] Settings dialog opens
   - [ ] Can enter org name and PAT
   - [ ] PAT is masked in UI
   - [ ] Save persists values
   - [ ] Config survives app restart

3. **Validation**:
   - [ ] Empty org name rejected
   - [ ] Invalid PAT format rejected
   - [ ] Invalid refresh interval rejected
   - [ ] Error messages displayed

4. **Keyring**:
   - [ ] PAT stored in keyring
   - [ ] PAT retrieved on launch
   - [ ] PAT can be updated

**Documentation**:
- Document any issues found
- Create bug tickets if needed

---

### Task 4.2: Run Full Test Suite
**Type**: Testing  
**Estimated Time**: 10 minutes

#### Steps:
1. Run `cargo test --lib`
2. Verify all tests pass
3. Fix any failures
4. Run `cargo clippy` for linting
5. Run `cargo fmt` for formatting

**Acceptance Criteria**:
- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] No clippy warnings
- [ ] Code properly formatted

**Commit**: "Feature 02 complete: Configuration & Authentication"

---

## Summary

### Total Estimated Time: ~5-6 hours

### Tasks Breakdown:
- **Phase 1**: 10 tasks (Core Config Module) - ~2.5 hours
- **Phase 2**: 3 tasks (Secure Storage) - ~45 minutes
- **Phase 3**: 4 tasks (UI Integration) - ~1.5 hours
- **Phase 4**: 2 tasks (Testing) - ~40 minutes

### Test Coverage Goals:
- Unit tests for all validation functions
- Unit tests for ConfigManager operations
- Unit tests for SecureStorage operations
- Integration tests for config persistence
- Manual testing of UI workflow

### Completion Criteria:
- [ ] All tests passing
- [ ] Config file created at XDG location with 0600 permissions
- [ ] PAT stored securely in keyring
- [ ] Settings UI functional and validated
- [ ] Changes persist across app restarts
- [ ] No compiler warnings or clippy issues

### Next Feature:
After completion: **Feature 03 - GitHub API Client**
