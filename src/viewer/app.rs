// SPDX-License-Identifier: GPL-3.0-only

//! Viewer application core logic and COSMIC Application trait implementation.

use crate::core::database::{repository::UsageRepository, DatabaseManager};
use cosmic::{app::Core, Application, Element};
use std::sync::Arc;

/// Messages that can be sent within the viewer application.
#[derive(Debug, Clone)]
pub enum Message {
    /// Exit the application
    Exit,
}

/// The main viewer application structure.
pub struct ViewerApp {
    core: Core,
    database_manager: Arc<DatabaseManager>,
    repository: Arc<UsageRepository>,
}

impl Application for ViewerApp {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = "com.vasilvestre.CosmicAppletOpencodeUsageViewer";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(mut core: Core, _flags: Self::Flags) -> (Self, cosmic::app::Task<Self::Message>) {
        // Initialize database
        let database_manager = match DatabaseManager::new() {
            Ok(manager) => Arc::new(manager),
            Err(e) => {
                eprintln!("Failed to initialize database: {e}");
                // For now, we'll panic. In production, we'd show an error dialog
                panic!("Database initialization failed: {e}");
            }
        };

        // Create repository
        let repository = Arc::new(UsageRepository::new(Arc::clone(&database_manager)));

        // Configure window title
        core.window.header_title = "OpenCode Usage History".to_string();

        let app = Self {
            core,
            database_manager,
            repository,
        };

        (app, cosmic::app::Task::none())
    }

    fn update(&mut self, message: Self::Message) -> cosmic::app::Task<Self::Message> {
        match message {
            Message::Exit => {
                // Close the window by returning exit task
                std::process::exit(0);
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        crate::viewer::ui::view_content()
    }
}

impl ViewerApp {
    /// Gets a reference to the database manager.
    #[must_use]
    pub fn database_manager(&self) -> &Arc<DatabaseManager> {
        &self.database_manager
    }

    /// Gets a reference to the usage repository.
    #[must_use]
    pub fn repository(&self) -> &Arc<UsageRepository> {
        &self.repository
    }

    /// Creates a new `ViewerApp` for testing purposes.
    #[cfg(test)]
    pub fn new_for_test(
        core: Core,
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    #[allow(clippy::no_effect_underscore_binding)]
    fn test_message_exit_variant_exists() {
        // Verify Message::Exit variant exists and can be constructed
        let _msg = Message::Exit;
        // This test validates the message type compiles
    }

    #[test]
    fn test_viewer_app_has_required_fields() {
        // Create temporary database
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let database_manager = Arc::new(DatabaseManager::new_with_path(&db_path).unwrap());
        let _repository = Arc::new(UsageRepository::new(Arc::clone(&database_manager)));

        // Test that the struct can be defined with these types
        let _: fn(Core, Arc<DatabaseManager>, Arc<UsageRepository>) -> ViewerApp =
            ViewerApp::new_for_test;
    }

    #[test]
    fn test_app_id_constant() {
        assert_eq!(
            ViewerApp::APP_ID,
            "com.vasilvestre.CosmicAppletOpencodeUsageViewer"
        );
    }
}
