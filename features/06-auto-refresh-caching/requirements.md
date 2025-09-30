# Feature 06: Auto-Refresh & Caching

## Overview
Implement automatic data refresh mechanism with intelligent caching to minimize API calls while keeping metrics up-to-date, respecting GitHub API rate limits.

## Requirements

### Data Refresh Behavior

**REQ-06.1**: The system SHALL fetch fresh metrics from the GitHub API on initial load.

**REQ-06.2**: AFTER initial load, the system SHALL automatically refresh metrics every 6 hours.

**REQ-06.3**: WHEN the user manually clicks a "Refresh" action THEN the system SHALL immediately fetch fresh metrics regardless of cache age.

**REQ-06.4**: WHILE a refresh operation is in progress, the system SHALL display a loading indicator in the UI.

**REQ-06.5**: WHEN a refresh completes successfully THEN the system SHALL update the displayed metrics and store the timestamp of the last refresh.

### Caching Strategy

**REQ-06.6**: The system SHALL cache fetched metrics data locally on the filesystem.

**REQ-06.7**: The cache file SHALL be stored in `~/.config/copilot-usage-monitor/cache.json`.

**REQ-06.8**: The cache SHALL include:
- Raw API response data
- Timestamp of when data was fetched
- Organization name the data corresponds to

**REQ-06.9**: WHEN the applet starts, IF cached data exists and is less than 6 hours old THEN the system SHALL load data from cache instead of making an API call.

**REQ-06.10**: WHEN the applet starts, IF cached data is more than 6 hours old THEN the system SHALL fetch fresh data from the API.

**REQ-06.11**: WHEN the applet starts, IF no cached data exists THEN the system SHALL fetch data from the API.

### Rate Limit Awareness

**REQ-06.12**: The system SHALL track the number of API requests made within a rolling 1-hour window.

**REQ-06.13**: IF the rate limit information is available in API response headers THEN the system SHALL parse and store:
- `X-RateLimit-Limit` (total requests allowed per hour)
- `X-RateLimit-Remaining` (requests remaining)
- `X-RateLimit-Reset` (timestamp when limit resets)

**REQ-06.14**: WHEN rate limit headers indicate fewer than 10 requests remaining THEN the system SHALL display a warning in the UI.

**REQ-06.15**: WHEN rate limit is exhausted (0 remaining) THEN the system SHALL:
- Prevent manual refresh attempts
- Display an error message with reset time
- Continue showing cached data if available

### Background Refresh

**REQ-06.16**: The system SHALL implement a background timer that triggers refresh checks every 6 hours.

**REQ-06.17**: WHEN a background refresh is triggered, the system SHALL only make an API call if the cache is stale (older than 6 hours).

**REQ-06.18**: IF a background refresh fails, the system SHALL retry up to 3 times with exponential backoff (5 minutes, 15 minutes, 45 minutes).

**REQ-06.19**: IF all retry attempts fail, the system SHALL continue displaying cached data and schedule the next refresh attempt for 6 hours later.

### Cache Management

**REQ-06.20**: The system SHALL validate the structure of cached data before loading it.

**REQ-06.21**: IF cached data is corrupted or has invalid structure THEN the system SHALL:
- Delete the invalid cache file
- Fetch fresh data from the API
- Log the corruption event

**REQ-06.22**: The cache file SHALL use atomic write operations to prevent corruption during save operations.

**REQ-06.23**: The system SHALL create the cache directory (`~/.config/copilot-usage-monitor/`) if it doesn't exist.

## Acceptance Criteria

### AC-06.1: Initial Load Behavior
- [ ] Fresh API call is made when no cache exists
- [ ] Cached data is used when cache is less than 6 hours old
- [ ] Fresh API call is made when cache is more than 6 hours old

### AC-06.2: Manual Refresh
- [ ] User can trigger manual refresh from UI
- [ ] Loading indicator appears during refresh
- [ ] UI updates with fresh data after successful refresh
- [ ] Last refresh timestamp is displayed and updated

### AC-06.3: Automatic Background Refresh
- [ ] Background timer triggers every 6 hours
- [ ] Only makes API call if cache is stale
- [ ] Implements retry logic on failure
- [ ] Continues using cached data if refresh fails

### AC-06.4: Rate Limit Handling
- [ ] Parses rate limit headers from API responses
- [ ] Displays warning when fewer than 10 requests remain
- [ ] Prevents refresh when rate limit exhausted
- [ ] Shows time until rate limit reset

### AC-06.5: Cache Persistence
- [ ] Cache file is created at correct location
- [ ] Cache includes all required metadata (timestamp, org name)
- [ ] Atomic writes prevent corruption
- [ ] Invalid/corrupted cache is detected and discarded

### AC-06.6: Performance
- [ ] Cache load is faster than 100ms
- [ ] Cache write completes within 500ms
- [ ] Background refresh doesn't block UI

## Dependencies

- **Feature 03**: GitHub API Client (provides API interaction layer)
- **Feature 04**: Basic UI Panel (displays loading states and last refresh time)
- **Feature 07**: Error Handling (handles refresh failures)

## Technical Notes

### Cache File Format (JSON)
```json
{
  "organization": "my-org",
  "fetched_at": "2025-09-30T10:30:00Z",
  "rate_limit": {
    "limit": 5000,
    "remaining": 4998,
    "reset": 1727697600
  },
  "data": {
    // Raw GitHub API response
  }
}
```

### Refresh Timer Implementation
- Use COSMIC's event loop for background timers
- Consider using `tokio::time::interval` for async timing
- Ensure timer cleanup on applet shutdown

### Data Freshness Strategy
Given that GitHub processes metrics once per day (data up to yesterday only):
- 6-hour refresh interval is reasonable (4 checks per day)
- Most checks will see no new data (expected behavior)
- Users can manually refresh if they want to check immediately

### Rate Limit Considerations
- GitHub API typically allows 5,000 requests/hour for authenticated requests
- With 4 automatic refreshes/day, we're well under limits
- Manual refreshes are the primary concern
- Rate limit tracking protects against aggressive manual refresh usage
