# Viewer App Scaffolding - Technical Design

## Architecture Overview

The viewer application will be a standalone COSMIC application that shares the core database infrastructure with the existing applet. It will follow the libcosmic application pattern rather than the applet pattern.

## Component Structure

```
src/
├── viewer/
│   ├── mod.rs          # Module exports and shared types
│   ├── main.rs         # Entry point (cosmic::applet::run())
│   ├── app.rs          # ViewerApp struct and Application trait impl
│   └── ui.rs           # UI rendering logic
```

## Core Components

### 1. ViewerApp Structure

```rust
pub struct ViewerApp {
    core: cosmic::app::Core,
    database_manager: Arc<DatabaseManager>,
    repository: Arc<UsageRepository>,
    // Future: UI state for data display
}
```

**Responsibilities:**
- Manage application lifecycle
- Hold references to shared database components
- Implement cosmic::Application trait
- Handle window configuration

### 2. Application Trait Implementation

The ViewerApp will implement `cosmic::Application` (not `cosmic::applet::Application`):

```rust
impl cosmic::Application for ViewerApp {
    type Message = Message;
    type Executor = cosmic::executor::Default;
    type Flags = ();
    
    const APP_ID: &'static str = "com.vasilvestre.CosmicAppletOpencodeUsageViewer";
    
    fn core(&self) -> &Core { &self.core }
    fn core_mut(&mut self) -> &mut Core { &mut self.core }
    
    fn init(core: Core, _flags: Self::Flags) -> (Self, Command<Self::Message>);
    fn update(&mut self, message: Self::Message) -> Command<Self::Message>;
    fn view(&self) -> Element<Self::Message>;
    fn header_start(&self) -> Vec<Element<Self::Message>>;
    fn header_end(&self) -> Vec<Element<Self::Message>>;
}
```

### 3. Message Types

```rust
#[derive(Debug, Clone)]
pub enum Message {
    Exit,
    // Future: data loading, filtering, etc.
}
```

### 4. Main Entry Point

```rust
fn main() -> cosmic::iced::Result {
    cosmic::app::run::<ViewerApp>(
        cosmic::app::Settings::default(),
        ()
    )
}
```

## Database Integration

### Shared Components
- **DatabaseManager**: Manages SQLite connection pool (from existing code)
- **UsageRepository**: Provides data access methods (from existing code)
- **Database Location**: `~/.local/share/cosmic-applet-opencode-usage/usage.db`

### Initialization Flow
1. ViewerApp::init() called by COSMIC runtime
2. Create DatabaseManager instance
3. Run migrations (if needed)
4. Create UsageRepository with Arc<DatabaseManager>
5. Store both in ViewerApp struct

### Concurrency Considerations
- SQLite supports multiple readers simultaneously
- Write locks are handled by SQLite internally
- Both applet and viewer can safely read data concurrently
- DatabaseManager uses connection pooling for efficiency

## UI Design

### Window Configuration
- **Title**: "OpenCode Usage History"
- **Default Size**: 1000x700 pixels
- **Resizable**: Yes
- **Position**: System default (later: remember last position)

### Layout Structure
```
┌─────────────────────────────────────────┐
│ File                              [_][□][X]│ <- Header
├─────────────────────────────────────────┤
│                                         │
│                                         │
│        [Empty content area]             │ <- Main view
│        (Placeholder for data display)   │
│                                         │
│                                         │
└─────────────────────────────────────────┘
```

### Menu Structure
- **File Menu**:
  - Exit (Ctrl+Q)

### Future Additions (not in this feature)
- Data display area
- Filter controls
- Date range selector
- Export options

## Error Handling

### Database Errors
```rust
fn init(core: Core, _flags: Self::Flags) -> (Self, Command<Self::Message>) {
    let database_manager = match DatabaseManager::new() {
        Ok(manager) => Arc::new(manager),
        Err(e) => {
            eprintln!("Failed to initialize database: {}", e);
            // Return app in error state or exit
        }
    };
    // ...
}
```

### Graceful Degradation
- Display error message if database unavailable
- Allow application to close cleanly
- Log errors for debugging

## Testing Strategy

### Unit Tests
1. **test_viewer_app_creation**: Verify ViewerApp can be constructed with valid components
2. **test_database_connection**: Verify DatabaseManager initialization
3. **test_repository_access**: Verify UsageRepository can be created

### Integration Tests
1. **test_viewer_binary_builds**: Verify the binary target compiles
2. **test_app_initialization**: Test full initialization flow
3. **test_no_applet_regression**: Ensure existing applet tests still pass

### Test Mocks
- Use temporary database files for testing
- Mock COSMIC runtime environment where needed
- Isolate tests from system state

## Build Configuration

### Cargo.toml Changes
```toml
[[bin]]
name = "cosmic-applet-opencode-usage-viewer"
path = "src/viewer/main.rs"

# Reuse existing dependencies - no new ones needed
```

### Build Commands
- Build viewer: `cargo build --bin cosmic-applet-opencode-usage-viewer`
- Run viewer: `cargo run --bin cosmic-applet-opencode-usage-viewer`
- Test all: `cargo test`

## Module Integration

### src/lib.rs Updates
```rust
pub mod viewer;  // Add viewer module export
```

### Visibility
- ViewerApp: Public (for testing)
- Message: Public (for testing)
- Internal helpers: Crate-private

## Security Considerations

### Database Access
- Read-only for now (no data modification in viewer)
- Same permissions model as applet
- Database file protected by user's file permissions

### Resource Cleanup
- Properly drop database connections on exit
- No resource leaks in error paths

## Performance Considerations

### Initialization
- Lazy load historical data (not in this feature)
- Connection pooling via DatabaseManager
- Async initialization where appropriate

### Memory Usage
- Minimal at startup (just scaffolding)
- Future: paginate large datasets

## Future Extensibility

This scaffolding provides foundation for:
1. **Historical Data Display** (next feature)
2. **Data Filtering and Search**
3. **Export Functionality**
4. **Settings Panel**
5. **Real-time Updates** (optional)

## Implementation Order (TDD)

1. Write failing test for ViewerApp creation
2. Create ViewerApp struct (minimal)
3. Write failing test for Application trait methods
4. Implement Application trait (minimal)
5. Write failing test for database initialization
6. Implement database initialization
7. Write failing test for UI rendering
8. Implement empty UI view
9. Refactor and clean up
10. Add integration tests

## Dependencies

### Existing (Reused)
- cosmic: UI framework
- tokio: Async runtime
- sqlx: Database access
- thiserror: Error handling

### No New Dependencies Required

## Compliance

- SPDX headers: GPL-3.0-only
- Code formatting: rustfmt
- Linting: clippy pedantic
- Documentation: Inline docs for public items
