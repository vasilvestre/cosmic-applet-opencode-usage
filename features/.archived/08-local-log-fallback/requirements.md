# Feature 08: Local Log Fallback Mechanism

## Overview
Implement an alternative data source using local Copilot usage logs as a fallback when GitHub API is unavailable, providing basic usage insights even without API access.

## Requirements

### Log File Discovery

**REQ-08.1**: The system SHALL attempt to locate Copilot usage logs in standard locations for supported editors:
- VS Code: `~/.vscode/extensions/github.copilot-*/logs/`
- JetBrains IDEs: `~/.cache/JetBrains/*/github-copilot/`
- Neovim: `~/.local/share/nvim/copilot/`

**REQ-08.2**: IF Copilot log directories are found THEN the system SHALL scan for log files with patterns:
- `copilot*.log`
- `*.log`
- Files modified within the last 30 days

**REQ-08.3**: The system SHALL maintain a list of discovered log file paths in memory.

**REQ-08.4**: The system SHALL re-scan for log files every 24 hours to detect newly installed editors.

### Log Parsing

**REQ-08.5**: The system SHALL parse log files to extract usage events including:
- Code completion requests
- Code completion acceptances
- Timestamps of events
- Programming language context (if available)

**REQ-08.6**: WHEN parsing log files, the system SHALL handle multiple log formats from different editors gracefully.

**REQ-08.7**: IF a log line cannot be parsed, the system SHALL skip that line and continue processing.

**REQ-08.8**: The system SHALL extract dates from log entries to group events by day.

**REQ-08.9**: The system SHALL track the last processed position in each log file to avoid re-parsing on subsequent reads.

### Metrics Aggregation

**REQ-08.10**: The system SHALL aggregate local log data into metrics:
- Total completions requested (last 7 days)
- Total completions accepted (last 7 days)
- Acceptance rate (accepted / requested)
- Daily breakdown of activity
- Most active editor (if multiple detected)

**REQ-08.11**: The system SHALL calculate an acceptance rate percentage: `(acceptances / requests) * 100`.

**REQ-08.12**: IF no requests are logged, the acceptance rate SHALL be displayed as "N/A" or "--".

**REQ-08.13**: The system SHALL store aggregated metrics in a structured format compatible with the cache system.

**REQ-08.14**: Local metrics SHALL be timestamped with the time they were aggregated, not the time of the last log entry.

### Fallback Triggering

**REQ-08.15**: WHEN the GitHub API is unavailable or returns errors THEN the system SHALL check if local log fallback is available.

**REQ-08.16**: WHEN the organization has fewer than 5 active users (API limitation) THEN the system SHALL offer local log fallback as an alternative.

**REQ-08.17**: IF local logs are available, the system SHALL display a notification: "Using local usage data (API unavailable)".

**REQ-08.18**: The UI SHALL clearly indicate when displaying local vs. API data with:
- Different color scheme or indicator
- Label stating "Local Data Only"
- Timestamp of when local data was aggregated

**REQ-08.19**: WHEN API becomes available again, the system SHALL automatically switch back to API data on next refresh.

### Configuration

**REQ-08.20**: The system SHALL provide a configuration option to enable/disable local log fallback.

**REQ-08.21**: The default setting for local log fallback SHALL be enabled.

**REQ-08.22**: The system SHALL provide a configuration option to specify custom log file paths.

**REQ-08.23**: WHEN custom log paths are configured, the system SHALL prioritize those over automatic discovery.

**REQ-08.24**: The system SHALL validate that configured custom paths exist and are readable.

### Privacy & Permissions

**REQ-08.25**: The system SHALL only read log files, never write to or modify them.

**REQ-08.26**: The system SHALL request read permissions from the user before accessing log files (if the OS requires explicit permission).

**REQ-08.27**: IF log files are not readable due to permissions, the system SHALL:
- Log the permission error
- Display a message: "Cannot read Copilot logs: Permission denied"
- Provide instructions for granting read permissions

**REQ-08.28**: The system SHALL NOT extract or store any code snippets from log files.

**REQ-08.29**: The system SHALL NOT transmit any log data over the network.

**REQ-08.30**: Aggregated metrics SHALL only include numerical counts and timestamps, no code content.

### Data Freshness

**REQ-08.31**: The system SHALL re-parse log files every 6 hours to update local metrics.

**REQ-08.32**: WHEN manually refreshing, the system SHALL re-parse logs immediately.

**REQ-08.33**: The system SHALL track which log files have been modified since last parse and only re-parse changed files.

**REQ-08.34**: IF a log file hasn't been modified in 30 days, the system SHALL exclude it from aggregation.

### UI Integration

**REQ-08.35**: The basic UI panel SHALL display a "Data Source" indicator showing either:
- "GitHub API" (when using API data)
- "Local Logs" (when using fallback)
- "Mixed" (if some local, some API data available)

**REQ-08.36**: WHEN displaying local log metrics, the UI SHALL show:
- Total completions (last 7 days)
- Acceptance rate
- Most recent activity timestamp
- Number of log files parsed

**REQ-08.37**: The detailed popup SHALL include a section explaining:
- What data source is being used
- Why fallback was activated (if applicable)
- Limitations of local log data vs. API data

**REQ-08.38**: IF no local logs are found AND API is unavailable THEN the UI SHALL display:
- "No data available"
- Explanation that both API and local logs are unavailable
- Suggestion to check configuration or wait for API

### Performance

**REQ-08.39**: Log parsing SHALL complete within 5 seconds for files up to 10MB total size.

**REQ-08.40**: The system SHALL process log files asynchronously to avoid blocking the UI.

**REQ-08.41**: IF log parsing takes longer than 10 seconds, the system SHALL display a "Processing logs..." indicator.

**REQ-08.42**: The system SHALL limit total log file processing to 50MB per parse operation.

**REQ-08.43**: IF log files exceed 50MB total, the system SHALL process only the most recent files up to the limit.

## Acceptance Criteria

### AC-08.1: Log Discovery
- [ ] Finds VS Code Copilot logs when present
- [ ] Finds JetBrains Copilot logs when present
- [ ] Finds Neovim Copilot logs when present
- [ ] Handles missing log directories gracefully
- [ ] Re-scans for new logs every 24 hours

### AC-08.2: Log Parsing
- [ ] Extracts completion requests from logs
- [ ] Extracts completion acceptances from logs
- [ ] Handles multiple log formats correctly
- [ ] Skips unparseable lines without errors
- [ ] Tracks last processed position in files

### AC-08.3: Metrics Aggregation
- [ ] Calculates total completions (7 days)
- [ ] Calculates acceptance rate correctly
- [ ] Groups activity by day
- [ ] Identifies most active editor
- [ ] Stores metrics in compatible format

### AC-08.4: Fallback Behavior
- [ ] Activates when API is unavailable
- [ ] Clearly indicates local data is being used
- [ ] Switches back to API when available
- [ ] Handles "no logs found" scenario

### AC-08.5: Configuration
- [ ] Enable/disable toggle works correctly
- [ ] Custom log paths can be configured
- [ ] Invalid paths are validated and rejected
- [ ] Default is enabled

### AC-08.6: Privacy & Security
- [ ] Only reads logs, never writes
- [ ] Handles permission errors gracefully
- [ ] Never stores code snippets
- [ ] Never transmits log data

### AC-08.7: Performance
- [ ] Parsing completes within 5s for typical logs
- [ ] Asynchronous processing doesn't block UI
- [ ] Handles large log files (50MB limit)
- [ ] Progress indicator shows for long operations

### AC-08.8: UI Integration
- [ ] Data source indicator displays correctly
- [ ] Local metrics shown in basic panel
- [ ] Detailed popup explains fallback
- [ ] "No data" state is clear and helpful

## Dependencies

- **Feature 02**: Configuration & Auth (stores fallback settings)
- **Feature 04**: Basic UI Panel (displays fallback data and indicator)
- **Feature 05**: Detailed UI Popup (explains fallback status)
- **Feature 06**: Auto-Refresh & Caching (triggers fallback parsing on schedule)
- **Feature 07**: Error Handling (triggers fallback when API fails)

## Technical Notes

### Log Format Examples

**VS Code Copilot Log Pattern**:
```
[2025-09-30 10:30:15.234] [info] Completion requested for file: main.rs
[2025-09-30 10:30:15.567] [info] Completion accepted
```

**JetBrains Pattern** (may vary):
```
2025-09-30 10:30:15.234 [INFO] github.copilot: Completion request
2025-09-30 10:30:15.567 [INFO] github.copilot: Completion applied
```

### Parsing Strategy
- Use regex patterns to identify completion events
- Extract timestamps for date grouping
- Match request/acceptance pairs when possible
- Handle missing or incomplete log entries gracefully

### Log Position Tracking
Store last parsed position per file:
```json
{
  "log_positions": {
    "/home/user/.vscode/extensions/github.copilot-1.0/logs/copilot.log": {
      "last_byte": 1048576,
      "last_modified": "2025-09-30T10:30:00Z"
    }
  }
}
```

### Metrics Storage Format
```json
{
  "source": "local_logs",
  "aggregated_at": "2025-09-30T10:30:00Z",
  "period_days": 7,
  "metrics": {
    "completions_requested": 1234,
    "completions_accepted": 892,
    "acceptance_rate": 72.3,
    "most_active_editor": "vscode",
    "log_files_parsed": 3
  },
  "daily_breakdown": [
    { "date": "2025-09-24", "requests": 156, "acceptances": 112 },
    { "date": "2025-09-25", "requests": 203, "acceptances": 145 }
  ]
}
```

### Editor Detection Priority
1. VS Code (most common)
2. JetBrains IDEs (IntelliJ, PyCharm, etc.)
3. Neovim/Vim with Copilot plugin
4. Custom configured paths

### Limitations to Communicate
- Local logs only show current machine's activity
- Cannot show organization-wide metrics
- May not capture all completion events (depends on log level)
- Different editors log different details
- Historical data limited to log retention period

### Future Considerations
- Support for more editors (Emacs, Sublime, etc.)
- Parsing of language-specific statistics from logs
- Detection of Copilot Chat usage in logs
- Integration with IDE extensions for real-time events
