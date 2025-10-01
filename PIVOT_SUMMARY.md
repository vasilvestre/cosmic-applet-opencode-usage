# Project Pivot Summary: OpenCode Usage Tracker

**Date**: October 1, 2025  
**Status**: Planning Complete - Ready for Implementation

## Overview

Successfully pivoted the COSMIC applet project from tracking GitHub Copilot usage to tracking OpenCode/Claude Code usage from local storage files.

## Changes Made

### 1. Features Reorganization
- **Archived**: Moved GitHub-specific features to `.archived/`
  - 03-github-api-client
  - 05-detailed-ui-popup
  - 06-auto-refresh-caching
  - 07-error-handling
  - 08-local-log-fallback

- **Retained**: Core infrastructure features
  - 01-project-setup
  - 02-configuration-auth (to be updated)
  - 04-basic-ui-panel (to be updated)

- **Created**: New OpenCode feature
  - 03-opencode-usage-reader (complete with requirements, design, and tasks)

### 2. Project Metadata Updates

#### Cargo.toml
- Renamed package: `cosmic-applet-opencode-usage`
- Updated lib/bin names
- Added description: "COSMIC Desktop applet for tracking OpenCode/Claude Code usage metrics"
- **Dependencies changed**:
  - Removed: `reqwest` (no HTTP calls needed)
  - Removed: `keyring` (no API tokens needed)
  - Added: `walkdir` (for directory traversal)
  - Updated: `tokio` with `fs` feature

#### README.md
- New title: "COSMIC OpenCode Usage Tracker"
- Updated description to focus on OpenCode usage tracking
- Added OpenCode explanation and link
- Updated requirements section
- Clarified data source (local storage files)

### 3. New Feature Specification

Created comprehensive specification for Feature 03:

**Requirements** (`requirements.md`):
- Storage access and discovery
- JSON parsing and data extraction
- Metrics aggregation
- Caching strategy
- Error handling
- Performance targets

**Design** (`design.md`):
- Three-component architecture:
  - Storage Scanner: File discovery with `walkdir`
  - Usage Parser: JSON parsing with `serde`
  - Aggregator: Metrics calculation
- Data structures defined
- Caching strategy (5-minute in-memory cache)
- Error handling approach
- Testing strategy outlined

**Tasks** (`tasks.md`):
- 12 implementation tasks defined
- TDD methodology with test cases for each task
- Clear acceptance criteria
- Development order specified

## Data Model

The applet will parse OpenCode storage files with this structure:

```json
{
  "id": "prt_...",
  "messageID": "msg_...",
  "sessionID": "ses_...",
  "type": "step-finish",
  "tokens": {
    "input": 26535,
    "output": 1322,
    "reasoning": 0,
    "cache": {
      "write": 0,
      "read": 24781
    }
  },
  "cost": 0
}
```

Aggregated metrics will include:
- Total input/output/reasoning tokens
- Cache read/write tokens
- Total cost
- Interaction count
- Last update timestamp

## Data Source

- **Location**: `~/.local/share/opencode/storage/part/`
- **Format**: Individual JSON files in nested directory structure
- **Estimated scale**: 10k-100k files for typical usage
- **File organization**: Grouped by message ID subdirectories

## Next Steps

### Ready for Implementation

Follow the task order in `features/03-opencode-usage-reader/tasks.md`:

1. **Task 1**: Setup module structure (`src/core/opencode/`)
2. **Tasks 2-6**: Implement core components using TDD
3. **Tasks 7-8**: Integration testing and error handling
4. **Tasks 9-10**: Clean up GitHub code and config
5. **Tasks 11-12**: Update UI and localization

### Implementation Approach

Choose one of:
- **TDD Mode**: Write tests first, implement to pass tests
- **Standard**: Implement features with tests
- **Collaborative**: Step-by-step guidance
- **Self-implementation**: Use the specs as reference

## Testing Strategy

- Unit tests for each component (parser, aggregator, scanner)
- Integration tests with sample data
- Manual testing with real OpenCode storage
- Target: >80% code coverage on core logic

## Estimated Effort

- Core implementation (Tasks 1-8): ~6-8 hours
- Cleanup and UI updates (Tasks 9-12): ~2-3 hours
- Testing and debugging: ~2-3 hours
- **Total**: ~10-14 hours

## Notes

- All GitHub API code will be removed (no HTTP dependencies needed)
- Simpler architecture: local file reading instead of API calls
- Better performance: no network latency
- Privacy-friendly: all data stays local
- No authentication needed: just filesystem access

## Dependencies

The project now has minimal external dependencies:
- `walkdir`: Directory traversal
- `serde_json`: JSON parsing
- `tokio`: Async file I/O
- `chrono`: Timestamps
- `thiserror`: Error handling
- `libcosmic`: COSMIC desktop integration

---

**Ready to begin implementation!** 🚀

Start with Task 1 when ready to proceed.
