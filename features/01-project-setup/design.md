# Technical Design: Project Setup & Boilerplate Applet

## Overview
This design document outlines the technical approach for establishing the foundational COSMIC applet structure based on the official cosmic-applet-template. The implementation follows the libcosmic applet architecture patterns.

## Architecture

### Module Organization
```
src/
├── main.rs           # Entry point, calls cosmic::applet::run()
├── app.rs            # Application implementation with Application trait
└── core/
    ├── mod.rs        # Core module exports
    └── localization.rs  # i18n support using i18n-embed
```

### Core Components

#### 1. Application Struct (`YourApp`)
```rust
pub struct YourApp {
    core: Core,              // COSMIC runtime state
    popup: Option<Id>,       // Popup window ID tracking
    example_row: bool,       // Example UI state
}
```

**Purpose**: Main application state container
**Responsibilities**:
- Manage COSMIC Core runtime
- Track popup window lifecycle
- Store application state

#### 2. Message Enum
```rust
pub enum Message {
    TogglePopup,           // Toggle popup window visibility
    PopupClosed(Id),       // Handle popup close event
    ToggleExampleRow(bool), // Example UI interaction
}
```

**Purpose**: Type-safe message passing for UI events
**Pattern**: Elm Architecture (Model-View-Update)

#### 3. Application Trait Implementation

**Required Methods**:
- `init()`: Initialize application state, return (App, Task)
- `view()`: Render panel icon button
- `view_window()`: Render popup window content
- `update()`: Handle messages and state updates
- `on_close_requested()`: Handle window close events
- `style()`: Return applet-specific styling

### Popup Management Strategy

**Lifecycle**:
1. User clicks icon button → TogglePopup message
2. If popup exists: destroy_popup(id)
3. If no popup: create unique ID, configure settings, get_popup()
4. Window manager closes popup → PopupClosed message
5. Clear popup reference from state

**Settings Configuration**:
```rust
popup_settings.positioner.size_limits = Limits::NONE
    .max_width(372.0)
    .min_width(300.0)
    .min_height(200.0)
    .max_height(1080.0);
```

## Dependency Management

### Cargo.toml Configuration

**Core Dependencies**:
- `libcosmic` (git): Provides applet framework, UI widgets, runtime
  - Features: `applet`, `tokio`, `wayland`
- `i18n-embed` (0.14): i18n system with fluent support
- `i18n-embed-fl` (0.8): Fluent macro integration
- `rust-embed` (8.3.0): Embed resources at compile time

**Versioning**:
- Rust Edition: 2021
- MSRV: 1.80
- License: GPL-3.0

## Internationalization Architecture

### i18n.toml Configuration
```toml
[package.metadata.i18n]
available-locales = ["en", "nl"]
default-locale = "en"
load-path = "i18n"
```

### Translation Files
- Location: `i18n/{locale}/cosmic_applet_template.ftl`
- Format: Fluent (Mozilla's localization system)
- Usage: `fl!("translation-key")` macro

### Localization Module
**Responsibilities**:
- Initialize i18n-embed system
- Load Fluent resources
- Provide fl! macro for string resolution

## Resource Management

### Desktop Integration

**Desktop Entry** (`res/com.example.CosmicAppletTemplate.desktop`):
- Type: Application
- Categories: System;Utility
- NoDisplay: true (applet, not standalone app)
- StartupNotify: false

**MetaInfo XML** (`res/com.example.CosmicAppletTemplate.metainfo.xml`):
- AppStream metadata for software centers
- Component type: desktop-application
- License, description, screenshots

### Icon Strategy

**Formats**: SVG (scalable vector graphics)
**Sizes**: 16, 24, 32, 48, 64, 128, 256 pixels
**Location**: `res/icons/hicolor/{size}x{size}/apps/`
**Naming**: `com.example.CosmicAppletTemplate.svg`

**Icon Usage in Code**:
```rust
core.applet.icon_button("display-symbolic")
```
Uses system icon theme, not embedded icons.

## Build System

### Cargo Build Process
1. Compile Rust source with rustc 1.80+
2. Embed resources via rust-embed
3. Link against libcosmic (built from git)
4. Generate binary: `cosmic-applet-template`

### Justfile Tasks
Common development commands:
- `just build`: Cargo build
- `just run`: Cargo run for testing
- `just install`: Install to system
- `just clean`: Clean build artifacts

### Installation
**Binary Location**: `/usr/bin/` or `~/.local/bin/`
**Desktop File**: `/usr/share/applications/` or `~/.local/share/applications/`
**Icons**: `/usr/share/icons/hicolor/` or `~/.local/share/icons/hicolor/`

## UI Component Design

### Panel View (icon button)
```rust
fn view(&self) -> Element<Self::Message> {
    self.core.applet.icon_button("display-symbolic")
        .on_press(Message::TogglePopup)
        .into()
}
```

**Behavior**:
- Displays icon in COSMIC panel
- Clickable button triggers TogglePopup message
- Icon from system theme

### Popup Window View
```rust
fn view_window(&self, _id: Id) -> Element<Self::Message> {
    let content_list = widget::list_column()
        .padding(5)
        .spacing(0)
        .add(settings::item(
            fl!("example-row"),
            widget::toggler(self.example_row)
                .on_toggle(Message::ToggleExampleRow),
        ));
    
    self.core.applet.popup_container(content_list).into()
}
```

**Structure**:
- `popup_container`: Provides applet-styled container
- `list_column`: Vertical list layout
- `settings::item`: Standard settings row pattern
- `widget::toggler`: Example interactive widget

## State Management

### State Flow
```
User Click → Message → Update → State Change → View Re-render
```

**Example Flow (Toggle Popup)**:
1. User clicks icon button
2. `on_press` sends `Message::TogglePopup`
3. `update()` receives message
4. Check if popup exists
5. If yes: destroy_popup Task
6. If no: create popup with get_popup Task
7. Update self.popup field
8. Return Task for execution
9. View re-renders based on new state

### Task System
- Async command execution via libcosmic runtime
- Tasks returned from `update()` and `init()`
- Window management tasks: get_popup, destroy_popup
- `Task::none()` for synchronous updates

## Error Handling Strategy

**Current Scope**: Minimal (template level)
- Rely on libcosmic error handling
- Panic on unrecoverable errors (development phase)
- No custom error types needed yet

**Future Considerations** (for quota tracker):
- Network error handling (GitHub API)
- Configuration file errors
- Display error states in UI

## Testing Strategy

### Manual Testing
- Build and run in COSMIC desktop
- Verify icon appears in panel
- Click icon, verify popup appears
- Click icon again, verify popup closes
- Check translations with different locales

### Integration Points to Verify
- [ ] Applet loads in COSMIC panel
- [ ] Icon button renders correctly
- [ ] Popup toggles on click
- [ ] Popup closes when clicking outside
- [ ] Translations load correctly
- [ ] Resources embedded in binary

## Migration Path from Template

To adapt this template for the quota tracker:

1. **Rename Project**:
   - Update Cargo.toml name
   - Update APP_ID in app.rs
   - Rename desktop and metainfo files
   - Update icon filenames

2. **Replace Placeholder UI**:
   - Modify `view()` to show quota status icon
   - Modify `view_window()` to show quota details
   - Update Message enum for quota-specific actions

3. **Add Quota State**:
   - Add quota data fields to YourApp struct
   - Add configuration loading
   - Add GitHub API client integration

4. **Update Translations**:
   - Replace example strings with quota-specific text
   - Add new translation keys for quota UI

## Security Considerations

**Current Scope**:
- GPL-3.0 license compliance
- No sensitive data handling yet

**Future Scope** (quota tracker):
- GitHub token storage (secure config)
- API rate limit enforcement
- Input validation for configuration

## Performance Considerations

**Current Implementation**:
- Minimal overhead: single icon button
- Popup created on-demand (lazy)
- No background tasks in template

**Future Optimization** (quota tracker):
- Cache quota data to minimize API calls
- Background refresh tasks
- Efficient state updates to prevent re-renders

## Deployment

### Local Development
```bash
cargo run  # Test in COSMIC desktop
```

### System Installation
```bash
cargo build --release
sudo cp target/release/cosmic-applet-template /usr/bin/
sudo cp res/*.desktop /usr/share/applications/
sudo cp -r res/icons/* /usr/share/icons/hicolor/
sudo update-desktop-database
```

### Package Distribution
- Create .deb package for Pop!_OS
- Include dependencies in package metadata
- Post-install scripts for icon cache update

## Implementation Notes

### Critical Patterns
1. Always use `cosmic::applet::run()`, not `cosmic::app::run()`
2. Popup ID lifecycle must be carefully managed
3. Use `core.applet.icon_button()` for panel icons
4. Use `core.applet.popup_container()` for popup windows
5. Return `Some(cosmic::applet::style())` from `style()` method

### Common Pitfalls
- Forgetting to clear popup ID on close → memory leak
- Using wrong run function → applet won't load
- Missing wayland feature → runtime errors
- Hardcoded strings instead of fl! → no i18n

## References

- Official cosmic-applet-template repository
- libcosmic documentation
- COSMIC desktop applet guidelines
- Fluent i18n documentation

## Next Steps

After design approval:
1. Create tasks.md with TDD breakdown
2. Implement tests for each component
3. Implement components following Red-Green-Refactor
4. Verify integration with COSMIC panel
5. Document build and installation process
