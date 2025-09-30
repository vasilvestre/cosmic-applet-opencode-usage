# Tasks: Basic UI Panel Display

**Feature:** 04 - Basic UI Panel Display  
**Project:** COSMIC Copilot Usage Monitor  
**Methodology:** Test-Driven Development (TDD)  
**Date:** 2025-09-30

---

## TDD Workflow

For each task, follow the **Red-Green-Refactor** cycle:

1. **RED**: Write a failing test first
2. **GREEN**: Write the minimum code to make the test pass
3. **REFACTOR**: Improve code structure while keeping tests green

---

## Task 1: Number Formatting Utility

**Priority:** High  
**Complexity:** Low  
**Dependencies:** None

### Test Scenarios (RED)
```rust
#[test]
fn test_format_number_under_thousand() {
    assert_eq!(format_number(0), "0");
    assert_eq!(format_number(42), "42");
    assert_eq!(format_number(999), "999");
}

#[test]
fn test_format_number_thousands() {
    assert_eq!(format_number(1_000), "1.0K");
    assert_eq!(format_number(1_234), "1.2K");
    assert_eq!(format_number(999_999), "1000.0K");
}

#[test]
fn test_format_number_millions() {
    assert_eq!(format_number(1_000_000), "1.0M");
    assert_eq!(format_number(1_234_567), "1.2M");
    assert_eq!(format_number(10_000_000), "10.0M");
}
```

### Implementation (GREEN)
- Create `src/ui/formatters.rs` module
- Implement `format_number(n: u64) -> String` function
- Handle edge cases (0, very large numbers)

### Refactoring Opportunities
- Consider adding formatting options (decimal places, units)
- Extract magic numbers to constants
- Add documentation

### Acceptance Criteria
- [ ] All test scenarios pass
- [ ] Numbers under 1,000 shown as-is
- [ ] Numbers 1K-999K formatted with one decimal place and "K"
- [ ] Numbers 1M+ formatted with one decimal place and "M"
- [ ] No unwanted precision (e.g., "1000.0K" should be "1.0M")

---

## Task 2: Tooltip Text Generation

**Priority:** High  
**Complexity:** Low  
**Dependencies:** None

### Test Scenarios (RED)
```rust
#[test]
fn test_format_tooltip_with_timestamp() {
    let dt = Utc.with_ymd_and_hms(2025, 9, 30, 14, 30, 0).unwrap();
    assert_eq!(
        format_tooltip(Some(dt)),
        "Last updated: 2025-09-30 14:30:00"
    );
}

#[test]
fn test_format_tooltip_no_data() {
    assert_eq!(format_tooltip(None), "No data available");
}

#[test]
fn test_format_tooltip_recent_time() {
    let now = Utc::now();
    let result = format_tooltip(Some(now));
    assert!(result.starts_with("Last updated:"));
}
```

### Implementation (GREEN)
- Add to `src/ui/formatters.rs` module
- Implement `format_tooltip(last_update: Option<DateTime<Utc>>) -> String`
- Use consistent datetime format

### Refactoring Opportunities
- Consider relative time formatting ("5 minutes ago")
- Extract datetime format string to constant
- Add timezone handling

### Acceptance Criteria
- [ ] All test scenarios pass
- [ ] Tooltip with timestamp shows "Last updated: YYYY-MM-DD HH:MM:SS"
- [ ] Tooltip without timestamp shows "No data available"
- [ ] Datetime format is consistent

---

## Task 3: Panel State Enum

**Priority:** High  
**Complexity:** Low  
**Dependencies:** None

### Test Scenarios (RED)
```rust
#[test]
fn test_panel_state_variants_exist() {
    let _loading = PanelState::Loading;
    let _error = PanelState::Error("test error".to_string());
    // Success and Stale require CopilotUsage
}

#[test]
fn test_panel_state_error_message() {
    let error = PanelState::Error("API failed".to_string());
    match error {
        PanelState::Error(msg) => assert_eq!(msg, "API failed"),
        _ => panic!("Expected Error variant"),
    }
}

#[test]
fn test_panel_state_success_holds_data() {
    let usage = create_mock_copilot_usage();
    let success = PanelState::Success(usage.clone());
    match success {
        PanelState::Success(data) => assert_eq!(data, usage),
        _ => panic!("Expected Success variant"),
    }
}
```

### Implementation (GREEN)
- Create `src/ui/state.rs` module
- Define `PanelState` enum with variants: Loading, Success, Error, Stale
- Derive necessary traits (Debug, Clone)

### Refactoring Opportunities
- Add helper methods (`is_loading()`, `is_error()`, etc.)
- Add state transition validation
- Consider adding timestamp to each state

### Acceptance Criteria
- [ ] All test scenarios pass
- [ ] `PanelState` enum has 4 variants
- [ ] Success and Stale variants hold `CopilotUsage` data
- [ ] Error variant holds error message string
- [ ] Loading variant has no data

---

## Task 4: AppState Structure

**Priority:** High  
**Complexity:** Low  
**Dependencies:** Task 3

### Test Scenarios (RED)
```rust
#[test]
fn test_app_state_initial_state() {
    let config = create_mock_config();
    let state = AppState::new(config.clone());
    
    assert!(matches!(state.panel_state, PanelState::Loading));
    assert_eq!(state.last_update, None);
    assert_eq!(state.config, config);
}

#[test]
fn test_app_state_update_to_success() {
    let mut state = AppState::new(create_mock_config());
    let usage = create_mock_copilot_usage();
    
    state.update_success(usage.clone());
    
    assert!(matches!(state.panel_state, PanelState::Success(_)));
    assert!(state.last_update.is_some());
}

#[test]
fn test_app_state_update_to_error() {
    let mut state = AppState::new(create_mock_config());
    
    state.update_error("Network error".to_string());
    
    assert!(matches!(state.panel_state, PanelState::Error(_)));
    assert_eq!(state.last_update, None);
}
```

### Implementation (GREEN)
- Add to `src/ui/state.rs` module
- Define `AppState` struct with fields: panel_state, last_update, config
- Implement constructor and update methods
- Add helper methods for state transitions

### Refactoring Opportunities
- Extract state transition logic to separate methods
- Add validation for state transitions
- Consider using a state machine pattern

### Acceptance Criteria
- [ ] All test scenarios pass
- [ ] AppState has fields: panel_state, last_update, config
- [ ] Constructor initializes to Loading state
- [ ] Update methods properly transition state
- [ ] Last update timestamp is set on success

---

## Task 5: Message Enum for UI Events

**Priority:** High  
**Complexity:** Low  
**Dependencies:** None

### Test Scenarios (RED)
```rust
#[test]
fn test_message_variants_exist() {
    let _fetch = Message::FetchMetrics;
    let _theme = Message::ThemeChanged;
    let _tooltip = Message::UpdateTooltip;
}

#[test]
fn test_message_metrics_fetched_success() {
    let usage = create_mock_copilot_usage();
    let msg = Message::MetricsFetched(Ok(usage.clone()));
    
    match msg {
        Message::MetricsFetched(Ok(data)) => assert_eq!(data, usage),
        _ => panic!("Expected MetricsFetched Ok variant"),
    }
}

#[test]
fn test_message_metrics_fetched_error() {
    let error = crate::core::github::GitHubError::NetworkError("timeout".into());
    let msg = Message::MetricsFetched(Err(error));
    
    assert!(matches!(msg, Message::MetricsFetched(Err(_))));
}
```

### Implementation (GREEN)
- Create `src/ui/mod.rs` if not exists
- Define `Message` enum with variants from design
- Derive necessary traits (Debug, Clone)

### Refactoring Opportunities
- Add message priority levels
- Group related messages
- Add message validation

### Acceptance Criteria
- [ ] All test scenarios pass
- [ ] Message enum has all variants from design
- [ ] MetricsFetched holds Result with CopilotUsage or error
- [ ] All variants are well-documented

---

## Task 6: Get Primary Metric from CopilotUsage

**Priority:** High  
**Complexity:** Low  
**Dependencies:** None

### Test Scenarios (RED)
```rust
#[test]
fn test_get_completions_count() {
    let usage = create_mock_copilot_usage_with_completions(1234);
    assert_eq!(get_primary_metric(&usage), 1234);
}

#[test]
fn test_get_primary_metric_no_breakdown() {
    let usage = CopilotUsage {
        breakdown: vec![],
        ..Default::default()
    };
    assert_eq!(get_primary_metric(&usage), 0);
}

#[test]
fn test_get_primary_metric_sums_all_editors() {
    let usage = create_mock_copilot_usage_multi_editor();
    // If VSCode has 100 and JetBrains has 50 completions
    assert_eq!(get_primary_metric(&usage), 150);
}
```

### Implementation (GREEN)
- Add to `src/ui/formatters.rs` module
- Implement `get_primary_metric(usage: &CopilotUsage) -> u64`
- Sum completions from all breakdown items

### Refactoring Opportunities
- Support different metric types (acceptance rate, etc.)
- Add configuration for metric selection
- Handle missing or invalid data gracefully

### Acceptance Criteria
- [ ] All test scenarios pass
- [ ] Returns total completions count from all editors
- [ ] Returns 0 when no breakdown data
- [ ] Correctly sums across multiple editors

---

## Task 7: Format Metric Display Text

**Priority:** High  
**Complexity:** Low  
**Dependencies:** Task 1, Task 6

### Test Scenarios (RED)
```rust
#[test]
fn test_format_metric_display_with_data() {
    let usage = create_mock_copilot_usage_with_completions(1234);
    assert_eq!(format_metric_display(Some(&usage)), "1.2K");
}

#[test]
fn test_format_metric_display_no_data() {
    assert_eq!(format_metric_display(None), "--");
}

#[test]
fn test_format_metric_display_large_numbers() {
    let usage = create_mock_copilot_usage_with_completions(1_234_567);
    assert_eq!(format_metric_display(Some(&usage)), "1.2M");
}

#[test]
fn test_format_metric_display_zero() {
    let usage = create_mock_copilot_usage_with_completions(0);
    assert_eq!(format_metric_display(Some(&usage)), "0");
}
```

### Implementation (GREEN)
- Add to `src/ui/formatters.rs` module
- Implement `format_metric_display(usage: Option<&CopilotUsage>) -> String`
- Use `format_number()` and `get_primary_metric()`

### Refactoring Opportunities
- Add metric type parameter (count, rate, etc.)
- Support custom formatting options
- Add unit tests for edge cases

### Acceptance Criteria
- [ ] All test scenarios pass
- [ ] Returns formatted number when data available
- [ ] Returns "--" when no data
- [ ] Uses number formatting from Task 1
- [ ] Handles zero gracefully

---

## Task 8: Basic Applet Structure (libcosmic Integration)

**Priority:** High  
**Complexity:** Medium  
**Dependencies:** Task 3, Task 4, Task 5

### Test Scenarios (RED)
```rust
// Note: UI testing is limited, focus on structure and state management
#[test]
fn test_applet_initialization() {
    let applet = CopilotMonitorApplet::new();
    assert!(matches!(applet.state.panel_state, PanelState::Loading));
}

#[test]
fn test_applet_has_required_fields() {
    let applet = CopilotMonitorApplet::new();
    // Verify fields exist (compilation test)
    let _ = applet.core;
    let _ = applet.state;
}
```

### Implementation (GREEN)
- Modify `src/app.rs`
- Define `CopilotMonitorApplet` struct
- Implement basic structure without full cosmic::Application yet
- Initialize with AppState in Loading state

### Refactoring Opportunities
- Extract initialization logic to builder pattern
- Add configuration validation
- Consider dependency injection for testing

### Acceptance Criteria
- [ ] Basic test scenarios pass
- [ ] Struct has core, state fields
- [ ] Initializes to Loading state
- [ ] Compiles without errors
- [ ] Can be instantiated for testing

---

## Task 9: Implement Message Handling Logic

**Priority:** High  
**Complexity:** Medium  
**Dependencies:** Task 4, Task 5, Task 8

### Test Scenarios (RED)
```rust
#[test]
fn test_handle_fetch_metrics_starts_loading() {
    let mut applet = create_test_applet();
    applet.handle_message(Message::FetchMetrics);
    assert!(matches!(applet.state.panel_state, PanelState::Loading));
}

#[test]
fn test_handle_metrics_fetched_success() {
    let mut applet = create_test_applet();
    let usage = create_mock_copilot_usage();
    
    applet.handle_message(Message::MetricsFetched(Ok(usage.clone())));
    
    assert!(matches!(applet.state.panel_state, PanelState::Success(_)));
    assert!(applet.state.last_update.is_some());
}

#[test]
fn test_handle_metrics_fetched_error() {
    let mut applet = create_test_applet();
    let error = GitHubError::NetworkError("timeout".into());
    
    applet.handle_message(Message::MetricsFetched(Err(error)));
    
    assert!(matches!(applet.state.panel_state, PanelState::Error(_)));
}
```

### Implementation (GREEN)
- Add to `src/app.rs`
- Implement `handle_message()` method
- Handle all message variants
- Update state based on message type

### Refactoring Opportunities
- Extract message handlers to separate methods
- Add logging for state transitions
- Consider command pattern for complex messages

### Acceptance Criteria
- [ ] All test scenarios pass
- [ ] FetchMetrics triggers loading state
- [ ] MetricsFetched(Ok) updates to success state
- [ ] MetricsFetched(Err) updates to error state
- [ ] State transitions are logged

---

## Task 10: Async API Fetching with Command

**Priority:** High  
**Complexity:** Medium  
**Dependencies:** Task 9

### Test Scenarios (RED)
```rust
#[tokio::test]
async fn test_fetch_metrics_command_success() {
    let client = create_mock_github_client_success();
    let config = create_mock_config();
    
    let result = fetch_metrics_command(client, config).await;
    
    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), Message::MetricsFetched(Ok(_))));
}

#[tokio::test]
async fn test_fetch_metrics_command_error() {
    let client = create_mock_github_client_error();
    let config = create_mock_config();
    
    let result = fetch_metrics_command(client, config).await;
    
    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), Message::MetricsFetched(Err(_))));
}
```

### Implementation (GREEN)
- Add to `src/app.rs`
- Implement async command that calls GitHubClient
- Return Message::MetricsFetched with result
- Handle errors gracefully

### Refactoring Opportunities
- Add retry logic
- Add timeout handling
- Extract to separate module
- Add progress reporting

### Acceptance Criteria
- [ ] All test scenarios pass
- [ ] Command calls GitHubClient.get_copilot_usage()
- [ ] Returns MetricsFetched message with result
- [ ] Handles errors without panicking
- [ ] Works with mock client for testing

---

## Task 11: View Function - Panel Widget Construction

**Priority:** High  
**Complexity:** Medium  
**Dependencies:** Task 7, Task 8, Task 9

### Test Scenarios (RED)
```rust
// Note: Full UI testing requires integration tests
// Focus on logic and component construction

#[test]
fn test_view_returns_widget() {
    let applet = create_test_applet_with_success_state();
    let element = applet.view();
    // Verify element is created (compilation test)
}

#[test]
fn test_view_displays_metric_text() {
    let applet = create_test_applet_with_success_state();
    let metric_text = applet.get_metric_text();
    assert!(!metric_text.is_empty());
}

#[test]
fn test_view_shows_placeholder_on_error() {
    let applet = create_test_applet_with_error_state();
    let metric_text = applet.get_metric_text();
    assert_eq!(metric_text, "--");
}
```

### Implementation (GREEN)
- Add to `src/app.rs`
- Implement `view()` method
- Create basic widget hierarchy (icon + text)
- Display metric based on state
- Use libcosmic widgets

### Refactoring Opportunities
- Extract widget construction to helper methods
- Add styling and theming
- Improve layout and spacing

### Acceptance Criteria
- [ ] Basic test scenarios pass
- [ ] View method constructs widget tree
- [ ] Displays metric text from state
- [ ] Shows "--" placeholder on error/loading
- [ ] Uses libcosmic widget components

---

## Task 12: Add Tooltip to Panel Widget

**Priority:** Medium  
**Complexity:** Low  
**Dependencies:** Task 2, Task 11

### Test Scenarios (RED)
```rust
#[test]
fn test_tooltip_text_with_data() {
    let applet = create_test_applet_with_success_state();
    let tooltip = applet.get_tooltip_text();
    assert!(tooltip.starts_with("Last updated:"));
}

#[test]
fn test_tooltip_text_without_data() {
    let applet = create_test_applet_initial_state();
    let tooltip = applet.get_tooltip_text();
    assert_eq!(tooltip, "No data available");
}
```

### Implementation (GREEN)
- Modify `src/app.rs` view method
- Add tooltip to root widget
- Use `format_tooltip()` from Task 2
- Display last update time from state

### Refactoring Opportunities
- Add more detail to tooltip (org name, etc.)
- Style tooltip appearance
- Add keyboard accessibility

### Acceptance Criteria
- [ ] All test scenarios pass
- [ ] Tooltip shows last update time when available
- [ ] Tooltip shows "No data available" when no data
- [ ] Tooltip appears on hover
- [ ] Uses libcosmic tooltip widget

---

## Task 13: Implement cosmic::Application Trait

**Priority:** High  
**Complexity:** Medium  
**Dependencies:** Task 8, Task 10, Task 11

### Test Scenarios (RED)
```rust
// Integration test level
#[test]
fn test_application_implements_trait() {
    // Compilation test - ensure trait is implemented
    fn assert_implements_application<T: cosmic::Application>() {}
    assert_implements_application::<CopilotMonitorApplet>();
}
```

### Implementation (GREEN)
- Modify `src/app.rs`
- Implement `cosmic::Application` trait for CopilotMonitorApplet
- Implement required methods: init, update, view
- Wire up message handling and commands

### Refactoring Opportunities
- Extract trait implementation to separate file
- Add proper error handling
- Add lifecycle logging

### Acceptance Criteria
- [ ] Trait implementation compiles
- [ ] init() initializes state and triggers FetchMetrics
- [ ] update() routes to handle_message()
- [ ] view() returns panel widget
- [ ] Application can be run

---

## Task 14: Add Icon to Panel Widget

**Priority:** Medium  
**Complexity:** Medium  
**Dependencies:** Task 11

### Test Scenarios (RED)
```rust
#[test]
fn test_icon_widget_created() {
    let applet = create_test_applet();
    // Verify icon is part of widget tree
    // This is more of an integration test
}
```

### Implementation (GREEN)
- Modify view() in `src/app.rs`
- Add icon widget using libcosmic::widget::icon
- Use appropriate icon (system icon or custom SVG)
- Position icon next to metric text

### Refactoring Opportunities
- Support custom icon paths
- Add icon size configuration
- Add state-based icon variations (loading spinner, error icon)

### Acceptance Criteria
- [ ] Icon displays in panel
- [ ] Icon is visible in light/dark themes
- [ ] Icon size is appropriate for panel
- [ ] Icon and text are properly aligned

---

## Task 15: Visual State Indicators

**Priority:** Medium  
**Complexity:** Medium  
**Dependencies:** Task 11, Task 14

### Test Scenarios (RED)
```rust
#[test]
fn test_loading_state_visual() {
    let mut applet = create_test_applet();
    applet.state.panel_state = PanelState::Loading;
    
    let indicator = applet.get_state_indicator();
    assert_eq!(indicator, StateIndicator::Loading);
}

#[test]
fn test_error_state_visual() {
    let mut applet = create_test_applet();
    applet.state.panel_state = PanelState::Error("test".into());
    
    let indicator = applet.get_state_indicator();
    assert_eq!(indicator, StateIndicator::Error);
}

#[test]
fn test_success_state_visual() {
    let mut applet = create_test_applet_with_success_state();
    
    let indicator = applet.get_state_indicator();
    assert_eq!(indicator, StateIndicator::Success);
}
```

### Implementation (GREEN)
- Add state indicator logic to view()
- Show loading spinner when Loading state
- Show error icon when Error state
- Show normal icon when Success state
- Consider subtle color changes

### Refactoring Opportunities
- Extract indicator logic to separate component
- Add animation for loading state
- Add configurable indicator styles

### Acceptance Criteria
- [ ] All test scenarios pass
- [ ] Loading state shows spinner/animation
- [ ] Error state shows error indicator
- [ ] Success state shows normal appearance
- [ ] State changes update visual indicator

---

## Task 16: Integration Test - Full Data Flow

**Priority:** High  
**Complexity:** High  
**Dependencies:** Task 13

### Test Scenarios (RED)
```rust
#[tokio::test]
async fn test_full_flow_success() {
    let mock_client = create_mock_github_client_success();
    let applet = create_test_applet_with_mock_client(mock_client);
    
    // Simulate startup
    applet.init();
    
    // Wait for async fetch
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Verify state transitioned to Success
    assert!(matches!(applet.state.panel_state, PanelState::Success(_)));
}

#[tokio::test]
async fn test_full_flow_error() {
    let mock_client = create_mock_github_client_error();
    let applet = create_test_applet_with_mock_client(mock_client);
    
    applet.init();
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    assert!(matches!(applet.state.panel_state, PanelState::Error(_)));
}
```

### Implementation (GREEN)
- Create integration test file
- Wire up all components
- Test end-to-end data flow
- Use mock GitHubClient for testing

### Refactoring Opportunities
- Add more comprehensive integration tests
- Test error recovery paths
- Add performance benchmarks

### Acceptance Criteria
- [ ] All integration tests pass
- [ ] Data flows from API to UI
- [ ] State transitions work correctly
- [ ] Error handling works end-to-end
- [ ] UI updates reflect data changes

---

## Task 17: Manual Testing and Theme Verification

**Priority:** Medium  
**Complexity:** Low  
**Dependencies:** Task 16

### Manual Test Scenarios
- [ ] Applet appears in COSMIC panel
- [ ] Icon is visible and properly sized
- [ ] Metric displays correct number
- [ ] Loading state appears on startup
- [ ] Error state appears when API fails
- [ ] Tooltip shows on hover
- [ ] Tooltip shows correct last update time
- [ ] Light theme: icon and text visible
- [ ] Dark theme: icon and text visible
- [ ] Numbers are formatted correctly (K, M suffixes)

### Implementation
- Build and run applet on COSMIC desktop
- Test all scenarios manually
- Take screenshots for documentation
- Fix any visual issues discovered

### Refactoring Opportunities
- Adjust spacing and alignment
- Fine-tune colors and contrast
- Optimize icon rendering

### Acceptance Criteria
- [ ] All manual tests pass
- [ ] Applet looks professional
- [ ] No visual glitches
- [ ] Accessible in both themes
- [ ] Meets COSMIC design guidelines

---

## Task 18: Documentation and Code Cleanup

**Priority:** Low  
**Complexity:** Low  
**Dependencies:** Task 17

### Activities
- [ ] Add documentation comments to public APIs
- [ ] Document state machine transitions
- [ ] Add usage examples
- [ ] Clean up unused imports
- [ ] Remove debug code
- [ ] Format code with rustfmt
- [ ] Run clippy and fix warnings

### Implementation
- Review all code in src/ui/ and src/app.rs
- Add comprehensive doc comments
- Create module-level documentation
- Ensure code follows Rust best practices

### Acceptance Criteria
- [ ] All public APIs documented
- [ ] No clippy warnings
- [ ] Code is formatted
- [ ] README updated (if needed)
- [ ] Examples are clear

---

## Summary

### Total Tasks: 18
### Estimated Complexity:
- **Low:** 10 tasks
- **Medium:** 7 tasks
- **High:** 1 task

### Task Dependencies Flow:
```
1 (format) ──┬──> 7 (metric display) ──> 11 (view)
2 (tooltip) ─┘                            │
3 (state) ──> 4 (AppState) ──> 8 (applet)─┤
5 (message) ────────────────> 9 (handler)─┤
6 (get metric) ─────────────> 7           │
                                          │
10 (async) ──> 13 (trait impl) ─────────> 16 (integration)
11 ──> 12 (tooltip widget)                │
11 ──> 14 (icon) ──> 15 (indicators) ─────┤
                                          │
                                          ▼
                                    17 (manual test)
                                          │
                                          ▼
                                    18 (docs)
```

### Next Steps:
1. Start with Task 1 (Number Formatting)
2. Follow TDD cycle strictly: Red → Green → Refactor
3. Run all tests after each task completion
4. Commit after each completed task (when all tests pass)
5. Request review before moving to next phase

---

**Ready to begin implementation?**

Choose your implementation approach:
- **TDD with AI**: I guide you through each test first, then implementation
- **Standard**: Write tests and implementation together with review
- **Self-implementation**: You implement following these tasks, I review
- **Collaborative**: We pair program through each task
