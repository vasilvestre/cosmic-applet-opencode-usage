# Technical Design: Multiple Panel Metrics Selection

## Overview
This design document details how to implement multiple panel metrics selection by replacing the single `PanelMetric` enum value with a `Vec<PanelMetric>` collection, updating UI to use checkboxes instead of radio buttons, and creating a new formatter to display multiple metrics.

## Architecture Changes

### 1. Data Model Changes

#### `src/core/config.rs`

**Current State:**
```rust
pub enum PanelMetric {
    None,
    Cost,
    Interactions,
    InputTokens,
    OutputTokens,
    ReasoningTokens,
}

pub struct AppConfig {
    pub panel_metric: PanelMetric,
    // ... other fields
}
```

**New State:**
```rust
pub enum PanelMetric {
    // Remove None variant
    Cost,
    Interactions,
    InputTokens,
    OutputTokens,
    ReasoningTokens,
}

pub struct AppConfig {
    pub panel_metrics: Vec<PanelMetric>,  // Changed from panel_metric
    // ... other fields
}
```

**Changes Required:**
1. Remove `PanelMetric::None` variant
2. Rename `panel_metric` → `panel_metrics` (plural)
3. Change type from `PanelMetric` → `Vec<PanelMetric>`
4. Update `Default::default()` to return all metrics enabled
5. Update serialization/deserialization (serde handles Vec automatically)
6. Update load/save methods to use "panel_metrics" key

#### `src/app.rs`

**Current State:**
```rust
pub struct CosmicAppletOpencodeUsage {
    temp_panel_metric: PanelMetric,
    // ... other fields
}
```

**New State:**
```rust
pub struct CosmicAppletOpencodeUsage {
    temp_panel_metrics: Vec<PanelMetric>,  // Changed to Vec
    // ... other fields
}
```

**Changes Required:**
1. Rename `temp_panel_metric` → `temp_panel_metrics`
2. Change type from `PanelMetric` → `Vec<PanelMetric>`
3. Update initialization in `new()` and `default()`

### 2. Message Handling Changes

#### `src/ui/messages.rs` (if exists) or `src/app.rs`

**Current:**
```rust
pub enum Message {
    SelectPanelMetric(PanelMetric),
    // ... other messages
}
```

**New:**
```rust
pub enum Message {
    TogglePanelMetric(PanelMetric),  // Renamed: toggle instead of select
    ResetPanelMetricsToDefaults,     // New: reset button
    // ... other messages
}
```

**Logic Changes:**
- `TogglePanelMetric(metric)`: Add/remove metric from `temp_panel_metrics` vec
  - If metric exists in vec: remove it
  - If metric doesn't exist: add it in sorted order
- `ResetPanelMetricsToDefaults`: Set `temp_panel_metrics` to all metrics

### 3. UI Changes

#### Settings Dialog (`src/app.rs::settings_view()`)

**Current:** Radio buttons (single selection)
```rust
cosmic::widget::radio("Cost", PanelMetric::Cost, 
    Some(self.temp_panel_metric), Message::SelectPanelMetric)
```

**New:** Checkboxes (multi-selection)
```rust
cosmic::widget::checkbox("Cost", self.temp_panel_metrics.contains(&PanelMetric::Cost))
    .on_toggle(|_| Message::TogglePanelMetric(PanelMetric::Cost))
```

**Add:** Reset to Defaults button
```rust
cosmic::widget::button::text("Reset to Defaults")
    .on_press(Message::ResetPanelMetricsToDefaults)
```

### 4. Formatter Changes

#### `src/ui/formatters.rs`

**New Function:** `format_multiple_panel_metrics()`
```rust
pub fn format_multiple_panel_metrics(
    usage: &UsageMetrics,
    metrics: &[PanelMetric],
    use_raw: bool,
) -> String
```

**Logic:**
1. If `metrics` is empty, return empty string (icon only)
2. For each metric in fixed order (Cost, Interactions, InputTokens, OutputTokens, ReasoningTokens):
   - If metric is in `metrics` list, format it with new labels:
     - Cost: "$1.23" (unchanged)
     - Interactions: "5x" (unchanged)
     - InputTokens: "IT: 10k" (new prefix)
     - OutputTokens: "OT: 5k" (new prefix)
     - ReasoningTokens: "RT: 2k" (new prefix)
3. Join formatted strings with " | " separator
4. Return combined string

**Helper:** Ensure consistent ordering
```rust
const METRIC_ORDER: [PanelMetric; 5] = [
    PanelMetric::Cost,
    PanelMetric::Interactions,
    PanelMetric::InputTokens,
    PanelMetric::OutputTokens,
    PanelMetric::ReasoningTokens,
];
```

### 5. Panel Button Content Changes

#### `src/app.rs::panel_button_content()`

**Current:**
```rust
if self.state.config.panel_metric != PanelMetric::None {
    let display_text = format_panel_metric(
        today_usage,
        self.state.config.panel_metric,
        self.state.config.use_raw_token_display,
    );
    // ... render icon + text
}
```

**New:**
```rust
if !self.state.config.panel_metrics.is_empty() {
    let display_text = format_multiple_panel_metrics(
        today_usage,
        &self.state.config.panel_metrics,
        self.state.config.use_raw_token_display,
    );
    // ... render icon + text
}
```

### 6. Data Fetching Logic Changes

#### `src/app.rs::update()` and async tasks

**Current:**
```rust
let today_metrics = if panel_metric != PanelMetric::None {
    reader.get_usage_today().ok()
} else {
    None
};
```

**New:**
```rust
let today_metrics = if !panel_metrics.is_empty() {
    reader.get_usage_today().ok()
} else {
    None
};
```

**Changes:**
- Replace `panel_metric != PanelMetric::None` → `!panel_metrics.is_empty()`
- Replace `panel_metric == PanelMetric::None` → `panel_metrics.is_empty()`

## Implementation Strategy (TDD)

### Phase 1: Data Model (Config)
1. Write tests for `Vec<PanelMetric>` serialization
2. Remove `PanelMetric::None` variant
3. Change `panel_metric` → `panel_metrics` (Vec)
4. Update Default implementation (all metrics enabled)
5. Update load/save methods

### Phase 2: Formatter
1. Write tests for `format_multiple_panel_metrics()`
2. Implement formatter with fixed ordering
3. Test all combinations (empty, single, multiple, all)
4. Test N/A handling for missing data

### Phase 3: App State & Messages
1. Write tests for state changes (`temp_panel_metrics`)
2. Update `CosmicAppletOpencodeUsage` struct
3. Add `TogglePanelMetric` message handler (add/remove logic)
4. Add `ResetPanelMetricsToDefaults` message handler
5. Update all initialization points

### Phase 4: UI Integration
1. Update settings dialog (radio → checkbox)
2. Add "Reset to Defaults" button
3. Update `panel_button_content()` to use new formatter
4. Update data fetching conditions

### Phase 5: Test Updates
1. Update all existing tests that reference `panel_metric`
2. Fix compilation errors from removed `None` variant
3. Update test expectations for new default (all metrics)

## Data Structures

### Metric Order Constant
```rust
pub const METRIC_DISPLAY_ORDER: [PanelMetric; 5] = [
    PanelMetric::Cost,
    PanelMetric::Interactions,
    PanelMetric::InputTokens,
    PanelMetric::OutputTokens,
    PanelMetric::ReasoningTokens,
];
```

### Helper Functions
```rust
// Returns metrics in display order, filtering for selected ones
pub fn order_metrics(selected: &[PanelMetric]) -> Vec<PanelMetric> {
    METRIC_DISPLAY_ORDER
        .iter()
        .filter(|m| selected.contains(m))
        .copied()
        .collect()
}

// Returns all metrics (for default/reset)
pub fn all_metrics() -> Vec<PanelMetric> {
    METRIC_DISPLAY_ORDER.to_vec()
}
```

## Edge Cases

1. **Empty selection**: Display icon only (no text)
2. **Duplicate prevention**: Use `contains()` check before adding
3. **Missing data**: Each metric formatter already handles N/A
4. **Order preservation**: Always use `METRIC_DISPLAY_ORDER` for iteration
5. **Save/Load**: Vec serialization handled by serde automatically

## Testing Strategy

### Unit Tests
- `PanelMetric` default (all metrics)
- `Vec<PanelMetric>` serialization/deserialization
- `format_multiple_panel_metrics()` with various combinations
- Toggle logic (add/remove from vec)
- Order preservation

### Integration Tests
- Settings dialog checkbox interactions
- Save/load roundtrip with multiple metrics
- Panel button content with multiple metrics
- Reset to defaults functionality

## Migration Notes

Since we're not handling migration (no users except developer):
- Old configs with `panel_metric: PanelMetric` will fail to load
- Will fall back to default (all metrics enabled)
- This is acceptable for the use case

## Performance Considerations

- Vec size is max 5 elements (very small)
- No performance impact expected
- `contains()` is O(n) but n ≤ 5
- Consider `HashSet` only if performance issues arise (not expected)

## Security Considerations

None - all changes are internal data structure and UI updates.

## Dependencies

No new dependencies required. Uses existing:
- `serde` for Vec serialization
- `cosmic` widgets for checkboxes
- Existing formatter infrastructure
