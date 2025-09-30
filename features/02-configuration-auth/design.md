# Technical Design: Configuration & Authentication

## Overview
This design document outlines the technical approach for implementing secure configuration storage and GitHub Personal Access Token (PAT) management for the COSMIC Copilot Quota Tracker applet. The implementation follows XDG Base Directory specifications and uses COSMIC's configuration patterns for secure persistence.

## Architecture

### Module Organization
```
src/
├── core/
│   ├── mod.rs           # Export config module
│   ├── config.rs        # Configuration management (NEW)
│   ├── models.rs        # Domain models (existing)
│   └── localization.rs  # i18n support (existing)
└── app.rs               # Integrate config into app state
```

### Core Components

#### 1. AppConfig Struct
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    pub organization_name: String,
    pub refresh_interval_minutes: u32,
    // PAT stored separately in keyring
}
```

**Purpose**: Store non-sensitive configuration
**Responsibilities**:
- Manage organization name
- Store refresh interval
- Serialize/deserialize to/from TOML

#### 2. ConfigManager
```rust
pub struct ConfigManager {
    config: AppConfig,
    config_path: PathBuf,
}
```

**Purpose**: Handle configuration persistence and validation
**Responsibilities**:
- Load configuration from disk
- Save configuration with validation
- Provide default values
- Ensure proper file permissions

#### 3. SecureStorage (Keyring Integration)
```rust
pub struct SecureStorage {
    service_name: &'static str,
}
```

**Purpose**: Secure PAT storage using system keyring
**Responsibilities**:
- Store PAT in system keyring
- Retrieve PAT securely
- Delete PAT when needed
- Handle keyring errors gracefully

## Configuration Storage Strategy

### Two-Tier Storage Approach

**Tier 1: TOML Config File (Non-Sensitive)**
- Location: `$XDG_CONFIG_HOME/cosmic-copilot-quota-tracker/config.toml`
- Permissions: 0644 (readable by user)
- Contents: organization_name, refresh_interval_minutes
- Format: TOML using serde

**Tier 2: System Keyring (Sensitive)**
- Service: "cosmic-copilot-quota-tracker"
- Username: "github_pat"
- Contents: GitHub Personal Access Token
- Backend: libsecret (Linux), keychain (macOS), Credential Manager (Windows)

### Default Configuration
```toml
organization_name = ""
refresh_interval_minutes = 15
```

### Configuration File Lifecycle

1. **First Launch**:
   - Check if config file exists at XDG path
   - If not, create parent directories
   - Write default config to file
   - Check keyring for existing PAT

2. **Subsequent Launches**:
   - Read config file
   - Parse TOML with serde
   - Validate fields
   - Retrieve PAT from keyring if present

3. **Configuration Updates**:
   - Validate new values
   - Write to config file atomically
   - Update PAT in keyring if changed
   - Notify app of changes

## Validation Strategy

### Organization Name Validation
```rust
fn validate_org_name(name: &str) -> Result<(), ConfigError> {
    if name.is_empty() {
        return Err(ConfigError::EmptyOrgName);
    }
    if name.len() > 39 {  // GitHub limit
        return Err(ConfigError::OrgNameTooLong);
    }
    // GitHub org names: alphanumeric and hyphens, no consecutive hyphens
    let valid_chars = name.chars().all(|c| c.is_alphanumeric() || c == '-');
    if !valid_chars || name.starts_with('-') || name.ends_with('-') {
        return Err(ConfigError::InvalidOrgNameFormat);
    }
    Ok(())
}
```

### PAT Validation
```rust
fn validate_pat(token: &str) -> Result<(), ConfigError> {
    if token.is_empty() {
        return Err(ConfigError::EmptyPat);
    }
    // GitHub PAT formats:
    // Classic: ghp_[40 alphanumeric]
    // Fine-grained: github_pat_[82 alphanumeric + underscore]
    let is_classic = token.starts_with("ghp_") && token.len() == 44;
    let is_fine_grained = token.starts_with("github_pat_") && token.len() >= 93;
    
    if !is_classic && !is_fine_grained {
        return Err(ConfigError::InvalidPatFormat);
    }
    Ok(())
}
```

### Refresh Interval Validation
```rust
fn validate_refresh_interval(minutes: u32) -> Result<(), ConfigError> {
    if minutes < 1 || minutes > 60 {
        return Err(ConfigError::InvalidRefreshInterval);
    }
    Ok(())
}
```

## Error Handling

### ConfigError Enum
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError {
    // File I/O errors
    FileNotFound,
    FileReadError(String),
    FileWriteError(String),
    PermissionError,
    
    // Parsing errors
    InvalidFormat(String),
    
    // Validation errors
    EmptyOrgName,
    OrgNameTooLong,
    InvalidOrgNameFormat,
    EmptyPat,
    InvalidPatFormat,
    InvalidRefreshInterval,
    
    // Keyring errors
    KeyringNotAvailable,
    KeyringAccessError(String),
    PatNotFound,
}
```

### Error Display Strategy
- User-friendly error messages in UI
- Detailed errors logged for debugging
- Specific error messages for each validation failure

## UI Integration

### Settings UI Components

#### 1. Settings Button in Popup Menu
```rust
// Add to Message enum
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

#### 2. Settings Dialog Layout
```
┌─────────────────────────────────────┐
│  Copilot Quota Tracker Settings    │
├─────────────────────────────────────┤
│                                     │
│  GitHub Organization                │
│  [_____________________________]    │
│                                     │
│  Personal Access Token              │
│  [●●●●●●●●●●●●●●●●●●●●●●●●●●●●●] [👁] │
│                                     │
│  Refresh Interval (minutes)         │
│  [15____] (1-60)                    │
│                                     │
│  [Cancel]              [Save]       │
│                                     │
└─────────────────────────────────────┘
```

#### 3. PAT Masking Strategy
```rust
fn mask_pat(token: &str) -> String {
    if token.is_empty() {
        return String::from("(not set)");
    }
    // Show first 7 chars (e.g., "ghp_" or "github_"), mask rest
    if token.len() > 7 {
        format!("{}{'●'.to_string().repeat(token.len() - 7)}", &token[..7])
    } else {
        "●".repeat(token.len())
    }
}
```

### Settings Access Flow
1. User clicks settings icon in popup
2. `OpenSettings` message sent
3. Load current config values
4. Retrieve PAT from keyring (masked for display)
5. Render settings dialog
6. User modifies values
7. On save: validate → save config → update keyring → close dialog
8. Display error if validation fails

## Dependency Management

### New Dependencies

```toml
[dependencies]
# Existing dependencies...

# Configuration management
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"

# Secure storage
keyring = { version = "2.3", features = ["linux-secret-service"] }

# XDG directories
directories = "5.0"

# Error handling
thiserror = "1.0"
```

### Platform Support
- **Linux**: Use libsecret for keyring (default on COSMIC)
- **macOS**: Use macOS Keychain (if cross-platform needed)
- **Windows**: Use Windows Credential Manager (if cross-platform needed)

## Security Considerations

### File Permissions
```rust
#[cfg(unix)]
fn set_secure_permissions(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path)?.permissions();
    perms.set_mode(0o600);  // rw------- (owner only)
    std::fs::set_permissions(path, perms)
}
```

### PAT Handling Rules
1. **Never log PAT in plain text**
2. **Never display PAT unmasked by default**
3. **Clear PAT from memory after use**
4. **Use keyring for storage, not config file**
5. **Validate PAT format before storage**

### Threat Mitigation
- **File read attacks**: Config file readable by user only
- **Memory dumps**: Minimize PAT lifetime in memory
- **Log exposure**: Never log sensitive data
- **Config backup**: PAT not in config file, so safe to backup

## Testing Strategy

### Unit Tests

#### ConfigManager Tests
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_default_config_creation() { }
    
    #[test]
    fn test_load_valid_config() { }
    
    #[test]
    fn test_save_config() { }
    
    #[test]
    fn test_validate_org_name_valid() { }
    
    #[test]
    fn test_validate_org_name_empty() { }
    
    #[test]
    fn test_validate_org_name_too_long() { }
    
    #[test]
    fn test_validate_org_name_invalid_chars() { }
    
    #[test]
    fn test_validate_refresh_interval_valid() { }
    
    #[test]
    fn test_validate_refresh_interval_too_low() { }
    
    #[test]
    fn test_validate_refresh_interval_too_high() { }
}
```

#### PAT Validation Tests
```rust
#[test]
fn test_validate_pat_classic_format() { }

#[test]
fn test_validate_pat_fine_grained_format() { }

#[test]
fn test_validate_pat_empty() { }

#[test]
fn test_validate_pat_invalid_format() { }

#[test]
fn test_mask_pat() { }
```

#### SecureStorage Tests
```rust
#[test]
fn test_store_and_retrieve_pat() { }

#[test]
fn test_delete_pat() { }

#[test]
fn test_retrieve_nonexistent_pat() { }
```

### Integration Tests
- Test config file creation on first launch
- Test config persistence across app restarts
- Test keyring integration
- Test settings UI workflow

### Manual Testing Checklist
- [ ] Config file created at correct XDG location
- [ ] Default values applied on first launch
- [ ] PAT stored in keyring successfully
- [ ] PAT retrieved from keyring on launch
- [ ] Settings dialog opens and displays values
- [ ] PAT masked in settings UI
- [ ] Validation errors displayed correctly
- [ ] Config updates persist after restart
- [ ] File permissions set to 0600

## State Management Integration

### App Struct Updates
```rust
pub struct CosmicCopilotQuotaTracker {
    core: Core,
    popup: Option<Id>,
    config_manager: ConfigManager,
    secure_storage: SecureStorage,
    settings_dialog_open: bool,
    // Temporary state for settings editing
    temp_org_name: String,
    temp_pat: String,
    temp_refresh_interval: u32,
    config_error: Option<ConfigError>,
}
```

### Initialization Flow
```rust
fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
    // Load config from file
    let config_manager = match ConfigManager::new() {
        Ok(cm) => cm,
        Err(e) => {
            eprintln!("Config error: {:?}, using defaults", e);
            ConfigManager::with_defaults()
        }
    };
    
    // Initialize secure storage
    let secure_storage = SecureStorage::new("cosmic-copilot-quota-tracker");
    
    let app = Self {
        core,
        popup: None,
        config_manager,
        secure_storage,
        settings_dialog_open: false,
        temp_org_name: String::new(),
        temp_pat: String::new(),
        temp_refresh_interval: 15,
        config_error: None,
    };
    
    (app, Task::none())
}
```

### Update Handler for Config Messages
```rust
fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
    match message {
        Message::OpenSettings => {
            let config = self.config_manager.get_config();
            self.temp_org_name = config.organization_name.clone();
            self.temp_refresh_interval = config.refresh_interval_minutes;
            
            // Retrieve PAT from keyring
            self.temp_pat = match self.secure_storage.get_pat() {
                Ok(pat) => pat,
                Err(_) => String::new(),
            };
            
            self.settings_dialog_open = true;
            self.config_error = None;
            Task::none()
        }
        
        Message::UpdateOrgName(name) => {
            self.temp_org_name = name;
            Task::none()
        }
        
        Message::UpdatePat(pat) => {
            self.temp_pat = pat;
            Task::none()
        }
        
        Message::UpdateRefreshInterval(interval) => {
            self.temp_refresh_interval = interval;
            Task::none()
        }
        
        Message::SaveConfig => {
            // Validate all fields
            if let Err(e) = validate_org_name(&self.temp_org_name) {
                self.config_error = Some(e);
                return Task::none();
            }
            
            if let Err(e) = validate_pat(&self.temp_pat) {
                self.config_error = Some(e);
                return Task::none();
            }
            
            if let Err(e) = validate_refresh_interval(self.temp_refresh_interval) {
                self.config_error = Some(e);
                return Task::none();
            }
            
            // Save to config file
            let new_config = AppConfig {
                organization_name: self.temp_org_name.clone(),
                refresh_interval_minutes: self.temp_refresh_interval,
            };
            
            if let Err(e) = self.config_manager.save_config(&new_config) {
                self.config_error = Some(e);
                return Task::none();
            }
            
            // Save PAT to keyring
            if let Err(e) = self.secure_storage.store_pat(&self.temp_pat) {
                self.config_error = Some(e);
                return Task::none();
            }
            
            // Success: close dialog
            self.settings_dialog_open = false;
            self.config_error = None;
            Task::none()
        }
        
        Message::CloseSettings => {
            self.settings_dialog_open = false;
            self.config_error = None;
            Task::none()
        }
        
        _ => Task::none()
    }
}
```

## Migration Notes

### Existing Code Changes Required
1. **Update src/core/mod.rs**: Add `pub mod config;`
2. **Update src/app.rs**: Add config fields to app struct
3. **Update Cargo.toml**: Add new dependencies
4. **Update src/lib.rs**: Export config module if needed for tests

### Backward Compatibility
- First version: no existing config to migrate
- Future versions: add version field to config for migrations

## Performance Considerations

### Config Loading
- Load on init: ~1-5ms (file read + TOML parse)
- Keyring access: ~10-50ms (system keyring call)
- Cache config in memory to avoid repeated disk access

### Config Saving
- Atomic write pattern to prevent corruption
- Write to temp file, then rename (atomic on POSIX)
- Validate before write to avoid partial updates

## Implementation Phases

### Phase 1: Core Config Module (TDD)
1. Write tests for AppConfig struct
2. Implement AppConfig with serde derives
3. Write tests for validation functions
4. Implement validation functions
5. Write tests for ConfigManager
6. Implement ConfigManager

### Phase 2: Secure Storage (TDD)
1. Write tests for SecureStorage
2. Implement SecureStorage with keyring
3. Handle keyring errors gracefully
4. Add fallback for keyring unavailability

### Phase 3: UI Integration
1. Add settings button to popup
2. Create settings dialog layout
3. Implement PAT masking
4. Wire up message handlers
5. Add error display

### Phase 4: Integration Testing
1. Test end-to-end config workflow
2. Test persistence across restarts
3. Test error scenarios
4. Manual testing on COSMIC desktop

## Next Steps

After design approval:
1. Create tasks.md with detailed TDD breakdown
2. Implement Phase 1 with Red-Green-Refactor
3. Implement Phase 2 with Red-Green-Refactor
4. Implement Phase 3 with UI testing
5. Perform integration testing
6. Document configuration usage

## References

- XDG Base Directory Specification
- GitHub PAT documentation
- keyring crate documentation
- COSMIC applet patterns
- serde documentation
