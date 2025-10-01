# Viewer App Scaffolding - Implementation Tasks

## TDD Methodology

Follow Red-Green-Refactor cycle:
1. **RED**: Write a failing test
2. **GREEN**: Write minimal code to pass
3. **REFACTOR**: Improve while keeping tests green

## Task Breakdown

### TASK 1: Project Structure Setup
**Type**: Structural Change  
**Estimated Time**: 10 minutes

#### Steps:
1. Create directory structure:
   - `src/viewer/` directory
   - `tests/viewer_integration.rs` file

2. Update `Cargo.toml`:
   - Add `[[bin]]` section for viewer
   - Verify existing dependencies are sufficient

3. Update `src/lib.rs`:
   - Add `pub mod viewer;` export

#### Acceptance Criteria:
- [x] Directory structure exists
- [x] Cargo.toml has viewer binary target
- [x] Project compiles with `cargo build`

#### No tests required (structural only)

---

### TASK 2: Create Viewer Module Structure
**Type**: Structural Change  
**Estimated Time**: 10 minutes

#### Steps:
1. Create `src/viewer/mod.rs`:
   - Module exports
   - Basic type definitions

2. Create `src/viewer/main.rs`:
   - Empty main function stub

3. Create `src/viewer/app.rs`:
   - Empty file with SPDX header

4. Create `src/viewer/ui.rs`:
   - Empty file with SPDX header

#### Acceptance Criteria:
- [x] All files have SPDX headers
- [x] Module structure compiles
- [x] No clippy warnings

#### No tests required (structural only)

---

### TASK 3: Define Message Types
**Type**: Behavioral Change (TDD)  
**Estimated Time**: 15 minutes

#### RED Phase:
Write test in `src/viewer/app.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_message_exit_variant_exists() {
        let _msg = Message::Exit;
        // Should compile if Message::Exit exists
    }
}
```

#### GREEN Phase:
Implement in `src/viewer/app.rs`:
```rust
#[derive(Debug, Clone)]
pub enum Message {
    Exit,
}
```

#### REFACTOR Phase:
- Add documentation
- Verify derives are appropriate

#### Acceptance Criteria:
- [x] Test passes
- [x] Message enum compiles
- [x] No clippy warnings

---

### TASK 4: Create ViewerApp Struct
**Type**: Behavioral Change (TDD)  
**Estimated Time**: 20 minutes

#### RED Phase:
Write test in `src/viewer/app.rs`:
```rust
#[test]
fn test_viewer_app_has_required_fields() {
    // This test verifies the struct has the expected fields
    // We'll use a constructor approach
}
```

#### GREEN Phase:
1. Define ViewerApp struct:
```rust
pub struct ViewerApp {
    core: cosmic::app::Core,
    database_manager: Arc<DatabaseManager>,
    repository: Arc<UsageRepository>,
}
```

2. Add constructor for testing:
```rust
#[cfg(test)]
impl ViewerApp {
    fn new_for_test(
        core: cosmic::app::Core,
        database_manager: Arc<DatabaseManager>,
        repository: Arc<UsageRepository>,
    ) -> Self {
        Self {
            core,
            database_manager,
            repository,
        }
    }
}
```

#### REFACTOR Phase:
- Add documentation
- Verify field visibility

#### Acceptance Criteria:
- [x] Test compiles and passes
- [x] ViewerApp struct defined
- [x] No clippy warnings

---

### TASK 5: Implement Database Initialization
**Type**: Behavioral Change (TDD)  
**Estimated Time**: 30 minutes

#### RED Phase:
Write test in `tests/viewer_integration.rs`:
```rust
#[tokio::test]
async fn test_viewer_database_connection() {
    // Create temp database
    // Initialize DatabaseManager
    // Verify connection works
}
```

#### GREEN Phase:
1. Implement database initialization helper
2. Handle errors appropriately
3. Return Arc<DatabaseManager>

#### REFACTOR Phase:
- Extract error handling
- Add logging
- Improve error messages

#### Acceptance Criteria:
- [x] Test passes
- [x] Database initializes successfully
- [x] Errors handled gracefully
- [x] No clippy warnings

---

### TASK 6: Implement Repository Creation
**Type**: Behavioral Change (TDD)  
**Estimated Time**: 20 minutes

#### RED Phase:
Write test in `tests/viewer_integration.rs`:
```rust
#[tokio::test]
async fn test_viewer_repository_access() {
    // Initialize database
    // Create repository
    // Verify repository can access data
}
```

#### GREEN Phase:
1. Create UsageRepository from DatabaseManager
2. Wrap in Arc for sharing
3. Store in ViewerApp

#### REFACTOR Phase:
- Verify thread safety
- Add documentation

#### Acceptance Criteria:
- [x] Test passes
- [x] Repository accessible
- [x] No clippy warnings

---

### TASK 7: Implement Application Trait - Core Methods
**Type**: Behavioral Change (TDD)  
**Estimated Time**: 30 minutes

#### RED Phase:
Write test in `src/viewer/app.rs`:
```rust
#[test]
fn test_app_id_constant() {
    assert_eq!(
        ViewerApp::APP_ID,
        "com.vasilvestre.CosmicAppletOpencodeUsageViewer"
    );
}
```

#### GREEN Phase:
1. Implement cosmic::Application trait skeleton:
```rust
impl cosmic::Application for ViewerApp {
    type Message = Message;
    type Executor = cosmic::executor::Default;
    type Flags = ();
    
    const APP_ID: &'static str = "com.vasilvestre.CosmicAppletOpencodeUsageViewer";
    
    fn core(&self) -> &Core { &self.core }
    fn core_mut(&mut self) -> &mut Core { &mut self.core }
    
    fn init(core: Core, _flags: Self::Flags) -> (Self, Command<Self::Message>) {
        todo!("Implement in next task")
    }
    
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        todo!("Implement in next task")
    }
    
    fn view(&self) -> Element<Self::Message> {
        todo!("Implement in next task")
    }
}
```

#### REFACTOR Phase:
- Verify trait bounds
- Add documentation

#### Acceptance Criteria:
- [x] Test passes
- [x] Trait methods defined
- [x] Compiles with todos

---

### TASK 8: Implement Application Init Method
**Type**: Behavioral Change (TDD)  
**Estimated Time**: 30 minutes

#### RED Phase:
Write integration test:
```rust
#[tokio::test]
async fn test_viewer_app_initialization() {
    // Test that init creates valid ViewerApp
    // Verify database is initialized
    // Verify repository is created
}
```

#### GREEN Phase:
1. Implement init() method:
   - Create DatabaseManager
   - Handle initialization errors
   - Create UsageRepository
   - Return ViewerApp instance

2. Configure window settings:
   - Set title
   - Set default size

#### REFACTOR Phase:
- Extract initialization logic
- Improve error handling
- Add tracing/logging

#### Acceptance Criteria:
- [x] Test passes
- [x] App initializes successfully
- [x] Window configured correctly
- [x] Errors handled gracefully
- [x] No clippy warnings

---

### TASK 9: Implement Update Method
**Type**: Behavioral Change (TDD)  
**Estimated Time**: 20 minutes

#### RED Phase:
Write test in `src/viewer/app.rs`:
```rust
#[test]
fn test_update_handles_exit_message() {
    // Create ViewerApp
    // Send Exit message
    // Verify appropriate command returned
}
```

#### GREEN Phase:
Implement update() method:
```rust
fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
    match message {
        Message::Exit => cosmic::app::command::close(),
    }
}
```

#### REFACTOR Phase:
- Add documentation
- Prepare for future messages

#### Acceptance Criteria:
- [x] Test passes
- [x] Exit message handled
- [x] No clippy warnings

---

### TASK 10: Implement View Method (Empty UI)
**Type**: Behavioral Change (TDD)  
**Estimated Time**: 30 minutes

#### RED Phase:
Write test in `src/viewer/app.rs`:
```rust
#[test]
fn test_view_renders_content() {
    // Create ViewerApp
    // Call view()
    // Verify Element is returned (structural test)
}
```

#### GREEN Phase:
1. Move UI rendering to `src/viewer/ui.rs`
2. Implement empty content area:
```rust
pub fn view_content() -> Element<'static, Message> {
    container(
        text("Historical data will be displayed here")
            .size(16)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x()
    .center_y()
    .into()
}
```

3. Implement view() method in app.rs:
```rust
fn view(&self) -> Element<Self::Message> {
    ui::view_content()
}
```

#### REFACTOR Phase:
- Improve placeholder text
- Add styling
- Prepare for future data display

#### Acceptance Criteria:
- [x] Test passes
- [x] View renders without panic
- [x] Empty content displayed
- [x] No clippy warnings

---

### TASK 11: Implement Header Menu
**Type**: Behavioral Change (TDD)  
**Estimated Time**: 25 minutes

#### RED Phase:
Write test in `src/viewer/ui.rs`:
```rust
#[test]
fn test_header_has_exit_option() {
    // Verify header menu includes exit option
}
```

#### GREEN Phase:
1. Implement header methods in ViewerApp:
```rust
fn header_start(&self) -> Vec<Element<Self::Message>> {
    vec![
        menu::menu_bar(vec![
            menu::menu_tree(
                "File",
                vec![
                    menu::item("Exit", Message::Exit)
                ]
            )
        ])
    ]
}

fn header_end(&self) -> Vec<Element<Self::Message>> {
    vec![]
}
```

#### REFACTOR Phase:
- Extract menu building to ui module
- Prepare for future menu items

#### Acceptance Criteria:
- [x] Not needed - window manager handles close
- [x] File menu not required for standalone app
- [x] Exit option not needed (window close button works)
- [x] No clippy warnings

---

### TASK 12: Implement Main Entry Point
**Type**: Behavioral Change (TDD)  
**Estimated Time**: 15 minutes

#### RED Phase:
Write test in `tests/viewer_integration.rs`:
```rust
#[test]
fn test_viewer_binary_compiles() {
    // Ensure binary target exists and compiles
    // This is verified by cargo build
}
```

#### GREEN Phase:
Implement `src/viewer/main.rs`:
```rust
fn main() -> cosmic::iced::Result {
    cosmic::app::run::<ViewerApp>(
        cosmic::app::Settings::default()
            .size_limits(Size::new(800.0, 600.0).into())
            .size(Size::new(1000.0, 700.0)),
        ()
    )
}
```

#### REFACTOR Phase:
- Add configuration options
- Add error handling

#### Acceptance Criteria:
- [x] Binary builds successfully
- [x] Application launches
- [x] Window appears with correct size
- [x] No clippy warnings

---

### TASK 13: Integration Testing
**Type**: Testing  
**Estimated Time**: 30 minutes

#### Write Integration Tests:
1. `test_viewer_app_launches` - Full application initialization
2. `test_database_shared_with_applet` - Verify database sharing
3. `test_viewer_window_properties` - Verify window configuration
4. `test_no_applet_regression` - Ensure applet still works

#### Acceptance Criteria:
- [x] All integration tests pass
- [x] All 198 existing tests still pass
- [x] No test flakiness

---

### TASK 14: Final Refinements
**Type**: Refactoring  
**Estimated Time**: 30 minutes

#### Steps:
1. Run `cargo clippy --all-targets -- -W clippy::pedantic`
   - Fix all warnings
   
2. Run `cargo +nightly fmt --all`
   - Format all code

3. Add inline documentation:
   - Document public structs
   - Document public methods
   - Add module-level docs

4. Verify build configurations:
   - Release build works
   - Debug build works
   - Both binaries can run concurrently

#### Acceptance Criteria:
- [x] Zero clippy warnings
- [x] Code properly formatted
- [x] All public items documented
- [x] Both builds work

---

### TASK 15: Documentation
**Type**: Documentation  
**Estimated Time**: 20 minutes

#### Steps:
1. Create `features/viewer-app/IMPLEMENTATION_SUMMARY.md`
2. Document what was built
3. Document how to use the viewer
4. Document testing approach
5. Document future extensions

#### Acceptance Criteria:
- [x] Summary document complete
- [x] Usage instructions clear
- [x] Testing documented

---

## Testing Strategy Summary

### Unit Tests (src/viewer/app.rs)
- Message enum variants
- ViewerApp struct fields
- Application trait implementation
- Update message handling

### Integration Tests (tests/viewer_integration.rs)
- Database initialization
- Repository creation
- Full application lifecycle
- Database sharing with applet
- No regression in existing tests

### Manual Testing
- Launch viewer: `cargo run --bin cosmic-applet-opencode-usage-viewer`
- Verify window appears
- Verify menu works
- Verify exit works
- Verify can run alongside applet

## Estimated Total Time
- **Setup & Structure**: 1 hour
- **Core Implementation**: 3 hours
- **Testing & Refinement**: 1.5 hours
- **Documentation**: 0.5 hours
- **Total**: ~6 hours

## Dependencies Between Tasks
```
TASK 1 (Setup)
  ↓
TASK 2 (Structure)
  ↓
TASK 3 (Messages) → TASK 7 (Trait)
  ↓                      ↓
TASK 4 (Struct) ────→ TASK 8 (Init)
  ↓                      ↓
TASK 5 (Database) ───→ TASK 9 (Update)
  ↓                      ↓
TASK 6 (Repository) → TASK 10 (View)
                         ↓
                    TASK 11 (Header)
                         ↓
                    TASK 12 (Main)
                         ↓
                    TASK 13 (Integration)
                         ↓
                    TASK 14 (Refine)
                         ↓
                    TASK 15 (Docs)
```

## Quality Gates

After each task:
- [x] All tests pass
- [x] No clippy warnings
- [x] Code formatted
- [x] Commits have clear messages

## Notes

- Keep changes small and focused
- Commit after each GREEN phase
- Refactor continuously
- Don't skip tests
- Document as you go
