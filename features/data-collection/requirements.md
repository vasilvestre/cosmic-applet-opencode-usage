# Data Collection Requirements

## Overview
The data collection feature enables automatic, periodic storage of OpenCode usage metrics for historical tracking and analysis.

## Functional Requirements

### Data Capture
1. The system SHALL capture usage metrics snapshots containing:
   - Date (YYYY-MM-DD)
   - Total tokens used
   - Input tokens used
   - Output tokens used
   - Cache creation tokens
   - Cache read tokens
   - Total cost in USD

### Automatic Collection
2. WHEN the applet starts THEN the system SHALL initialize the data collector
3. WHEN usage metrics are fetched from OpenCode THEN the system SHALL check if a daily snapshot should be saved
4. WHEN the current date differs from the last collection date THEN the system SHALL save a new snapshot
5. The system SHALL collect usage data once per day maximum
6. WHEN the system date changes to a new day THEN the system SHALL trigger a new snapshot on the next metrics fetch

### Data Retrieval
7. The system SHALL provide retrieval of a specific date's snapshot
8. The system SHALL provide retrieval of snapshots within a date range
9. The system SHALL provide retrieval of the most recent snapshot
10. WHEN no data exists for a requested date THEN the system SHALL return None without error

### Data Persistence
11. The system SHALL store snapshots in the SQLite database
12. The system SHALL prevent duplicate snapshots for the same date
13. WHEN a snapshot already exists for the current date THEN the system SHALL skip saving
14. The system SHALL handle database errors gracefully without crashing the applet

### Data Retention
15. The system SHALL provide cleanup of snapshots older than a specified number of days
16. WHEN cleanup is invoked THEN the system SHALL delete all snapshots older than the retention period
17. The system SHALL preserve data integrity during cleanup operations

### Integration
18. The system SHALL integrate with the existing applet without requiring user intervention
19. The system SHALL operate transparently in the background
20. WHEN the applet is restarted multiple times per day THEN the system SHALL not create duplicate snapshots
21. The system SHALL track the last collection timestamp to prevent duplicates

## Non-Functional Requirements

### Performance
22. The system SHALL complete snapshot save operations within 100ms
23. The system SHALL complete snapshot retrieval operations within 50ms
24. The system SHALL not block the UI thread during database operations

### Reliability
25. WHEN database operations fail THEN the system SHALL log errors and continue applet operation
26. The system SHALL maintain data consistency across applet restarts
27. The system SHALL handle concurrent database access safely

### Maintainability
28. The system SHALL separate repository logic from collection logic
29. The system SHALL provide clear error messages for debugging
30. The system SHALL follow existing code style and conventions

## Test Requirements
31. The system SHALL have unit tests for all repository operations
32. The system SHALL have integration tests for the collector
33. The system SHALL have tests verifying no duplicate daily saves
34. The system SHALL have tests for date change detection
35. The system SHALL have tests for error handling scenarios
