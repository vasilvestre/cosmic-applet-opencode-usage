# Feature 07: Error Handling & Fallback UI

## Overview
Implement comprehensive error handling for all failure scenarios with user-friendly fallback UI states, ensuring the applet remains stable and informative even when things go wrong.

## Requirements

### Network & API Errors

**REQ-07.1**: WHEN the API request fails due to network connectivity issues THEN the system SHALL:
- Display "Network Error: Unable to reach GitHub API" message
- Show cached data if available
- Provide a "Retry" button

**REQ-07.2**: WHEN the API returns HTTP 401 (Unauthorized) THEN the system SHALL:
- Display "Authentication Failed: Invalid or expired token"
- Provide a button to open configuration settings
- Log the error details

**REQ-07.3**: WHEN the API returns HTTP 403 (Forbidden) THEN the system SHALL:
- Display "Access Denied: Check organization permissions and API policy"
- Explain possible causes (insufficient permissions, API not enabled)
- Provide a link to GitHub documentation

**REQ-07.4**: WHEN the API returns HTTP 404 (Not Found) THEN the system SHALL:
- Display "Organization not found or you don't have access"
- Prompt user to verify organization name in configuration
- Provide a button to open configuration settings

**REQ-07.5**: WHEN the API returns HTTP 500 or 503 (Server Error) THEN the system SHALL:
- Display "GitHub API is temporarily unavailable"
- Show cached data if available
- Automatically retry after 5 minutes

**REQ-07.6**: WHEN the API returns an error indicating fewer than 5 active users THEN the system SHALL:
- Display "Insufficient Data: Metrics require at least 5 active Copilot users"
- Explain this is a GitHub API limitation
- Show last successful data if available in cache

**REQ-07.7**: WHEN the API request times out (no response within 30 seconds) THEN the system SHALL:
- Display "Request Timeout: GitHub API did not respond"
- Show cached data if available
- Provide a "Retry" button

### Configuration Errors

**REQ-07.8**: WHEN the applet starts, IF no configuration file exists THEN the system SHALL:
- Display "Setup Required: Please configure your GitHub credentials"
- Show a prominent "Configure" button
- Provide setup instructions

**REQ-07.9**: WHEN loading configuration, IF the config file is corrupted or has invalid JSON THEN the system SHALL:
- Display "Configuration Error: Invalid configuration file"
- Create a backup of the corrupted file
- Create a new default configuration file
- Prompt user to reconfigure

**REQ-07.10**: WHEN validating configuration, IF required fields are missing THEN the system SHALL:
- Display "Incomplete Configuration: Missing [field name]"
- Highlight which fields need to be provided
- Provide a button to open configuration settings

**REQ-07.11**: IF the configured organization name contains invalid characters THEN the system SHALL:
- Display "Invalid Organization Name: Use only alphanumeric characters and hyphens"
- Prevent API calls until corrected
- Provide configuration editing option

### Data Parsing Errors

**REQ-07.12**: WHEN parsing API response, IF the response structure doesn't match expected schema THEN the system SHALL:
- Log the unexpected structure for debugging
- Display "Data Format Error: Unexpected API response format"
- Show cached data if available
- Report the error with response details

**REQ-07.13**: WHEN parsing API response, IF required fields are missing THEN the system SHALL:
- Use default/zero values for missing metrics
- Log which fields are missing
- Display available data with a warning indicator

**REQ-07.14**: WHEN parsing dates/timestamps, IF format is invalid THEN the system SHALL:
- Use current timestamp as fallback
- Log the parsing error
- Continue processing other data

### Cache Errors

**REQ-07.15**: WHEN attempting to write cache file, IF disk is full THEN the system SHALL:
- Display "Unable to save cache: Disk full"
- Continue operating with in-memory data only
- Retry cache save on next successful API fetch

**REQ-07.16**: WHEN attempting to write cache file, IF permissions are insufficient THEN the system SHALL:
- Display "Unable to save cache: Permission denied on ~/.config/copilot-usage-monitor/"
- Log the permission error with path details
- Continue operating with in-memory data only

**REQ-07.17**: WHEN loading cache, IF cache file is corrupted THEN the system SHALL:
- Rename corrupted file to `cache.json.corrupted.[timestamp]`
- Display "Cache corrupted, fetching fresh data"
- Proceed with fresh API call

### Fallback UI States

**REQ-07.18**: The system SHALL implement the following UI states:
- **Loading**: Animated spinner with "Fetching metrics..." text
- **Success**: Normal metrics display
- **Error**: Error icon with descriptive message and action buttons
- **Offline**: Warning icon with "Showing cached data from [timestamp]"
- **Unconfigured**: Setup prompt with configuration button

**REQ-07.19**: WHEN in an error state, the UI SHALL display:
- Clear error icon (⚠️ or similar)
- Human-readable error message (no technical jargon)
- Suggested action or remedy
- Timestamp of when error occurred
- "Retry" button (if retry is applicable)

**REQ-07.20**: WHEN showing cached data due to an error, the UI SHALL:
- Display a distinct visual indicator (e.g., orange/yellow color)
- Show timestamp of cached data age
- Display reason for using cache (e.g., "Network unavailable")
- Provide manual refresh option

### Error Logging

**REQ-07.21**: The system SHALL log all errors to `~/.config/copilot-usage-monitor/errors.log`.

**REQ-07.22**: Error log entries SHALL include:
- Timestamp (ISO 8601 format)
- Error severity level (ERROR, WARN, INFO)
- Error type/category
- Detailed error message
- Context information (API endpoint, request details if applicable)
- Stack trace (for unexpected errors)

**REQ-07.23**: The error log file SHALL be rotated when it exceeds 1MB in size.

**REQ-07.24**: The system SHALL keep the last 5 rotated log files and delete older ones.

**REQ-07.25**: WHEN rotating logs, the system SHALL name rotated files as `errors.log.1`, `errors.log.2`, etc.

### Error Recovery

**REQ-07.26**: WHEN an error occurs during background refresh, the system SHALL implement exponential backoff retry logic:
- 1st retry: 5 minutes
- 2nd retry: 15 minutes
- 3rd retry: 45 minutes
- After 3 failures: wait until next scheduled refresh (6 hours)

**REQ-07.27**: WHEN network connectivity is restored after network errors, the system SHALL automatically attempt to fetch fresh data on the next scheduled refresh.

**REQ-07.28**: WHEN transitioning from error state to success state, the system SHALL:
- Clear error indicators from UI
- Update to normal display mode
- Show success notification (optional, subtle)

**REQ-07.29**: The system SHALL NOT crash or become unresponsive due to any error condition.

**REQ-07.30**: WHEN unexpected panics occur in Rust code, the system SHALL catch them at thread boundaries and log the panic details.

## Acceptance Criteria

### AC-07.1: Network Error Handling
- [ ] Offline state is properly detected and communicated
- [ ] Network errors don't crash the applet
- [ ] Cached data is shown when network is unavailable
- [ ] Retry mechanism works correctly

### AC-07.2: API Error Handling
- [ ] All HTTP error codes (401, 403, 404, 500, 503) are handled
- [ ] Error messages are user-friendly and actionable
- [ ] Rate limit errors are properly communicated
- [ ] Timeout errors are caught and handled

### AC-07.3: Configuration Error Handling
- [ ] Missing configuration is detected on startup
- [ ] Corrupted configuration is backed up and recreated
- [ ] Invalid values are validated before use
- [ ] User is guided to fix configuration issues

### AC-07.4: Fallback UI States
- [ ] All UI states (loading, success, error, offline, unconfigured) are implemented
- [ ] State transitions are smooth and clear
- [ ] Error messages include actionable suggestions
- [ ] Visual indicators match state (colors, icons)

### AC-07.5: Error Logging
- [ ] All errors are logged to file
- [ ] Log entries include all required information
- [ ] Log rotation works when file exceeds 1MB
- [ ] Old logs are cleaned up (keeps last 5)

### AC-07.6: Error Recovery
- [ ] Exponential backoff retry logic is implemented
- [ ] Successful recovery transitions UI back to normal
- [ ] No memory leaks from repeated error/recovery cycles
- [ ] Background refresh continues after errors

### AC-07.7: Stability
- [ ] No error condition crashes the applet
- [ ] Panics are caught and logged
- [ ] UI remains responsive during error states
- [ ] Resource cleanup occurs properly during errors

## Dependencies

- **Feature 02**: Configuration & Auth (handles config loading/validation)
- **Feature 03**: GitHub API Client (generates API errors)
- **Feature 04**: Basic UI Panel (displays error states)
- **Feature 06**: Auto-Refresh & Caching (triggers errors, provides fallback data)

## Technical Notes

### Error Types to Implement (Rust)
```rust
#[derive(Debug)]
pub enum AppError {
    NetworkError(String),
    ApiError { status: u16, message: String },
    AuthenticationError(String),
    ConfigurationError(String),
    CacheError(String),
    ParseError(String),
    TimeoutError,
    RateLimitExceeded { reset_at: DateTime<Utc> },
    InsufficientUsers,
}
```

### Error Logging Format
```
[2025-09-30T10:30:45Z] ERROR NetworkError: Failed to connect to api.github.com
  Context: GET /orgs/my-org/copilot/metrics
  Details: Connection timeout after 30s
  Stack: ...
```

### User-Facing vs. Technical Messages
- **User-facing**: "Unable to connect to GitHub. Check your internet connection."
- **Log message**: "NetworkError: ConnectionTimeout after 30000ms to api.github.com:443"

### Recovery Strategy Priority
1. Try cached data first
2. Implement retry with backoff
3. Provide manual retry option
4. Guide user to fix configuration if needed
5. Maintain applet stability above all else
