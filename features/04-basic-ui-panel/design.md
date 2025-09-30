# Design Document: Basic UI Panel Display

**Feature:** 04 - Basic UI Panel Display  
**Project:** COSMIC Copilot Usage Monitor  
**Date:** 2025-09-30

---

## 1. Architecture Overview

The Basic UI Panel Display feature integrates the COSMIC desktop applet frontend (using libcosmic) with the existing backend components:

- **ConfigManager**: Provides configuration and authentication details.
- **GitHubClient**: Fetches Copilot usage metrics from the GitHub API.
- **Domain Models**: (e.g., `CopilotUsage`, `UsageBreakdown`) represent usage data.

**Integration Flow:**
- On applet startup, the UI requests metrics from the backend via `GitHubClient`.
- The UI panel component displays the Copilot icon and a primary metric (e.g., completions count).
- The UI reflects the current state: loading, success, error, or stale.
- The UI updates automatically when new data arrives or when the theme changes.

**Diagram:**

```
Backend                    Frontend (libcosmic)
┌──────────────────┐       ┌──────────────────────┐
│ ConfigManager    │──────>│ PanelDisplayComponent│
│ GitHubClient     │       │   ├─ IconComponent   │
│ Domain Models    │       │   ├─ MetricText      │
└──────────────────┘       │   └─ Tooltip         │
                           └──────────────────────┘
```

---

## 2. Component Breakdown

### PanelDisplayComponent
- Root UI component for the COSMIC panel.
- Manages state and orchestrates subcomponents.
- Handles data fetching and state transitions.

### IconComponent
- Displays the GitHub Copilot icon.
- Changes appearance based on state (normal, loading, error, stale).
- Uses libcosmic icon APIs and theme support.

### MetricTextComponent
- Renders the primary metric (completions count or acceptance rate).
- Formats numbers for readability (e.g., "1.2K").
- Shows placeholder ("--") when no data.

### TooltipComponent
- Shows last update time on hover.
- Ensures accessibility and contrast.

### StateIndicator
- Visual cues for loading, error, and stale states (spinner, error icon, faded style).

---

## 3. State Management

The UI maintains a finite state machine with the following states:

**State Model:**

```rust
enum PanelState {
    Loading,
    Success(CopilotUsage),
    Error(String),
    Stale(CopilotUsage),
}

struct AppState {
    panel_state: PanelState,
    last_update: Option<chrono::DateTime<chrono::Utc>>,
    config: AppConfig,
}
```

**State Transitions:**
- On startup → Loading
- On successful fetch → Success
- On fetch failure → Error
- On data timeout/expiry → Stale

State changes trigger UI updates via libcosmic's reactive/event system.

---

## 4. Data Flow

1. **Startup**: PanelDisplayComponent triggers data fetch via GitHubClient.
2. **Loading**: UI shows spinner/loading indicator.
3. **API Response**:
    - On success: MetricTextComponent displays formatted metric; TooltipComponent updates last fetch time.
    - On error: IconComponent switches to error icon; MetricTextComponent shows placeholder.
4. **Updates**: On new data or config changes, UI refreshes metric and tooltip.

**Sequence:**

```
UI → GitHubClient → GitHub API → CopilotUsage → UI Update → Render
```

---

## 5. Visual Design

### Panel Layout
- **Icon**: GitHub Copilot logo, sized for panel, theme-aware.
- **Metric**: Large, readable number or percentage next to/overlaid on icon.
- **Tooltip**: On hover, shows "Last updated: [time]".

### Visual States
- **Loading**: Spinner or animated icon overlay.
- **Success**: Normal icon, metric in bold.
- **Error**: Error icon (e.g., exclamation mark), metric replaced with "--".
- **Stale**: Icon faded or with clock overlay.

### Accessibility
- Text and icon contrast meet WCAG standards.
- Tooltip text is clear and readable.
- Responsive to theme changes (light/dark).

---

## 6. Error Handling Approach

- **API Failure**: Switch to error state, show error icon and placeholder metric.
- **No Data**: Show "--" as metric, tooltip indicates "No data available".
- **Timeout/Slow Response**: If no data within 2 seconds, show loading or error.
- **Logging**: Errors are logged for diagnostics.

**Error Recovery:**
- User can manually retry fetch (future feature).
- Automatic retry with exponential backoff (future feature).

---

## 7. libcosmic Integration Details

### Core Components
- **Icon Rendering**: Use `libcosmic::widget::icon` for Copilot logo.
- **Text Rendering**: Use `libcosmic::widget::text` for metric display.
- **Panel Embedding**: Implement `cosmic::app::applet::CosmicApplet` trait.
- **Theming**: Use `libcosmic::theme` for automatic theme adaptation.
- **Tooltip**: Use `libcosmic::widget::tooltip` for hover text.

### Application Structure
```rust
struct CopilotMonitorApplet {
    core: cosmic::app::Core,
    state: AppState,
    github_client: GitHubClient,
    config_manager: ConfigManager,
}

impl cosmic::Application for CopilotMonitorApplet {
    // Implement application lifecycle
}
```

### Message Passing
```rust
enum Message {
    FetchMetrics,
    MetricsFetched(Result<CopilotUsage, Error>),
    UpdateTooltip,
    ThemeChanged,
}
```

---

## 8. Testing Strategy for UI Components

### Unit Tests
- Test state transitions (loading, success, error, stale).
- Test metric formatting (e.g., "1.2K", "--").
- Test tooltip text generation.

### Integration Tests
- Simulate API responses (success, error, no data).
- Verify UI updates within 2 seconds of data arrival.
- Test theme change handling.

### Visual/Manual Tests
- Verify appearance in light and dark themes.
- Check accessibility (contrast, tooltip readability).
- Confirm panel layout fits COSMIC guidelines.

### Automated UI Tests
- Mock API responses.
- Test state transitions.
- Verify error handling.

---

## 9. Implementation Notes

### Number Formatting
```rust
fn format_number(n: u64) -> String {
    match n {
        0..=999 => n.to_string(),
        1_000..=999_999 => format!("{:.1}K", n as f64 / 1000.0),
        _ => format!("{:.1}M", n as f64 / 1_000_000.0),
    }
}
```

### Tooltip Text
```rust
fn format_tooltip(last_update: Option<DateTime<Utc>>) -> String {
    match last_update {
        Some(time) => format!("Last updated: {}", time.format("%Y-%m-%d %H:%M:%S")),
        None => "No data available".to_string(),
    }
}
```

---

## References

- [requirements.md](requirements.md)
- [libcosmic documentation](https://github.com/pop-os/libcosmic)
- [cosmic_text documentation](https://github.com/pop-os/cosmic-text)
- [COSMIC design guidelines](https://github.com/pop-os/cosmic-design)

---

## Approval Gate

Does this technical design address all requirements for the Basic UI Panel Display?

**Review Checklist:**
- [ ] Architecture integrates with existing backend
- [ ] Component breakdown is clear and logical
- [ ] State management covers all required states
- [ ] Data flow is well-defined
- [ ] Visual design meets COSMIC guidelines
- [ ] Error handling is comprehensive
- [ ] libcosmic integration is detailed
- [ ] Testing strategy is thorough

If approved, proceed to task breakdown phase.
