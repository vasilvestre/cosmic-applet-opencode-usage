# Implementation Tasks: Multiple Panel Metrics Selection

## Overview
Implementation tasks following Test-Driven Development (TDD) methodology. Each task follows the Red-Green-Refactor cycle.

---

## Phase 1: Core Data Model Changes

### Task 1.1: Remove PanelMetric::None variant
**Status**: Pending  
**Priority**: High  
**Estimated Time**: 10 minutes

**Acceptance Criteria**:
- [ ] `PanelMetric::None` variant is removed from enum
- [ ] All test compilation errors are identified (RED phase)
- [ ] Tests that don't make sense with new model are removed
- [ ] All tests compile and pass (GREEN phase)

**Files to Modify**:
- `src/core/config.rs`: Remove `None` variant
- All test files: Update or remove tests using `None`

**TDD Steps**:
1. RED: Remove `PanelMetric::None` variant → compilation errors
2. GREEN: Fix all compilation errors
3. REFACTOR: Clean up any redundant code

---

### Task 1.2: Change panel_metric to panel_metrics (Vec)
**Status**: Pending  
**Priority**: High  
**Estimated Time**: 20 minutes

**Acceptance Criteria**:
- [ ] `AppConfig.panel_metric` changed to `panel_metrics: Vec<PanelMetric>`
- [ ] Default implementation returns all 5 metrics
- [ ] Serialization/deserialization tests pass
- [ ] Config save/load with Vec works correctly

**Files to Modify**:
- `src/core/config.rs`: Change struct field and Default impl
- `src/core/config.rs` (tests): Update test expectations

**TDD Steps**:
1. RED: Write test for `default().panel_metrics` expecting all 5 metrics
2. RED: Write test for Vec serialization roundtrip
3. GREEN: Change `panel_metric: PanelMetric` → `panel_metrics: Vec<PanelMetric>`
4. GREEN: Update `Default::default()` to return all metrics
5. GREEN: Update load/save methods to use "panel_metrics" key
6. REFACTOR: Clean up any duplicate code

**Test Cases**:
```rust
#[test]
fn test_default_panel_metrics_all_enabled() {
    let config = AppConfig::default();
    assert_eq!(config.panel_metrics.len(), 5);
    assert!(config.panel_metrics.contains(&PanelMetric::Cost));
    assert!(config.panel_metrics.contains(&PanelMetric::Interactions));
    assert!(config.panel_metrics.contains(&PanelMetric::InputTokens));
    assert!(config.panel_metrics.contains(&PanelMetric::OutputTokens));
    assert!(config.panel_metrics.contains(&PanelMetric::ReasoningTokens));
}

#[test]
fn test_panel_metrics_vec_roundtrip() {
    let app_id = test_app_id("vec_roundtrip");
    
    let config = AppConfig {
        panel_metrics: vec![
            PanelMetric::Cost,
            PanelMetric::Interactions,
        ],
        // ... other fields
    };
    
    config.save_with_id(&app_id).expect("save");
    let loaded = AppConfig::load_with_id(&app_id).expect("load");
    
    assert_eq!(loaded.panel_metrics.len(), 2);
    assert!(loaded.panel_metrics.contains(&PanelMetric::Cost));
    assert!(loaded.panel_metrics.contains(&PanelMetric::Interactions));
}

#[test]
fn test_empty_panel_metrics() {
    let app_id = test_app_id("empty_metrics");
    
    let config = AppConfig {
        panel_metrics: vec![],
        // ... other fields
    };
    
    config.save_with_id(&app_id).expect("save");
    let loaded = AppConfig::load_with_id(&app_id).expect("load");
    
    assert_eq!(loaded.panel_metrics.len(), 0);
}
```

---

## Phase 2: Formatter Implementation

### Task 2.1: Create METRIC_DISPLAY_ORDER constant
**Status**: Pending  
**Priority**: Medium  
**Estimated Time**: 5 minutes

**Acceptance Criteria**:
- [ ] `METRIC_DISPLAY_ORDER` constant defined in `formatters.rs`
- [ ] Order is: Cost, Interactions, InputTokens, OutputTokens, ReasoningTokens

**Files to Modify**:
- `src/ui/formatters.rs`

**TDD Steps**:
1. RED: Write test that uses the constant
2. GREEN: Define the constant
3. REFACTOR: N/A (simple constant)

---

### Task 2.2: Implement format_multiple_panel_metrics()
**Status**: Pending  
**Priority**: High  
**Estimated Time**: 30 minutes

**Acceptance Criteria**:
- [ ] Function signature: `fn format_multiple_panel_metrics(usage: &UsageMetrics, metrics: &[PanelMetric], use_raw: bool) -> String`
- [ ] Empty metrics returns empty string
- [ ] Single metric works correctly
- [ ] Multiple metrics joined with " | "
- [ ] Metrics displayed in fixed order
- [ ] Token metrics use new prefixes (IT:, OT:, RT:)
- [ ] Cost and Interactions unchanged
- [ ] N/A handling works for missing data

**Files to Modify**:
- `src/ui/formatters.rs`

**TDD Steps**:
1. RED: Write test for empty metrics → ""
2. RED: Write test for single metric (Cost) → "$1.23"
3. RED: Write test for two metrics (Cost + Interactions) → "$1.23 | 5x"
4. RED: Write test for token metric with prefix → "IT: 10k"
5. RED: Write test for all metrics → "$1.23 | 5x | IT: 10k | OT: 5k | RT: 2k"
6. RED: Write test for ordering (Interactions + Cost selected) → "$1.23 | 5x" (not "5x | $1.23")
7. RED: Write test for N/A handling
8. GREEN: Implement function
9. REFACTOR: Extract helper functions if needed

**Test Cases**:
```rust
#[test]
fn test_format_multiple_empty() {
    let usage = create_test_usage(1.23, 5, 10000, 5000, 2000);
    let result = format_multiple_panel_metrics(&usage, &[], false);
    assert_eq!(result, "");
}

#[test]
fn test_format_multiple_single_cost() {
    let usage = create_test_usage(1.23, 5, 10000, 5000, 2000);
    let result = format_multiple_panel_metrics(
        &usage, 
        &[PanelMetric::Cost], 
        false
    );
    assert_eq!(result, "$1.23");
}

#[test]
fn test_format_multiple_cost_and_interactions() {
    let usage = create_test_usage(1.23, 5, 10000, 5000, 2000);
    let result = format_multiple_panel_metrics(
        &usage,
        &[PanelMetric::Cost, PanelMetric::Interactions],
        false
    );
    assert_eq!(result, "$1.23 | 5x");
}

#[test]
fn test_format_multiple_token_prefixes() {
    let usage = create_test_usage(0.0, 0, 10000, 5000, 2000);
    let result = format_multiple_panel_metrics(
        &usage,
        &[PanelMetric::InputTokens, PanelMetric::OutputTokens, PanelMetric::ReasoningTokens],
        false
    );
    assert_eq!(result, "IT: 10k | OT: 5k | RT: 2k");
}

#[test]
fn test_format_multiple_all_metrics() {
    let usage = create_test_usage(1.23, 5, 10000, 5000, 2000);
    let result = format_multiple_panel_metrics(
        &usage,
        &[
            PanelMetric::Cost,
            PanelMetric::Interactions,
            PanelMetric::InputTokens,
            PanelMetric::OutputTokens,
            PanelMetric::ReasoningTokens,
        ],
        false
    );
    assert_eq!(result, "$1.23 | 5x | IT: 10k | OT: 5k | RT: 2k");
}

#[test]
fn test_format_multiple_preserves_order() {
    let usage = create_test_usage(1.23, 5, 10000, 5000, 2000);
    // Pass in reverse order, should still display in fixed order
    let result = format_multiple_panel_metrics(
        &usage,
        &[PanelMetric::Interactions, PanelMetric::Cost],
        false
    );
    assert_eq!(result, "$1.23 | 5x"); // Cost before Interactions
}

#[test]
fn test_format_multiple_with_raw_tokens() {
    let usage = create_test_usage(0.0, 0, 10543, 5234, 2876);
    let result = format_multiple_panel_metrics(
        &usage,
        &[PanelMetric::InputTokens, PanelMetric::OutputTokens],
        true // use_raw = true
    );
    assert_eq!(result, "IT: 10543 | OT: 5234");
}

#[test]
fn test_format_multiple_with_missing_data() {
    let usage = UsageMetrics {
        total_cost: None,
        interaction_count: Some(5),
        total_input_tokens: Some(10000),
        total_output_tokens: None,
        total_reasoning_tokens: Some(2000),
    };
    let result = format_multiple_panel_metrics(
        &usage,
        &[
            PanelMetric::Cost,
            PanelMetric::Interactions,
            PanelMetric::OutputTokens,
        ],
        false
    );
    assert_eq!(result, "N/A | 5x | N/A");
}
```

---

## Phase 3: App State & Message Handling

### Task 3.1: Update CosmicAppletOpencodeUsage struct
**Status**: Pending  
**Priority**: High  
**Estimated Time**: 15 minutes

**Acceptance Criteria**:
- [ ] `temp_panel_metric` renamed to `temp_panel_metrics: Vec<PanelMetric>`
- [ ] Initialization updated in `new()` and `default()`
- [ ] All compilation errors fixed

**Files to Modify**:
- `src/app.rs`

**TDD Steps**:
1. RED: Change field type → compilation errors
2. GREEN: Fix all initialization points
3. REFACTOR: N/A

---

### Task 3.2: Add TogglePanelMetric message handler
**Status**: Pending  
**Priority**: High  
**Estimated Time**: 20 minutes

**Acceptance Criteria**:
- [ ] `Message::TogglePanelMetric(PanelMetric)` added
- [ ] Handler adds metric if not present, removes if present
- [ ] No duplicates allowed
- [ ] Tests verify toggle behavior

**Files to Modify**:
- `src/app.rs`: Message enum and handler

**TDD Steps**:
1. RED: Write test for adding metric to empty vec
2. RED: Write test for removing metric from vec
3. RED: Write test for toggling same metric twice (add then remove)
4. RED: Write test for no duplicates
5. GREEN: Implement message and handler
6. REFACTOR: Extract toggle logic if complex

**Test Cases**:
```rust
#[test]
fn test_toggle_panel_metric_adds_to_empty() {
    let mut applet = create_test_applet();
    applet.temp_panel_metrics = vec![];
    
    applet.handle_message(Message::TogglePanelMetric(PanelMetric::Cost));
    
    assert_eq!(applet.temp_panel_metrics.len(), 1);
    assert!(applet.temp_panel_metrics.contains(&PanelMetric::Cost));
}

#[test]
fn test_toggle_panel_metric_removes_existing() {
    let mut applet = create_test_applet();
    applet.temp_panel_metrics = vec![PanelMetric::Cost, PanelMetric::Interactions];
    
    applet.handle_message(Message::TogglePanelMetric(PanelMetric::Cost));
    
    assert_eq!(applet.temp_panel_metrics.len(), 1);
    assert!(!applet.temp_panel_metrics.contains(&PanelMetric::Cost));
    assert!(applet.temp_panel_metrics.contains(&PanelMetric::Interactions));
}

#[test]
fn test_toggle_panel_metric_twice() {
    let mut applet = create_test_applet();
    applet.temp_panel_metrics = vec![];
    
    applet.handle_message(Message::TogglePanelMetric(PanelMetric::Cost));
    applet.handle_message(Message::TogglePanelMetric(PanelMetric::Cost));
    
    assert_eq!(applet.temp_panel_metrics.len(), 0);
}
```

---

### Task 3.3: Add ResetPanelMetricsToDefaults message handler
**Status**: Pending  
**Priority**: Medium  
**Estimated Time**: 10 minutes

**Acceptance Criteria**:
- [ ] `Message::ResetPanelMetricsToDefaults` added
- [ ] Handler sets `temp_panel_metrics` to all 5 metrics
- [ ] Test verifies reset behavior

**Files to Modify**:
- `src/app.rs`

**TDD Steps**:
1. RED: Write test for reset from empty
2. RED: Write test for reset from partial selection
3. GREEN: Implement message handler
4. REFACTOR: N/A

**Test Case**:
```rust
#[test]
fn test_reset_panel_metrics_to_defaults() {
    let mut applet = create_test_applet();
    applet.temp_panel_metrics = vec![PanelMetric::Cost];
    
    applet.handle_message(Message::ResetPanelMetricsToDefaults);
    
    assert_eq!(applet.temp_panel_metrics.len(), 5);
    assert!(applet.temp_panel_metrics.contains(&PanelMetric::Cost));
    assert!(applet.temp_panel_metrics.contains(&PanelMetric::Interactions));
    assert!(applet.temp_panel_metrics.contains(&PanelMetric::InputTokens));
    assert!(applet.temp_panel_metrics.contains(&PanelMetric::OutputTokens));
    assert!(applet.temp_panel_metrics.contains(&PanelMetric::ReasoningTokens));
}
```

---

## Phase 4: UI Integration

### Task 4.1: Update settings dialog to use checkboxes
**Status**: Pending  
**Priority**: High  
**Estimated Time**: 20 minutes

**Acceptance Criteria**:
- [ ] Radio buttons replaced with checkboxes
- [ ] Each checkbox uses `TogglePanelMetric` message
- [ ] Checkbox state reflects `temp_panel_metrics.contains(&metric)`
- [ ] "Reset to Defaults" button added

**Files to Modify**:
- `src/app.rs`: `settings_view()` method

**TDD Steps**:
1. No RED phase (UI rendering not easily testable)
2. GREEN: Update view code to use checkboxes
3. GREEN: Add reset button
4. REFACTOR: Extract repeated checkbox code if needed
5. Manual testing required

---

### Task 4.2: Update panel_button_content()
**Status**: Pending  
**Priority**: High  
**Estimated Time**: 15 minutes

**Acceptance Criteria**:
- [ ] Uses `format_multiple_panel_metrics()` instead of `format_panel_metric()`
- [ ] Checks `!panel_metrics.is_empty()` instead of `!= None`
- [ ] Displays icon only when metrics is empty

**Files to Modify**:
- `src/app.rs`: `panel_button_content()` method

**TDD Steps**:
1. RED: Write test for empty metrics → icon only
2. RED: Write test for single metric → icon + text
3. RED: Write test for multiple metrics → icon + combined text
4. GREEN: Update implementation
5. REFACTOR: Clean up conditionals

**Test Cases**:
```rust
#[test]
fn test_panel_button_empty_metrics_shows_icon_only() {
    let mut applet = create_test_applet();
    applet.state.config.panel_metrics = vec![];
    applet.state.set_today_usage(create_test_usage(1.23, 5, 10000, 5000, 2000));
    
    let content = applet.panel_button_content();
    // Verify icon is present but no text (implementation-specific assertion)
}

#[test]
fn test_panel_button_multiple_metrics_shows_combined() {
    let mut applet = create_test_applet();
    applet.state.config.panel_metrics = vec![
        PanelMetric::Cost,
        PanelMetric::Interactions,
    ];
    applet.state.set_today_usage(create_test_usage(1.23, 5, 10000, 5000, 2000));
    
    let content = applet.panel_button_content();
    // Verify icon + "$1.23 | 5x" text is present
}
```

---

### Task 4.3: Update data fetching conditions
**Status**: Pending  
**Priority**: Medium  
**Estimated Time**: 10 minutes

**Acceptance Criteria**:
- [ ] Replace `panel_metric != None` with `!panel_metrics.is_empty()`
- [ ] Replace `panel_metric == None` with `panel_metrics.is_empty()`
- [ ] Today's data fetched only when metrics are selected

**Files to Modify**:
- `src/app.rs`: Async data fetching logic

**TDD Steps**:
1. RED: Identify all fetching conditions
2. GREEN: Update conditions
3. REFACTOR: N/A

---

## Phase 5: Test Updates

### Task 5.1: Update all existing tests
**Status**: Pending  
**Priority**: High  
**Estimated Time**: 30 minutes

**Acceptance Criteria**:
- [ ] All tests using `panel_metric` updated to `panel_metrics`
- [ ] All tests using `PanelMetric::None` updated or removed
- [ ] All tests expecting single value updated to Vec
- [ ] All tests passing

**Files to Modify**:
- `src/app.rs` (tests)
- `src/core/config.rs` (tests)
- Any other test files

**TDD Steps**:
1. RED: Run tests → identify failures
2. GREEN: Fix each test systematically
3. REFACTOR: Remove redundant tests

---

## Summary

**Total Estimated Time**: ~3 hours

**Implementation Order**:
1. Phase 1 (Data Model) - Foundation
2. Phase 2 (Formatter) - Logic
3. Phase 3 (Messages) - Behavior
4. Phase 4 (UI) - Integration
5. Phase 5 (Tests) - Validation

**Risk Areas**:
- Many tests to update (Phase 5)
- UI changes require manual testing
- Need to ensure no `None` references remain

**Success Criteria**:
- All tests pass (214+ tests)
- Checkboxes work in settings dialog
- Multiple metrics display correctly in panel
- Reset to defaults works
- Config persists correctly
