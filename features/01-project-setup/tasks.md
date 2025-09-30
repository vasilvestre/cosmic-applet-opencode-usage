# Implementation Tasks: Project Setup & Boilerplate Applet

## Overview
This document breaks down Feature 01 (Project Setup) into granular, testable tasks following Test-Driven Development (TDD) methodology. Each task follows the Red-Green-Refactor cycle.

## TDD Implementation Approach

### Red-Green-Refactor Cycle
1. **RED**: Write a failing test that defines desired behavior
2. **GREEN**: Write minimal code to make the test pass
3. **REFACTOR**: Improve code structure while keeping tests green

### Testing Strategy
- **Compilation Tests**: Verify code compiles and dependencies resolve
- **Integration Tests**: Verify applet integrates with COSMIC runtime
- **Manual Tests**: Verify UI behavior in COSMIC panel (automated UI testing not feasible)

---

## Task Breakdown

### Phase 1: Project Foundation

#### Task 1.1: Initialize Cargo Project Structure
**Priority**: High  
**Estimated Time**: 15 minutes

**Test Scenario**:
```bash
# Verify project structure exists
ls Cargo.toml
ls src/main.rs
cargo check  # Should compile (even if empty)
```

**Acceptance Criteria**:
- [ ] Cargo.toml exists with correct package metadata
- [ ] Edition = "2021", rust-version = "1.80"
- [ ] License = "GPL-3.0-only"
- [ ] src/main.rs exists with empty main function
- [ ] src/app.rs stub file created
- [ ] src/core/mod.rs and src/core/localization.rs stub files created
- [ ] `cargo check` runs successfully

**Implementation Steps**:
1. Run `cargo init` if needed (or verify existing structure)
2. Edit Cargo.toml package section
3. Create directory structure: `src/core/`
4. Create stub files with module declarations
5. Run `cargo check` to verify

**Dependencies**: None

---

#### Task 1.2: Configure Core Dependencies
**Priority**: High  
**Estimated Time**: 20 minutes

**Test Scenario**:
```bash
# Verify dependencies resolve and compile
cargo check
cargo tree | grep libcosmic
cargo tree | grep i18n-embed
```

**Acceptance Criteria**:
- [ ] libcosmic added as git dependency with features: ["applet", "tokio", "wayland"]
- [ ] i18n-embed = "0.14" with features: ["fluent-system", "desktop-requester"]
- [ ] i18n-embed-fl = "0.8"
- [ ] rust-embed = "8.3.0"
- [ ] `cargo check` completes without dependency errors
- [ ] `cargo tree` shows all dependencies resolved

**Implementation Steps**:
1. Add libcosmic git dependency to Cargo.toml
2. Add i18n-embed, i18n-embed-fl, rust-embed
3. Run `cargo check` to download and verify dependencies
4. Run `cargo tree` to inspect dependency graph

**Dependencies**: Task 1.1

**Red Phase**:
```bash
cargo check  # Will fail with missing dependencies
```

**Green Phase**:
```toml
[dependencies]
libcosmic = { git = "https://github.com/pop-os/libcosmic", features = ["applet", "tokio", "wayland"] }
i18n-embed = { version = "0.14", features = ["fluent-system", "desktop-requester"] }
i18n-embed-fl = "0.8"
rust-embed = "8.3.0"
```

---

#### Task 1.3: Set Up Internationalization Configuration
**Priority**: Medium  
**Estimated Time**: 15 minutes

**Test Scenario**:
```bash
# Verify i18n files exist
ls i18n.toml
ls i18n/en/cosmic_applet_template.ftl
ls i18n/nl/cosmic_applet_template.ftl
```

**Acceptance Criteria**:
- [ ] i18n.toml exists with correct configuration
- [ ] available-locales = ["en", "nl"]
- [ ] default-locale = "en"
- [ ] load-path = "i18n"
- [ ] Translation files exist with example keys
- [ ] English: `example-row = Example Row`
- [ ] Dutch: `example-row = Voorbeeldrij`

**Implementation Steps**:
1. Create i18n.toml in project root
2. Create i18n/en/ directory
3. Create i18n/nl/ directory
4. Create cosmic_applet_template.ftl in each locale directory
5. Add example translation keys

**Dependencies**: None

---

### Phase 2: Core Application Structure

#### Task 2.1: Implement Localization Module
**Priority**: High  
**Estimated Time**: 25 minutes

**Test Scenario**:
```rust
// In tests/ or within src/core/localization.rs
#[test]
fn test_localization_initialization() {
    // This will be tested by compilation + cargo check
    // Manual test: verify fl! macro works in app.rs
}
```

**Acceptance Criteria**:
- [ ] src/core/localization.rs implements i18n-embed setup
- [ ] Creates Fluent loader with RustEmbed
- [ ] Provides LANGUAGE_LOADER static variable
- [ ] fl! macro available for use
- [ ] Code compiles without errors
- [ ] cargo clippy passes

**Implementation Steps**:
1. Write minimal localization.rs structure (RED)
2. Add i18n-embed imports
3. Create RustEmbed struct for i18n files
4. Initialize LANGUAGE_LOADER with FluentLanguageLoader
5. Set up fl! macro
6. Run `cargo check` (GREEN)
7. Refactor for clarity if needed

**Dependencies**: Task 1.2, Task 1.3

**Red Phase**:
```rust
// Empty file or stub - cargo check will fail on usage
```

**Green Phase**:
```rust
use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DesktopLanguageRequester,
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "i18n/"]
struct Localizations;

pub static LANGUAGE_LOADER: FluentLanguageLoader = fluent_language_loader!();

#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {{
        i18n_embed_fl::fl!($crate::core::localization::LANGUAGE_LOADER, $message_id)
    }};
}

pub fn init() {
    let localizations = Localizations;
    let requested_languages = DesktopLanguageRequester::requested_languages();
    i18n_embed::select(&*LANGUAGE_LOADER, &localizations, &requested_languages).unwrap();
}
```

---

#### Task 2.2: Define Application Message Enum
**Priority**: High  
**Estimated Time**: 10 minutes

**Test Scenario**:
```rust
#[test]
fn test_message_enum_creation() {
    let msg1 = Message::TogglePopup;
    let msg2 = Message::PopupClosed(cosmic::iced::window::Id::unique());
    let msg3 = Message::ToggleExampleRow(true);
    // Should compile without errors
}
```

**Acceptance Criteria**:
- [ ] Message enum defined with variants: TogglePopup, PopupClosed(Id), ToggleExampleRow(bool)
- [ ] Derives Clone, Debug
- [ ] Compiles successfully
- [ ] cargo clippy passes

**Implementation Steps**:
1. Write test for Message enum creation (RED)
2. Define Message enum in src/app.rs
3. Add required derives
4. Run test (GREEN)
5. Verify with cargo clippy

**Dependencies**: Task 2.1

**Red Phase**:
```rust
// Test will fail - Message enum doesn't exist
#[test]
fn test_message_enum_creation() {
    let msg = Message::TogglePopup;  // Compilation error
}
```

**Green Phase**:
```rust
#[derive(Clone, Debug)]
pub enum Message {
    TogglePopup,
    PopupClosed(cosmic::iced::window::Id),
    ToggleExampleRow(bool),
}
```

---

#### Task 2.3: Define Application State Struct
**Priority**: High  
**Estimated Time**: 15 minutes

**Test Scenario**:
```rust
#[test]
fn test_app_struct_initialization() {
    // Verify struct can be instantiated
    // This will be tested via Application::init() implementation
}
```

**Acceptance Criteria**:
- [ ] YourApp struct defined with fields: core, popup, example_row
- [ ] core: cosmic::app::Core
- [ ] popup: Option<cosmic::iced::window::Id>
- [ ] example_row: bool
- [ ] Struct is public
- [ ] Compiles successfully

**Implementation Steps**:
1. Define YourApp struct skeleton (RED - won't compile without Application trait)
2. Add required fields with correct types
3. Ensure struct is public
4. Run `cargo check` (will fail on trait not implemented yet)
5. Document field purposes

**Dependencies**: Task 2.2

**Red Phase**:
```bash
cargo check  # Will fail - struct defined but no trait implementation
```

**Green Phase**:
```rust
pub struct YourApp {
    /// COSMIC runtime state
    core: cosmic::app::Core,
    /// Currently open popup window ID
    popup: Option<cosmic::iced::window::Id>,
    /// Example state for UI demonstration
    example_row: bool,
}
```

---

### Phase 3: Application Trait Implementation

#### Task 3.1: Implement Application Trait Skeleton
**Priority**: High  
**Estimated Time**: 30 minutes

**Test Scenario**:
```bash
# Verify trait implementation compiles
cargo check
cargo clippy
```

**Acceptance Criteria**:
- [ ] Application trait implemented for YourApp
- [ ] const APP_ID: &'static str defined
- [ ] type Executor = cosmic::executor::Default
- [ ] type Flags = ()
- [ ] type Message = Message
- [ ] All required methods present (may be stubs)
- [ ] Compiles without errors

**Implementation Steps**:
1. Add `impl cosmic::Application for YourApp` (RED)
2. Define associated types (APP_ID, Executor, Flags, Message)
3. Add stub implementations for all required methods
4. Run `cargo check` (GREEN)
5. Verify with cargo clippy

**Dependencies**: Task 2.3

**Red Phase**:
```bash
cargo check  # Fails with "trait not implemented"
```

**Green Phase**:
```rust
impl cosmic::Application for YourApp {
    const APP_ID: &'static str = "com.example.CosmicAppletTemplate";
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    fn core(&self) -> &cosmic::app::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::app::Core {
        &mut self.core
    }

    fn init(core: cosmic::app::Core, _flags: Self::Flags) -> (Self, cosmic::iced::Task<Self::Message>) {
        // Stub implementation
        todo!("Implement init")
    }

    fn view(&self) -> cosmic::prelude::Element<Self::Message> {
        // Stub implementation
        todo!("Implement view")
    }

    fn update(&mut self, message: Self::Message) -> cosmic::iced::Task<Self::Message> {
        // Stub implementation
        todo!("Implement update")
    }
}
```

---

#### Task 3.2: Implement init() Method
**Priority**: High  
**Estimated Time**: 20 minutes

**Test Scenario**:
```rust
#[test]
fn test_app_initialization() {
    let core = cosmic::app::Core::default();  // May need mock
    let (app, _task) = YourApp::init(core, ());
    
    assert!(app.popup.is_none());
    assert_eq!(app.example_row, false);
}
```

**Acceptance Criteria**:
- [ ] init() method creates YourApp instance
- [ ] popup initialized to None
- [ ] example_row initialized to false
- [ ] Returns (YourApp, Task::none())
- [ ] Compiles and passes test

**Implementation Steps**:
1. Write test for init() (RED)
2. Implement init() method
3. Initialize localization system
4. Create YourApp with default values
5. Return (app, Task::none())
6. Run test (GREEN)

**Dependencies**: Task 3.1

**Red Phase**:
```rust
// Test will fail with "not yet implemented"
```

**Green Phase**:
```rust
fn init(core: cosmic::app::Core, _flags: Self::Flags) -> (Self, cosmic::iced::Task<Self::Message>) {
    crate::core::localization::init();
    
    let app = YourApp {
        core,
        popup: None,
        example_row: false,
    };
    
    (app, cosmic::iced::Task::none())
}
```

---

#### Task 3.3: Implement view() Method (Panel Icon)
**Priority**: High  
**Estimated Time**: 20 minutes

**Test Scenario**:
```rust
// Manual test: Run applet and verify icon appears in panel
// Automated test: Verify method compiles and returns correct type
#[test]
fn test_view_returns_element() {
    // Type checking test - verify view() signature
    // Full test requires COSMIC runtime
}
```

**Acceptance Criteria**:
- [ ] view() method returns icon button Element
- [ ] Uses core.applet.icon_button("display-symbolic")
- [ ] Calls .on_press(Message::TogglePopup)
- [ ] Returns Element<Self::Message>
- [ ] Compiles successfully

**Implementation Steps**:
1. Remove todo!() from view() (RED if removed without implementation)
2. Implement icon_button creation
3. Add on_press handler
4. Convert to Element with .into()
5. Run `cargo check` (GREEN)
6. Manual test: verify icon displays

**Dependencies**: Task 3.2

**Red Phase**:
```bash
cargo check  # Will fail when todo!() is removed
```

**Green Phase**:
```rust
fn view(&self) -> cosmic::prelude::Element<Self::Message> {
    self.core
        .applet
        .icon_button("display-symbolic")
        .on_press(Message::TogglePopup)
        .into()
}
```

---

#### Task 3.4: Implement view_window() Method (Popup Content)
**Priority**: High  
**Estimated Time**: 25 minutes

**Test Scenario**:
```rust
// Manual test: Click icon and verify popup displays
// Automated: Verify method compiles
#[test]
fn test_view_window_compiles() {
    // Type checking - verify signature matches
}
```

**Acceptance Criteria**:
- [ ] view_window() method implemented
- [ ] Creates list_column with padding(5) and spacing(0)
- [ ] Adds settings::item with fl!("example-row")
- [ ] Includes toggler widget bound to example_row state
- [ ] Wraps in core.applet.popup_container()
- [ ] Returns Element<Self::Message>
- [ ] Compiles successfully

**Implementation Steps**:
1. Remove todo!() from view_window() (RED)
2. Create widget::list_column() with settings
3. Add settings::item with translated label
4. Add toggler widget with on_toggle handler
5. Wrap in popup_container()
6. Run `cargo check` (GREEN)
7. Manual test: verify popup renders

**Dependencies**: Task 3.3

**Red Phase**:
```bash
cargo check  # Fails when todo!() removed
```

**Green Phase**:
```rust
fn view_window(&self, _id: cosmic::iced::window::Id) -> cosmic::prelude::Element<Self::Message> {
    use cosmic::widget;
    
    let content_list = widget::list_column()
        .padding(5)
        .spacing(0)
        .add(cosmic::widget::settings::item(
            &fl!("example-row"),
            widget::toggler(self.example_row)
                .on_toggle(Message::ToggleExampleRow),
        ));
    
    self.core.applet.popup_container(content_list).into()
}
```

---

#### Task 3.5: Implement update() Method (Message Handling)
**Priority**: High  
**Estimated Time**: 35 minutes

**Test Scenario**:
```rust
#[test]
fn test_update_toggle_popup_creates_popup() {
    let mut app = create_test_app();
    assert!(app.popup.is_none());
    
    let _task = app.update(Message::TogglePopup);
    // After first toggle, popup ID should be set
    // (Task execution happens in runtime, not testable directly)
}

#[test]
fn test_update_toggle_example_row() {
    let mut app = create_test_app();
    assert_eq!(app.example_row, false);
    
    app.update(Message::ToggleExampleRow(true));
    assert_eq!(app.example_row, true);
}
```

**Acceptance Criteria**:
- [ ] update() handles Message::TogglePopup
  - If popup exists: destroys popup and sets self.popup = None
  - If no popup: creates popup with get_popup and stores ID
- [ ] update() handles Message::PopupClosed(id)
  - Sets self.popup = None if ID matches
- [ ] update() handles Message::ToggleExampleRow(bool)
  - Updates self.example_row state
- [ ] Returns appropriate Task for each message
- [ ] Tests pass

**Implementation Steps**:
1. Write tests for each message handler (RED)
2. Implement match statement on message
3. Implement TogglePopup logic with popup creation/destruction
4. Implement PopupClosed logic
5. Implement ToggleExampleRow logic
6. Run tests (GREEN)
7. Refactor for clarity

**Dependencies**: Task 3.4

**Red Phase**:
```rust
// Tests will fail with "not yet implemented"
```

**Green Phase**:
```rust
fn update(&mut self, message: Self::Message) -> cosmic::iced::Task<Self::Message> {
    use cosmic::iced_winit::commands::popup::{destroy_popup, get_popup};
    
    match message {
        Message::TogglePopup => {
            if let Some(id) = self.popup.take() {
                // Popup exists, destroy it
                destroy_popup(id)
            } else {
                // No popup, create one
                let new_id = cosmic::iced::window::Id::unique();
                self.popup = Some(new_id);
                
                let mut popup_settings = self.core.applet.get_popup_settings(
                    cosmic::iced::window::Id::MAIN,
                    new_id,
                    None,
                    None,
                    None,
                );
                
                popup_settings.positioner.size_limits = cosmic::iced::Limits::NONE
                    .max_width(372.0)
                    .min_width(300.0)
                    .min_height(200.0)
                    .max_height(1080.0);
                
                get_popup(popup_settings)
            }
        }
        Message::PopupClosed(id) => {
            if self.popup == Some(id) {
                self.popup = None;
            }
            cosmic::iced::Task::none()
        }
        Message::ToggleExampleRow(value) => {
            self.example_row = value;
            cosmic::iced::Task::none()
        }
    }
}
```

---

#### Task 3.6: Implement Additional Trait Methods
**Priority**: High  
**Estimated Time**: 15 minutes

**Test Scenario**:
```bash
cargo check
cargo clippy
```

**Acceptance Criteria**:
- [ ] on_close_requested() implemented
  - Returns Message::PopupClosed(id) for popup windows
  - Returns None for main window
- [ ] style() implemented
  - Returns Some(cosmic::applet::style())
- [ ] Compiles successfully
- [ ] cargo clippy passes

**Implementation Steps**:
1. Implement on_close_requested() (RED - missing method)
2. Check if window ID matches popup
3. Return appropriate message
4. Implement style() method
5. Run `cargo check` (GREEN)
6. Verify with clippy

**Dependencies**: Task 3.5

**Red Phase**:
```bash
cargo check  # May warn about missing trait methods
```

**Green Phase**:
```rust
fn on_close_requested(&self, id: cosmic::iced::window::Id) -> Option<Self::Message> {
    if self.popup == Some(id) {
        Some(Message::PopupClosed(id))
    } else {
        None
    }
}

fn style(&self) -> Option<cosmic::iced::Application> {
    Some(cosmic::applet::style())
}
```

---

### Phase 4: Main Entry Point

#### Task 4.1: Implement main() Function
**Priority**: High  
**Estimated Time**: 10 minutes

**Test Scenario**:
```bash
cargo build
# Verify binary is created
ls target/debug/cosmic-applet-copilot-quota-tracker
```

**Acceptance Criteria**:
- [ ] main.rs contains main() function
- [ ] Calls cosmic::applet::run::<YourApp>()
- [ ] Includes proper error handling (.expect())
- [ ] Compiles successfully
- [ ] Binary is created

**Implementation Steps**:
1. Write main() function (RED - missing)
2. Import YourApp from app module
3. Call cosmic::applet::run::<YourApp>(true, ())
4. Add .expect() for error handling
5. Run `cargo build` (GREEN)

**Dependencies**: Task 3.6

**Red Phase**:
```bash
cargo build  # Fails without main() function
```

**Green Phase**:
```rust
// src/main.rs
mod app;
mod core;

use app::YourApp;

fn main() -> cosmic::iced::Result {
    cosmic::applet::run::<YourApp>(true, ())
}
```

---

### Phase 5: Resource Files

#### Task 5.1: Create Desktop Entry File
**Priority**: Medium  
**Estimated Time**: 10 minutes

**Test Scenario**:
```bash
# Verify desktop file syntax
desktop-file-validate res/com.example.CosmicAppletTemplate.desktop
```

**Acceptance Criteria**:
- [ ] Desktop file exists at res/com.example.CosmicAppletTemplate.desktop
- [ ] Type = Application
- [ ] Categories = System;Utility
- [ ] NoDisplay = true
- [ ] StartupNotify = false
- [ ] X-CosmicApplet = true
- [ ] Passes desktop-file-validate

**Implementation Steps**:
1. Create res/ directory
2. Create desktop file
3. Add required keys
4. Validate with desktop-file-validate
5. Commit file

**Dependencies**: None

---

#### Task 5.2: Create MetaInfo XML File
**Priority**: Medium  
**Estimated Time**: 15 minutes

**Test Scenario**:
```bash
# Verify metainfo syntax
appstreamcli validate res/com.example.CosmicAppletTemplate.metainfo.xml
```

**Acceptance Criteria**:
- [ ] MetaInfo file exists at res/com.example.CosmicAppletTemplate.metainfo.xml
- [ ] Component type = desktop-application
- [ ] Contains name, summary, description
- [ ] Contains license information
- [ ] Passes appstream validation

**Implementation Steps**:
1. Create metainfo.xml file
2. Add AppStream metadata
3. Validate with appstreamcli
4. Commit file

**Dependencies**: None

---

#### Task 5.3: Add Application Icons
**Priority**: Low  
**Estimated Time**: 20 minutes

**Test Scenario**:
```bash
# Verify icons exist
for size in 16 24 32 48 64 128 256; do
  ls res/icons/hicolor/${size}x${size}/apps/com.example.CosmicAppletTemplate.svg
done
```

**Acceptance Criteria**:
- [ ] Icon directories created for all required sizes
- [ ] SVG icons placed in each directory
- [ ] Icons follow hicolor theme structure
- [ ] All files named consistently

**Implementation Steps**:
1. Create directory structure: res/icons/hicolor/{size}x{size}/apps/
2. Create or copy SVG icon files
3. Verify all sizes present
4. Commit files

**Dependencies**: None

---

### Phase 6: Build System & Documentation

#### Task 6.1: Create Justfile
**Priority**: Low  
**Estimated Time**: 15 minutes

**Test Scenario**:
```bash
# Verify justfile commands work
just build
just run
just clean
```

**Acceptance Criteria**:
- [ ] justfile exists in project root
- [ ] Contains commands: build, run, install, clean
- [ ] All commands execute successfully
- [ ] Install command copies files to correct locations

**Implementation Steps**:
1. Create justfile
2. Add build recipe (cargo build --release)
3. Add run recipe (cargo run)
4. Add install recipe (copy binaries and resources)
5. Add clean recipe (cargo clean)
6. Test each command

**Dependencies**: Task 4.1

---

#### Task 6.2: Write README Documentation
**Priority**: Medium  
**Estimated Time**: 20 minutes

**Test Scenario**:
```bash
# Verify README exists and is readable
cat README.md
# Follow instructions manually
```

**Acceptance Criteria**:
- [ ] README.md exists
- [ ] Contains project description
- [ ] Contains build requirements (Rust 1.80+, COSMIC desktop)
- [ ] Contains build instructions
- [ ] Contains installation instructions
- [ ] Contains usage instructions
- [ ] Contains license information

**Implementation Steps**:
1. Create README.md
2. Add project overview
3. Document dependencies
4. Add build steps with examples
5. Add installation steps
6. Add usage instructions
7. Review for clarity

**Dependencies**: None

---

### Phase 7: Integration Testing

#### Task 7.1: Manual Integration Test - Applet Loading
**Priority**: High  
**Estimated Time**: 15 minutes

**Test Scenario**:
```bash
# Build and run applet
cargo build --release
# Install to system
sudo cp target/release/cosmic-applet-copilot-quota-tracker /usr/bin/
sudo cp res/*.desktop /usr/share/applications/
sudo update-desktop-database
# Restart COSMIC panel or log out/in
```

**Acceptance Criteria**:
- [ ] Applet icon appears in COSMIC panel
- [ ] Icon displays correctly (not broken/missing)
- [ ] Applet does not crash on load
- [ ] No error messages in journal

**Test Steps**:
1. Build release binary
2. Install to system locations
3. Restart COSMIC panel
4. Verify icon appears
5. Check system logs for errors

**Dependencies**: Task 4.1, Task 5.1, Task 5.2

---

#### Task 7.2: Manual Integration Test - Popup Interaction
**Priority**: High  
**Estimated Time**: 10 minutes

**Test Scenario**:
```
1. Click applet icon in panel
2. Verify popup window appears
3. Verify popup contains example toggle row
4. Toggle the example row switch
5. Click outside popup
6. Verify popup closes
7. Click icon again
8. Verify popup reopens
```

**Acceptance Criteria**:
- [ ] Clicking icon opens popup
- [ ] Popup displays with correct styling
- [ ] Popup contains translated "Example Row" text
- [ ] Toggler responds to clicks
- [ ] Clicking outside closes popup
- [ ] Re-clicking icon reopens popup
- [ ] No crashes or errors

**Test Steps**:
1. Follow test scenario steps
2. Check for visual glitches
3. Verify translations display
4. Monitor system logs

**Dependencies**: Task 7.1

---

#### Task 7.3: Manual Integration Test - Internationalization
**Priority**: Medium  
**Estimated Time**: 10 minutes

**Test Scenario**:
```bash
# Change system locale to Dutch
localectl set-locale LANG=nl_NL.UTF-8
# Restart COSMIC session
# Open applet popup and verify text is in Dutch
```

**Acceptance Criteria**:
- [ ] Applet respects system locale setting
- [ ] English translations display correctly
- [ ] Dutch translations display correctly
- [ ] No missing translation warnings in logs

**Test Steps**:
1. Set system locale to English, verify applet
2. Set system locale to Dutch, verify applet
3. Check logs for i18n errors

**Dependencies**: Task 7.2

---

## Summary

### Task Execution Order
1. **Phase 1** (Foundation): Tasks 1.1 → 1.2 → 1.3
2. **Phase 2** (Core Structure): Tasks 2.1 → 2.2 → 2.3
3. **Phase 3** (Application Trait): Tasks 3.1 → 3.2 → 3.3 → 3.4 → 3.5 → 3.6
4. **Phase 4** (Entry Point): Task 4.1
5. **Phase 5** (Resources): Tasks 5.1, 5.2, 5.3 (parallel)
6. **Phase 6** (Build/Docs): Tasks 6.1, 6.2 (parallel)
7. **Phase 7** (Integration): Tasks 7.1 → 7.2 → 7.3

### Estimated Total Time
- Phase 1: 50 minutes
- Phase 2: 50 minutes
- Phase 3: 155 minutes (2h 35m)
- Phase 4: 10 minutes
- Phase 5: 45 minutes
- Phase 6: 35 minutes
- Phase 7: 35 minutes

**Total: ~380 minutes (~6.3 hours)**

### Critical Path
Tasks that block others:
- 1.1 → 1.2 (must have project before dependencies)
- 3.1 → 3.2 → 3.3 → 3.4 → 3.5 → 3.6 (sequential trait implementation)
- 3.6 → 4.1 (must have Application trait before main)
- 4.1 → 7.1 (must compile before integration testing)

### Testing Strategy Summary
- **Compilation Tests**: Every task verifies `cargo check` passes
- **Unit Tests**: Where possible (Message enum, state initialization)
- **Integration Tests**: Manual testing in COSMIC desktop (Tasks 7.x)
- **Validation Tools**: desktop-file-validate, appstreamcli, cargo clippy

### Success Metrics
- [ ] All cargo check/build commands succeed
- [ ] Zero compiler warnings
- [ ] Zero clippy warnings
- [ ] Applet loads in COSMIC panel
- [ ] All manual integration tests pass
- [ ] Code follows libcosmic patterns
- [ ] Documentation is clear and accurate

---

## Next Steps

After task completion:
1. Commit all changes with meaningful messages
2. Create git tag: `v0.1.0-foundation`
3. Proceed to Feature 02: Configuration & Authentication
4. Begin adapting template for quota tracker functionality

## Notes for TDD Implementation

- **Start Small**: Implement one task at a time
- **Run Tests Frequently**: After each change, run `cargo check`
- **Commit Often**: Commit after each green phase
- **Refactor Continuously**: Clean up code structure while tests pass
- **Manual Testing**: Schedule Phase 7 when COSMIC desktop is available
- **Documentation**: Update README as features are implemented
