// SPDX-License-Identifier: GPL-3.0-only

use cosmic::app::{Core, Task};
use cosmic::iced::window::Id;
use cosmic::iced::Limits;
use cosmic::iced_winit::commands::popup::{destroy_popup, get_popup};
use cosmic::widget::{self, settings};
use cosmic::{Application, Element};

use crate::core::config::AppConfig;
use crate::fl;
use crate::ui::state::AppState;

/// This is our Copilot quota tracker applet structure.
/// This will replace YourApp once we implement the full Application trait (Task 13).
pub struct CopilotMonitorApplet {
    /// Application state which is managed by the COSMIC runtime.
    core: Core,
    /// Our application state containing UI and data state.
    state: AppState,
}

impl CopilotMonitorApplet {
    /// Create a new CopilotMonitorApplet instance.
    /// This is a temporary constructor for testing - the actual initialization
    /// will happen via the Application::init trait method in Task 13.
    pub fn new(config: AppConfig) -> Self {
        Self {
            core: Core::default(),
            state: AppState::new(config),
        }
    }

    /// Handle incoming messages and update application state accordingly.
    pub fn handle_message(&mut self, message: crate::ui::Message) {
        use crate::ui::Message;

        match message {
            Message::FetchMetrics => {
                // Set state to Loading when fetch is triggered
                self.state.panel_state = crate::ui::state::PanelState::Loading;
            }
            Message::MetricsFetched(Ok(usage)) => {
                // Update state with successful data fetch
                self.state.update_success(usage);
            }
            Message::MetricsFetched(Err(error)) => {
                // Update state with error message
                self.state.update_error(error);
            }
            Message::ThemeChanged => {
                // No state changes needed for theme change
            }
            Message::UpdateTooltip => {
                // No state changes needed for tooltip update
            }
        }
    }
}

/// This is the struct that represents your application.
/// It is used to define the data that will be used by your application.
#[derive(Default)]
pub struct YourApp {
    /// Application state which is managed by the COSMIC runtime.
    core: Core,
    /// The popup id.
    popup: Option<Id>,
    /// Example row toggler.
    example_row: bool,
}

/// This is the enum that contains all the possible variants that your application will need to transmit messages.
/// This is used to communicate between the different parts of your application.
/// If your application does not need to send messages, you can use an empty enum or `()`.
#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    ToggleExampleRow(bool),
}

/// Implement the `Application` trait for your application.
/// This is where you define the behavior of your application.
///
/// The `Application` trait requires you to define the following types and constants:
/// - `Executor` is the async executor that will be used to run your application's commands.
/// - `Flags` is the data that your application needs to use before it starts.
/// - `Message` is the enum that contains all the possible variants that your application will need to transmit messages.
/// - `APP_ID` is the unique identifier of your application.
impl Application for YourApp {
    type Executor = cosmic::executor::Default;

    type Flags = ();

    type Message = Message;

    const APP_ID: &'static str = "com.example.CosmicAppletTemplate";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    /// This is the entry point of your application, it is where you initialize your application.
    ///
    /// Any work that needs to be done before the application starts should be done here.
    ///
    /// - `core` is used to passed on for you by libcosmic to use in the core of your own application.
    /// - `flags` is used to pass in any data that your application needs to use before it starts.
    /// - `Command` type is used to send messages to your application. `Command::none()` can be used to send no messages to your application.
    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let app = YourApp {
            core,
            ..Default::default()
        };

        (app, Task::none())
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    /// This is the main view of your application, it is the root of your widget tree.
    ///
    /// The `Element` type is used to represent the visual elements of your application,
    /// it has a `Message` associated with it, which dictates what type of message it can send.
    ///
    /// To get a better sense of which widgets are available, check out the `widget` module.
    fn view(&self) -> Element<'_, Self::Message> {
        self.core
            .applet
            .icon_button("display-symbolic")
            .on_press(Message::TogglePopup)
            .into()
    }

    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        let content_list = widget::list_column()
            .padding(5)
            .spacing(0)
            .add(settings::item(
                fl!("example-row"),
                widget::toggler(self.example_row).on_toggle(Message::ToggleExampleRow),
            ));

        self.core.applet.popup_container(content_list).into()
    }

    /// Application messages are handled here. The application state can be modified based on
    /// what message was received. Commands may be returned for asynchronous execution on a
    /// background thread managed by the application's executor.
    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    let new_id = Id::unique();
                    self.popup.replace(new_id);
                    let mut popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );
                    popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(372.0)
                        .min_width(300.0)
                        .min_height(200.0)
                        .max_height(1080.0);
                    get_popup(popup_settings)
                }
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::ToggleExampleRow(toggled) => self.example_row = toggled,
        }
        Task::none()
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::AppConfig;
    use crate::core::models::{CopilotUsage, UsageBreakdown};
    use crate::ui::state::PanelState;
    use crate::ui::Message;

    fn create_mock_config() -> AppConfig {
        AppConfig {
            organization_name: "test-org".to_string(),
            refresh_interval_seconds: 900,
        }
    }

    fn create_mock_copilot_usage() -> CopilotUsage {
        CopilotUsage {
            total_suggestions_count: 100,
            total_acceptances_count: 50,
            total_lines_suggested: 200,
            total_lines_accepted: 75,
            day: "2025-09-30".to_string(),
            breakdown: vec![UsageBreakdown {
                language: "rust".to_string(),
                editor: "vscode".to_string(),
                suggestions_count: 100,
                acceptances_count: 50,
                lines_suggested: 200,
                lines_accepted: 75,
            }],
        }
    }

    fn create_test_applet() -> CopilotMonitorApplet {
        CopilotMonitorApplet::new(create_mock_config())
    }

    // Task 8 tests
    #[test]
    fn test_applet_initialization() {
        let config = create_mock_config();
        let applet = CopilotMonitorApplet::new(config);
        assert!(matches!(applet.state.panel_state, PanelState::Loading));
    }

    #[test]
    fn test_applet_has_required_fields() {
        let config = create_mock_config();
        let applet = CopilotMonitorApplet::new(config);
        // Verify fields exist (compilation test)
        let _ = applet.core;
        let _ = applet.state;
    }

    // Task 9 tests: Message Handling Logic
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
        let error = "Network timeout".to_string();
        
        applet.handle_message(Message::MetricsFetched(Err(error)));
        
        assert!(matches!(applet.state.panel_state, PanelState::Error(_)));
    }
}
