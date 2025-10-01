# Manual Test Report - Feature 02: Configuration & Authentication

**Date**: 2025-09-30  
**Tester**: AI Assistant (TDD Agent)  
**Feature**: Configuration and Authentication (Feature 02)  
**Build Status**: ✅ Compiles successfully

## Test Environment
- **OS**: Linux (COSMIC Desktop)
- **Session Type**: Wayland
- **Build Profile**: Debug
- **Config Directory**: `~/.config/cosmic/copilot-quota`

## Automated Test Status
✅ **All 180 unit/integration tests passing** (verified in previous session)

## Manual Testing Status

### Note on Testing Limitations
COSMIC applets are designed to run within the COSMIC panel, not as standalone applications. Full manual testing requires:
1. Building the release version: `just build-release`
2. Installing the applet: `sudo just install`
3. Restarting the COSMIC panel or logging out/in
4. Adding the applet to the panel through COSMIC settings

For this test report, we'll document what **should** be tested once installed, based on our implementation and unit tests.

---

## Test Scenarios (To Be Executed After Installation)

### 1. First Launch Testing
**Objective**: Verify config file creation and initialization on first run

| Test Case | Expected Behavior | Status | Notes |
|-----------|------------------|--------|-------|
| Config file creation | File created at `~/.config/cosmic/copilot-quota/config.toml` | ⏳ Pending | - |
| Default values | Empty org name, 900s (15 min) interval | ⏳ Pending | - |
| File permissions (Unix) | File has 0600 permissions (owner read/write only) | ⏳ Pending | Security requirement |
| Directory creation | Parent directories created if missing | ⏳ Pending | - |

**Test Steps**:
1. Remove existing config: `rm -rf ~/.config/cosmic/copilot-quota`
2. Install and launch applet
3. Check file existence: `ls -la ~/.config/cosmic/copilot-quota/config.toml`
4. Verify permissions: `stat -c "%a" ~/.config/cosmic/copilot-quota/config.toml` (should show `600`)
5. Check contents: `cat ~/.config/cosmic/copilot-quota/config.toml`

**Expected File Contents**:
```toml
organization_name = ""
refresh_interval_seconds = 900
```

---

### 2. Settings Dialog Testing
**Objective**: Verify settings UI opens, accepts input, and displays properly

| Test Case | Expected Behavior | Status | Notes |
|-----------|------------------|--------|-------|
| Open settings | Dialog opens via applet menu/button | ⏳ Pending | - |
| Organization field | Text input for org name visible and editable | ⏳ Pending | - |
| PAT field | Secure text input (masked) visible and editable | ⏳ Pending | Should show • • • |
| Refresh interval | Numeric input with "seconds" label | ⏳ Pending | - |
| Save button | Button present and clickable | ⏳ Pending | - |
| Cancel button | Button present, closes without saving | ⏳ Pending | - |
| Error display | Error messages shown in dialog | ⏳ Pending | Red text or alert |

**Test Steps**:
1. Click applet in panel to open menu
2. Select "Settings" or "Configure" option
3. Verify all fields are present
4. Enter test values:
   - Org: "test-org"
   - PAT: "ghp_" + 40 random chars
   - Interval: "300"
5. Click Save
6. Verify dialog closes without errors

---

### 3. Settings Persistence Testing
**Objective**: Verify configuration persists across app restarts

| Test Case | Expected Behavior | Status | Notes |
|-----------|------------------|--------|-------|
| Save to disk | Values written to config.toml | ⏳ Pending | - |
| Save to keyring | PAT stored in system keyring | ⏳ Pending | Use `secret-tool` to verify |
| Load on restart | Values restored after panel restart | ⏳ Pending | - |
| PAT retrieval | PAT loaded from keyring on dialog open | ⏳ Pending | Should show masked |

**Test Steps**:
1. Configure settings (as in scenario 2)
2. Check file: `cat ~/.config/cosmic/copilot-quota/config.toml`
3. Verify org name and interval are saved (PAT should NOT be in file)
4. Check keyring: `secret-tool lookup service cosmic-copilot-quota username github-pat`
5. Restart COSMIC panel: `systemctl --user restart cosmic-panel`
6. Open settings again, verify values are restored

**Expected File Contents After Save**:
```toml
organization_name = "test-org"
refresh_interval_seconds = 300
```

---

### 4. Validation Testing
**Objective**: Verify input validation rejects invalid values

#### 4a. Organization Name Validation

| Test Case | Input | Expected Behavior | Status | Notes |
|-----------|-------|------------------|--------|-------|
| Empty org | "" | Error: "Organization name cannot be empty" | ⏳ Pending | - |
| Valid org | "my-org" | Save succeeds | ⏳ Pending | - |

**Test Steps**:
1. Open settings
2. Leave org name empty, enter valid PAT
3. Click Save
4. Verify error message appears
5. Verify dialog stays open
6. Enter valid org name, click Save again
7. Verify save succeeds

#### 4b. PAT Format Validation

| Test Case | Input | Expected Behavior | Status | Notes |
|-----------|-------|------------------|--------|-------|
| Empty PAT | "" | Error: "Invalid PAT format" | ⏳ Pending | - |
| Short ghp_ | "ghp_123" | Error: "Invalid PAT format" | ⏳ Pending | < 44 chars |
| Valid ghp_ | "ghp_" + 40 chars | Save succeeds | ⏳ Pending | Classic token |
| Valid github_pat_ | "github_pat_" + 82 chars | Save succeeds | ⏳ Pending | Fine-grained token |
| Invalid prefix | "abc_" + 40 chars | Error: "Invalid PAT format" | ⏳ Pending | Wrong prefix |

**Test Steps**:
1. Open settings, enter valid org
2. Test each invalid PAT case
3. Verify error message for each
4. Test valid PAT formats
5. Verify save succeeds

#### 4c. Refresh Interval Validation

| Test Case | Input | Expected Behavior | Status | Notes |
|-----------|-------|------------------|--------|-------|
| Below minimum | "0" | Error: "must be between 1 and 3600" | ⏳ Pending | - |
| Minimum valid | "1" | Save succeeds | ⏳ Pending | 1 second |
| Normal value | "300" | Save succeeds | ⏳ Pending | 5 minutes |
| Maximum valid | "3600" | Save succeeds | ⏳ Pending | 60 minutes |
| Above maximum | "3601" | Error: "must be between 1 and 3600" | ⏳ Pending | - |
| Non-numeric | "abc" | Error: parse error | ⏳ Pending | - |

**Test Steps**:
1. Open settings, enter valid org and PAT
2. Test each interval value
3. Verify validation behavior

---

### 5. Keyring Integration Testing
**Objective**: Verify PAT is securely stored and retrieved from system keyring

| Test Case | Expected Behavior | Status | Notes |
|-----------|------------------|--------|-------|
| PAT stored | PAT retrievable via secret-tool | ⏳ Pending | - |
| PAT updated | New PAT replaces old in keyring | ⏳ Pending | - |
| PAT not in config | PAT absent from config.toml | ⏳ Pending | Security check |
| PAT retrieval | PAT loaded when settings dialog opens | ⏳ Pending | - |

**Test Steps**:
1. Save settings with PAT: "ghp_1234567890123456789012345678901234567890"
2. Verify stored: `secret-tool lookup service cosmic-copilot-quota username github-pat`
3. Verify NOT in config: `grep -i ghp ~/.config/cosmic/copilot-quota/config.toml` (should find nothing)
4. Update PAT to: "ghp_abcdefghijklmnopqrstuvwxyz12345678901234"
5. Save again
6. Verify updated: `secret-tool lookup service cosmic-copilot-quota username github-pat`
7. Restart panel and open settings
8. Verify PAT field shows masked version of current PAT

---

## Code Changes Made During Testing

### 1. Fixed main.rs Initialization
**File**: `src/main.rs`  
**Change**: Updated to use `ConfigManager::new()` and `.load()` instead of hardcoded config  
**Reason**: Enable proper config file loading on application startup

**Before**:
```rust
let config = AppConfig {
    organization_name: "default-org".to_string(),
    refresh_interval_seconds: 900,
};
```

**After**:
```rust
let config = match ConfigManager::new() {
    Ok(config_manager) => match config_manager.load() {
        Ok(config) => {
            println!("Loaded configuration: org={}, interval={}s", ...);
            config
        }
        Err(e) => {
            eprintln!("Warning: Failed to load config: {}", e);
            AppConfig::default()
        }
    },
    Err(e) => {
        eprintln!("Warning: Failed to create config manager: {}", e);
        AppConfig::default()
    }
};
```

---

## Summary

### Current Status
- ✅ **Code compiles** successfully
- ✅ **All 180 unit tests pass**
- ✅ **Main.rs updated** to load config properly
- ⏳ **Manual testing pending** - requires applet installation

### Next Steps for Manual Testing
1. **Build release**: `just build-release`
2. **Install**: `sudo just install`
3. **Restart panel**: `systemctl --user restart cosmic-panel` or logout/login
4. **Add to panel**: COSMIC Settings → Panel → Add Applet
5. **Execute test scenarios** 1-5 above
6. **Document results** in this file
7. **Fix any issues found**
8. **Mark Task 4.1 complete** in tasks.md

### Automated Test Coverage
Our comprehensive unit tests already cover:
- ✅ Config file creation/loading/saving
- ✅ File permissions (Unix)
- ✅ Organization name validation
- ✅ PAT format validation (ghp_ and github_pat_)
- ✅ Refresh interval validation
- ✅ Keyring storage/retrieval
- ✅ Error handling
- ✅ Message handling (OpenSettings, CloseSettings, SaveConfig, etc.)

Manual testing will verify:
- 🎯 UI rendering and interaction
- 🎯 End-to-end workflows
- 🎯 System integration (keyring, file system)
- 🎯 User experience

---

## Test Execution Notes
_(To be filled in during actual manual testing)_

### Issues Found
- None yet (pending installation)

### Test Execution Log
_(Add timestamps and observations during testing)_


---

# Manual Test Report - Feature 04: Basic UI Panel & Popup

**Date**: 2025-10-01  
**Tester**: AI Assistant (TDD Agent)  
**Feature**: Basic UI Panel & Popup (Feature 04)  
**Build Status**: ✅ Compiles successfully  
**Test Status**: ✅ All 183 unit/integration tests passing

## Test Environment
- **OS**: Linux (COSMIC Desktop)
- **Session Type**: Wayland
- **Build Profile**: Release
- **Rust Version**: Latest stable

## Feature Implementation Summary

### Architecture Changes
We implemented a proper COSMIC applet popup pattern following the framework conventions:

1. **Panel View** (`view()` method):
   - Shows icon-only button (not tiny elements)
   - Uses `button::icon()` with proper icon
   - Properly handles `TogglePopup` message

2. **Popup Window** (`view_window()` method):
   - Wraps content with `popup_container` for proper sizing
   - Max width: 500px, Max height: 400px
   - Routes to either metrics view or settings view based on `settings_dialog_open` flag

3. **Metrics Popup View** (`metrics_popup_view()` method):
   - Shows loading state: "Loading..."
   - Shows error state: Error message + Retry button
   - Shows success/stale state: Metrics data + Settings button
   - Displays: Suggestions count, Acceptances count, Acceptance rate, Last updated date

4. **Message Handling**:
   - `TogglePopup`: Opens/closes metrics popup, creates window ID
   - `OpenSettings`: Switches to settings view in popup
   - `CloseSettings`: Returns to metrics view in popup

### Code Quality
- ✅ 183 unit/integration tests passing
- ✅ All warnings addressed except unused helper methods (intentionally kept)
- ✅ Follows COSMIC applet patterns
- ✅ TDD methodology applied throughout

---

## Test Scenarios (To Be Executed After Installation)

### 1. Panel Icon Display Testing
**Objective**: Verify icon displays correctly in panel (not tiny elements)

| Test Case | Expected Behavior | Status | Notes |
|-----------|------------------|--------|-------|
| Icon rendering | Icon shows at proper size in panel | ⏳ Pending | Should be ~24-32px |
| Icon state - Loading | Shows loading/spinner icon | ⏳ Pending | "content-loading-symbolic" |
| Icon state - Error | Shows error icon | ⏳ Pending | "dialog-error-symbolic" |
| Icon state - Success | Shows info icon | ⏳ Pending | "dialog-information-symbolic" |
| Icon state - Stale | Shows info icon | ⏳ Pending | "dialog-information-symbolic" |

**Test Steps**:
1. Install and add applet to COSMIC panel
2. Verify icon appears at normal size (not tiny)
3. Configure with valid org/PAT
4. Wait for loading → success transition
5. Verify icon changes appropriately

**Expected Results**:
- Icon should be same size as other panel icons
- Should NOT show three dots (•••) or tiny elements
- Should change icons based on state

---

### 2. Metrics Popup Display Testing
**Objective**: Verify metrics popup opens and displays correctly

| Test Case | Expected Behavior | Status | Notes |
|-----------|------------------|--------|-------|
| Popup opens on click | Click icon → popup appears | ⏳ Pending | - |
| Popup size | Reasonable size (≤500x400px) | ⏳ Pending | Via popup_container |
| Loading state | Shows "Loading..." message | ⏳ Pending | On first load |
| Success state | Shows metrics data | ⏳ Pending | After API call |
| Error state | Shows error + Retry button | ⏳ Pending | On API failure |
| Popup positioning | Positioned near applet icon | ⏳ Pending | COSMIC framework handles |

**Test Steps**:
1. Click applet icon in panel
2. Verify popup opens near the icon
3. Check initial state (should be Loading if unconfigured)
4. Configure valid credentials in settings
5. Wait for data to load
6. Verify metrics display correctly

**Expected Popup Content (Success State)**:
```
Suggestions: 100
Acceptances: 50
Acceptance Rate: 50.0%
Last Updated: 2025-09-30
[Settings] button
```

---

### 3. Metrics Data Display Testing
**Objective**: Verify metrics show correct values and formatting

| Test Case | Expected Behavior | Status | Notes |
|-----------|------------------|--------|-------|
| Suggestions count | Number formatted with commas | ⏳ Pending | e.g., "1,234" |
| Acceptances count | Number formatted with commas | ⏳ Pending | e.g., "567" |
| Acceptance rate | Percentage with 1 decimal | ⏳ Pending | e.g., "45.9%" |
| Date display | YYYY-MM-DD format | ⏳ Pending | e.g., "2025-09-30" |
| Large numbers | Proper formatting (1,000+) | ⏳ Pending | Test with real data |
| Zero values | Shows "0" not "--" | ⏳ Pending | Edge case |

**Test Steps**:
1. Configure with valid GitHub org that has Copilot usage
2. Wait for metrics to load
3. Verify each field displays correctly
4. Compare with GitHub API response to verify accuracy

---

### 4. Settings Button & Navigation Testing
**Objective**: Verify navigation between metrics and settings views

| Test Case | Expected Behavior | Status | Notes |
|-----------|------------------|--------|-------|
| Settings button present | Shows in metrics popup | ⏳ Pending | - |
| Settings button click | Opens settings in same popup | ⏳ Pending | Not new window |
| Settings view displays | Shows config form | ⏳ Pending | Org, PAT, interval |
| Cancel button | Returns to metrics view | ⏳ Pending | - |
| Save button | Returns to metrics view | ⏳ Pending | After successful save |
| Popup stays open | Doesn't close during navigation | ⏳ Pending | - |

**Test Steps**:
1. Open metrics popup
2. Click "Settings" button
3. Verify settings view appears in same popup
4. Click "Cancel"
5. Verify returns to metrics view
6. Click "Settings" again
7. Make changes and click "Save"
8. Verify returns to metrics view with updated data

---

### 5. Error Handling & Retry Testing
**Objective**: Verify error states display correctly and retry works

| Test Case | Expected Behavior | Status | Notes |
|-----------|------------------|--------|-------|
| Invalid credentials | Shows error in popup | ⏳ Pending | - |
| Network error | Shows error in popup | ⏳ Pending | - |
| Error message display | Clear, readable error text | ⏳ Pending | - |
| Retry button present | Shows "Retry" button | ⏳ Pending | - |
| Retry button works | Retries API call on click | ⏳ Pending | - |
| State update | Loading → Error → Success flow | ⏳ Pending | - |

**Test Steps**:
1. Configure with invalid PAT
2. Open popup
3. Verify error message displays
4. Verify Retry button is present
5. Fix credentials in settings
6. Click Retry button
7. Verify transitions to Loading then Success

---

### 6. Popup Lifecycle Testing
**Objective**: Verify popup opens/closes correctly

| Test Case | Expected Behavior | Status | Notes |
|-----------|------------------|--------|-------|
| Open popup | Click icon → popup opens | ⏳ Pending | - |
| Close popup | Click icon again → popup closes | ⏳ Pending | Toggle behavior |
| Close by clicking outside | Click elsewhere → popup closes | ⏳ Pending | COSMIC default |
| Close by ESC key | ESC key → popup closes | ⏳ Pending | COSMIC default |
| State persists | Reopen shows same view | ⏳ Pending | Metrics or settings |
| Multiple opens/closes | No memory leaks or issues | ⏳ Pending | Stress test |

**Test Steps**:
1. Click icon to open popup
2. Click icon again to close popup
3. Repeat 5-10 times
4. Open popup, click outside to close
5. Open popup, press ESC to close
6. Open popup, navigate to settings, close popup, reopen
7. Verify no crashes or UI glitches

---

## Implementation Details

### Files Modified
1. **`src/app.rs`**:
   - Added `metrics_popup_view()` method (lines 232-289)
   - Updated `view_window()` to use `popup_container` (lines 422-445)
   - Fixed `TogglePopup`, `OpenSettings`, `CloseSettings` handlers
   - Added 3 comprehensive tests for popup view states

### Key Design Decisions
1. **Single Window ID**: Both metrics and settings use the same `popup` window ID
2. **View Routing**: `settings_dialog_open` boolean determines which view to show
3. **Proper Sizing**: `popup_container` with max dimensions (500x400)
4. **Icon-Only Panel**: Minimal panel view, detailed popup view

### Test Coverage
- ✅ `test_metrics_popup_view_shows_loading_state`
- ✅ `test_metrics_popup_view_shows_success_state`
- ✅ `test_metrics_popup_view_shows_error_state`
- ✅ All previous tests still passing (183 total)

---

## Installation & Testing Instructions

### Build & Install
```bash
# Build release version
cargo build --release

# Find the built binary
ls -lh target/release/cosmic-applet-template

# Copy to applets directory (may need sudo)
sudo mkdir -p /usr/local/share/cosmic/applets
sudo cp target/release/cosmic-applet-template /usr/local/share/cosmic/applets/copilot-quota-tracker

# Copy desktop file
sudo cp res/com.example.CosmicAppletTemplate.desktop /usr/share/applications/

# Restart COSMIC panel
systemctl --user restart cosmic-panel
# OR logout/login
```

### Add to Panel
1. Open COSMIC Settings
2. Navigate to "Desktop" → "Panel"
3. Click "Add Applet"
4. Find "Copilot Quota Tracker" in the list
5. Click to add it to the panel

### Configure & Test
1. Click the applet icon in panel
2. Click "Settings" button in popup
3. Enter GitHub organization name
4. Enter GitHub Personal Access Token (ghp_...)
5. Set refresh interval (e.g., 300 seconds)
6. Click "Save"
7. Wait for metrics to load
8. Verify metrics display correctly
9. Test all scenarios above

---

## Known Issues & Limitations

### Minor Warnings
- ⚠️ Unused methods `get_metric_text()` and `get_tooltip_text()` in `app.rs`
  - **Reason**: Previously used for text display in panel, now using icon-only
  - **Impact**: None (tests pass, functionality works)
  - **Decision**: Keeping them for potential future tooltip functionality

### Testing Limitations
- Manual testing requires actual COSMIC desktop environment
- API calls require valid GitHub organization with Copilot enabled
- Some edge cases (network errors) require special test setups

---

## Summary

### ✅ Completed Tasks
- [x] Task 1: Write tests for metrics popup view (Red phase)
- [x] Task 2: Implement metrics popup in view_window() (Green phase)
- [x] Task 3: Separate popup states for metrics vs settings
- [x] Task 4: Use popup_container for proper sizing
- [x] Task 5: Manual testing documentation prepared

### ⏳ Pending Verification
- [ ] Physical testing in COSMIC panel (requires installation)
- [ ] All test scenarios execution
- [ ] Real API integration testing

### Next Steps
1. **User Action Required**: Install and test in COSMIC panel
2. Execute all test scenarios in this document
3. Document results and any issues found
4. Mark Feature 04 as complete in tasks.md

---

## Test Execution Notes
_(To be filled in during actual manual testing)_

### Issues Found
- None yet (pending installation)

### Test Execution Log
_(Add timestamps and observations during testing)_



