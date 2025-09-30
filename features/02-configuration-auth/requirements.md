# Feature: Configuration & Authentication

## Overview
Implement a secure mechanism to store and access the user's GitHub Personal Access Token (PAT), organization name, and other configuration settings required for the applet.

## Requirements

### Configuration Storage
- The system SHALL store configuration in a secure location following XDG Base Directory specification
- The system SHALL support the following configuration parameters:
  - GitHub Personal Access Token (PAT)
  - GitHub organization name
  - Refresh interval (in minutes)
- WHEN no configuration file exists THEN the system SHALL create one with default values
- The system SHALL validate configuration file format on load

### Security
- The system SHALL store the GitHub PAT with restricted file permissions (0600)
- WHEN storing sensitive data THEN the system SHALL use appropriate file system permissions
- The system SHALL NOT log or display the PAT in plain text
- WHERE the user provides an invalid PAT THEN the system SHALL display a clear error message

### Configuration UI
- The system SHALL provide a configuration interface accessible from the applet menu
- WHEN the user opens settings THEN the system SHALL display current configuration values (PAT masked)
- The system SHALL allow users to update configuration values
- WHEN configuration is updated THEN the system SHALL validate new values before saving
- WHEN configuration is saved THEN the system SHALL apply changes without requiring restart

### Validation
- The system SHALL validate PAT format (GitHub token structure)
- The system SHALL validate organization name is not empty
- The system SHALL validate refresh interval is between 1 and 60 minutes
- WHEN validation fails THEN the system SHALL display specific error messages

## Acceptance Criteria
- [ ] Configuration file created in XDG_CONFIG_HOME/copilot-usage-monitor/
- [ ] PAT stored with 0600 permissions
- [ ] Configuration can be read and parsed successfully
- [ ] Settings UI accessible from applet menu
- [ ] PAT displayed as masked characters in UI
- [ ] Configuration values can be updated through UI
- [ ] Invalid configuration triggers appropriate error messages
- [ ] Changes applied without restart

## Technical Notes
- Use XDG_CONFIG_HOME environment variable (default: ~/.config)
- Consider using keyring crate for secure PAT storage
- Use serde for configuration serialization (TOML or JSON)
- Follow libcosmic patterns for settings UI

## Dependencies
- Feature 01: Project Setup & Boilerplate Applet
