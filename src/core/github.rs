// SPDX-License-Identifier: GPL-3.0-only

//! GitHub API client for fetching Copilot metrics
//!
//! This module provides types and functionality for interacting with the
//! GitHub Copilot Metrics API.

use serde::{Deserialize, Serialize};

/// Represents code completion metrics from GitHub Copilot IDE
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CopilotIdeCodeCompletions {
    /// Total number of users who engaged with code completions
    pub total_engaged_users: u32,
    /// Breakdown by programming language
    pub languages: Vec<LanguageBreakdown>,
    /// Breakdown by editor
    pub editors: Vec<EditorBreakdown>,
}

/// Breakdown of metrics by programming language
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LanguageBreakdown {
    /// Programming language name (e.g., "rust", "python")
    pub name: String,
    /// Number of users who engaged with this language
    pub total_engaged_users: u32,
    /// Total number of code suggestions made
    pub total_code_suggestions: u32,
    /// Total number of code suggestions accepted
    pub total_code_acceptances: u32,
    /// Total lines of code suggested
    pub total_code_lines_suggested: u32,
    /// Total lines of code accepted
    pub total_code_lines_accepted: u32,
}

/// Breakdown of metrics by editor
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EditorBreakdown {
    /// Editor name (e.g., "vscode", "neovim")
    pub name: String,
    /// Number of users who engaged with this editor
    pub total_engaged_users: u32,
    /// Total number of code suggestions made
    pub total_code_suggestions: u32,
    /// Total number of code suggestions accepted
    pub total_code_acceptances: u32,
    /// Total lines of code suggested
    pub total_code_lines_suggested: u32,
    /// Total lines of code accepted
    pub total_code_lines_accepted: u32,
    /// Models used with this editor
    pub models: Vec<ModelBreakdown>,
}

/// Breakdown of metrics by AI model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelBreakdown {
    /// Model name
    pub name: String,
    /// Whether this is a custom model
    #[serde(default)]
    pub is_custom_model: bool,
    /// Custom model training date (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_model_training_date: Option<String>,
    /// Number of users engaged with this model
    pub total_engaged_users: u32,
    /// Total number of code suggestions made
    pub total_code_suggestions: u32,
    /// Total number of code suggestions accepted
    pub total_code_acceptances: u32,
    /// Total lines of code suggested
    pub total_code_lines_suggested: u32,
    /// Total lines of code accepted
    pub total_code_lines_accepted: u32,
}

/// Represents IDE chat metrics from GitHub Copilot
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CopilotIdeChat {
    /// Total number of users who engaged with IDE chat
    pub total_engaged_users: u32,
    /// Breakdown by editor
    pub editors: Vec<ChatEditorBreakdown>,
}

/// Breakdown of chat metrics by editor
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatEditorBreakdown {
    /// Editor name (e.g., "vscode", "neovim")
    pub name: String,
    /// Number of users who engaged with this editor
    pub total_engaged_users: u32,
    /// Models used for chat in this editor
    pub models: Vec<ChatModelBreakdown>,
}

/// Breakdown of chat metrics by AI model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatModelBreakdown {
    /// Model name
    pub name: String,
    /// Whether this is a custom model
    #[serde(default)]
    pub is_custom_model: bool,
    /// Custom model training date (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_model_training_date: Option<String>,
    /// Number of users engaged with this model
    pub total_engaged_users: u32,
    /// Total number of chats
    pub total_chats: u32,
    /// Total number of chat insertion events
    pub total_chat_insertion_events: u32,
    /// Total number of chat copy events
    pub total_chat_copy_events: u32,
}

/// Represents Dotcom chat metrics from GitHub Copilot
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CopilotDotcomChat {
    /// Total number of users who engaged with dotcom chat
    pub total_engaged_users: u32,
    /// Models used for chat
    pub models: Vec<DotcomChatModelBreakdown>,
}

/// Breakdown of dotcom chat metrics by AI model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DotcomChatModelBreakdown {
    /// Model name
    pub name: String,
    /// Whether this is a custom model
    #[serde(default)]
    pub is_custom_model: bool,
    /// Custom model training date (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_model_training_date: Option<String>,
    /// Number of users engaged with this model
    pub total_engaged_users: u32,
    /// Total number of chats
    pub total_chats: u32,
}

/// Represents pull request summary metrics from GitHub Copilot
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CopilotDotcomPullRequests {
    /// Total number of users who engaged with PR summaries
    pub total_engaged_users: u32,
    /// Breakdown by repository
    pub repositories: Vec<RepositoryBreakdown>,
}

/// Breakdown of PR summary metrics by repository
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RepositoryBreakdown {
    /// Repository name
    pub name: String,
    /// Number of users who engaged with this repository
    pub total_engaged_users: u32,
    /// Models used for PR summaries in this repository
    pub models: Vec<PullRequestModelBreakdown>,
}

/// Breakdown of PR summary metrics by AI model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PullRequestModelBreakdown {
    /// Model name
    pub name: String,
    /// Whether this is a custom model
    #[serde(default)]
    pub is_custom_model: bool,
    /// Custom model training date (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_model_training_date: Option<String>,
    /// Number of users engaged with this model
    pub total_engaged_users: u32,
    /// Total number of PR summaries created
    pub total_pr_summaries_created: u32,
}

/// Represents a single day's worth of GitHub Copilot metrics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GitHubMetricsDay {
    /// Date in YYYY-MM-DD format
    pub date: String,
    /// Total number of users with active Copilot licenses
    pub total_active_users: u32,
    /// Total number of users who engaged with Copilot features
    pub total_engaged_users: u32,
    /// Code completion metrics
    pub copilot_ide_code_completions: CopilotIdeCodeCompletions,
    /// IDE chat metrics (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copilot_ide_chat: Option<CopilotIdeChat>,
    /// Dotcom chat metrics (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copilot_dotcom_chat: Option<CopilotDotcomChat>,
    /// Pull request summary metrics (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copilot_dotcom_pull_requests: Option<CopilotDotcomPullRequests>,
}

/// HTTP client for GitHub API interactions
pub struct GitHubClient {
    pub(crate) client: reqwest::Client,
    pub(crate) token: String,
    pub(crate) base_url: String,
}

impl GitHubClient {
    /// Creates a new GitHub API client with the given personal access token
    ///
    /// # Arguments
    /// * `token` - GitHub Personal Access Token with appropriate permissions
    ///
    /// # Returns
    /// * `Result<Self, reqwest::Error>` - The client or an error if header configuration fails
    ///
    /// # Example
    /// ```
    /// use cosmic_applet_template::core::github::GitHubClient;
    /// let client = GitHubClient::new("ghp_your_token_here".to_string())
    ///     .expect("Failed to create GitHub client");
    /// ```
    pub fn new(token: String) -> Result<Self, reqwest::Error> {
        let mut headers = reqwest::header::HeaderMap::new();
        
        // Add Authorization header with Bearer token
        let auth_value = format!("Bearer {}", token);
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&auth_value)
                .expect("Invalid characters in token"),
        );
        
        // Add User-Agent header (GitHub API best practice)
        headers.insert(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_static("cosmic-applet-copilot-quota-tracker"),
        );
        
        // Build client with default headers
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;
        
        Ok(Self {
            client,
            token,
            base_url: "https://api.github.com".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_github_metrics_response_minimal() {
        // RED: This test will fail because we haven't defined the structs yet
        let json = r#"[
            {
                "date": "2025-09-30",
                "total_active_users": 10,
                "total_engaged_users": 8,
                "copilot_ide_code_completions": {
                    "total_engaged_users": 8,
                    "languages": [],
                    "editors": []
                }
            }
        ]"#;

        let response: Vec<GitHubMetricsDay> = serde_json::from_str(json).unwrap();
        assert_eq!(response.len(), 1);
        assert_eq!(response[0].date, "2025-09-30");
        assert_eq!(response[0].total_active_users, 10);
        assert_eq!(response[0].total_engaged_users, 8);
        assert_eq!(response[0].copilot_ide_code_completions.total_engaged_users, 8);
    }

    #[test]
    fn test_deserialize_language_breakdown_with_metrics() {
        // GREEN: Test for language-specific metrics (suggestions, acceptances, lines)
        let json = r#"{
            "name": "rust",
            "total_engaged_users": 5,
            "total_code_suggestions": 1000,
            "total_code_acceptances": 750,
            "total_code_lines_suggested": 5000,
            "total_code_lines_accepted": 3750
        }"#;

        let breakdown: LanguageBreakdown = serde_json::from_str(json).unwrap();
        assert_eq!(breakdown.name, "rust");
        assert_eq!(breakdown.total_engaged_users, 5);
        assert_eq!(breakdown.total_code_suggestions, 1000);
        assert_eq!(breakdown.total_code_acceptances, 750);
        assert_eq!(breakdown.total_code_lines_suggested, 5000);
        assert_eq!(breakdown.total_code_lines_accepted, 3750);
    }

    #[test]
    fn test_deserialize_editor_breakdown_with_metrics() {
        // RED: Test for editor-specific metrics with model breakdown
        let json = r#"{
            "name": "vscode",
            "total_engaged_users": 8,
            "total_code_suggestions": 2000,
            "total_code_acceptances": 1500,
            "total_code_lines_suggested": 10000,
            "total_code_lines_accepted": 7500,
            "models": [
                {
                    "name": "gpt-4",
                    "is_custom_model": false,
                    "total_engaged_users": 8,
                    "total_code_suggestions": 1200,
                    "total_code_acceptances": 900,
                    "total_code_lines_suggested": 6000,
                    "total_code_lines_accepted": 4500
                }
            ]
        }"#;

        let breakdown: EditorBreakdown = serde_json::from_str(json).unwrap();
        assert_eq!(breakdown.name, "vscode");
        assert_eq!(breakdown.total_engaged_users, 8);
        assert_eq!(breakdown.total_code_suggestions, 2000);
        assert_eq!(breakdown.total_code_acceptances, 1500);
        assert_eq!(breakdown.total_code_lines_suggested, 10000);
        assert_eq!(breakdown.total_code_lines_accepted, 7500);
        assert_eq!(breakdown.models.len(), 1);
        assert_eq!(breakdown.models[0].name, "gpt-4");
        assert_eq!(breakdown.models[0].total_code_suggestions, 1200);
    }

    #[test]
    fn test_deserialize_copilot_ide_chat() {
        // RED: Test for IDE chat metrics
        let json = r#"{
            "total_engaged_users": 15,
            "editors": [
                {
                    "name": "vscode",
                    "total_engaged_users": 12,
                    "models": [
                        {
                            "name": "gpt-4",
                            "is_custom_model": false,
                            "total_engaged_users": 12,
                            "total_chats": 250,
                            "total_chat_insertion_events": 80,
                            "total_chat_copy_events": 120
                        }
                    ]
                }
            ]
        }"#;

        let chat: CopilotIdeChat = serde_json::from_str(json).unwrap();
        assert_eq!(chat.total_engaged_users, 15);
        assert_eq!(chat.editors.len(), 1);
        assert_eq!(chat.editors[0].name, "vscode");
        assert_eq!(chat.editors[0].total_engaged_users, 12);
        assert_eq!(chat.editors[0].models.len(), 1);
        assert_eq!(chat.editors[0].models[0].total_chats, 250);
        assert_eq!(chat.editors[0].models[0].total_chat_insertion_events, 80);
        assert_eq!(chat.editors[0].models[0].total_chat_copy_events, 120);
    }

    #[test]
    fn test_deserialize_copilot_dotcom_chat() {
        // RED: Test for dotcom chat metrics
        let json = r#"{
            "total_engaged_users": 25,
            "models": [
                {
                    "name": "gpt-4",
                    "is_custom_model": false,
                    "total_engaged_users": 20,
                    "total_chats": 500
                }
            ]
        }"#;

        let chat: CopilotDotcomChat = serde_json::from_str(json).unwrap();
        assert_eq!(chat.total_engaged_users, 25);
        assert_eq!(chat.models.len(), 1);
        assert_eq!(chat.models[0].name, "gpt-4");
        assert_eq!(chat.models[0].total_engaged_users, 20);
        assert_eq!(chat.models[0].total_chats, 500);
    }

    #[test]
    fn test_deserialize_copilot_dotcom_pull_requests() {
        // RED: Test for pull request summaries
        let json = r#"{
            "total_engaged_users": 10,
            "repositories": [
                {
                    "name": "my-repo",
                    "total_engaged_users": 8,
                    "models": [
                        {
                            "name": "gpt-4",
                            "is_custom_model": false,
                            "total_engaged_users": 8,
                            "total_pr_summaries_created": 50
                        }
                    ]
                }
            ]
        }"#;

        let pr: CopilotDotcomPullRequests = serde_json::from_str(json).unwrap();
        assert_eq!(pr.total_engaged_users, 10);
        assert_eq!(pr.repositories.len(), 1);
        assert_eq!(pr.repositories[0].name, "my-repo");
        assert_eq!(pr.repositories[0].total_engaged_users, 8);
        assert_eq!(pr.repositories[0].models.len(), 1);
        assert_eq!(pr.repositories[0].models[0].total_pr_summaries_created, 50);
    }

    #[test]
    fn test_deserialize_github_metrics_day_with_all_optional_fields() {
        // RED: Test for GitHubMetricsDay with all optional metric types present
        let json = r#"{
            "date": "2025-09-30",
            "total_active_users": 50,
            "total_engaged_users": 45,
            "copilot_ide_code_completions": {
                "total_engaged_users": 40,
                "languages": [],
                "editors": []
            },
            "copilot_ide_chat": {
                "total_engaged_users": 30,
                "editors": []
            },
            "copilot_dotcom_chat": {
                "total_engaged_users": 25,
                "models": []
            },
            "copilot_dotcom_pull_requests": {
                "total_engaged_users": 20,
                "repositories": []
            }
        }"#;

        let day: GitHubMetricsDay = serde_json::from_str(json).unwrap();
        assert_eq!(day.date, "2025-09-30");
        assert_eq!(day.total_active_users, 50);
        assert_eq!(day.total_engaged_users, 45);
        
        // Code completions should always be present
        assert_eq!(day.copilot_ide_code_completions.total_engaged_users, 40);
        
        // Optional fields should be Some when present
        assert!(day.copilot_ide_chat.is_some());
        assert_eq!(day.copilot_ide_chat.as_ref().unwrap().total_engaged_users, 30);
        
        assert!(day.copilot_dotcom_chat.is_some());
        assert_eq!(day.copilot_dotcom_chat.as_ref().unwrap().total_engaged_users, 25);
        
        assert!(day.copilot_dotcom_pull_requests.is_some());
        assert_eq!(day.copilot_dotcom_pull_requests.as_ref().unwrap().total_engaged_users, 20);
    }

    #[test]
    fn test_deserialize_complete_realistic_api_response() {
        // RED: Test with realistic complete API response including nested data
        let json = r#"[
            {
                "date": "2025-09-29",
                "total_active_users": 100,
                "total_engaged_users": 85,
                "copilot_ide_code_completions": {
                    "total_engaged_users": 75,
                    "languages": [
                        {
                            "name": "rust",
                            "total_engaged_users": 45,
                            "total_code_suggestions": 5000,
                            "total_code_acceptances": 3750,
                            "total_code_lines_suggested": 25000,
                            "total_code_lines_accepted": 18750
                        },
                        {
                            "name": "python",
                            "total_engaged_users": 30,
                            "total_code_suggestions": 3000,
                            "total_code_acceptances": 2250,
                            "total_code_lines_suggested": 15000,
                            "total_code_lines_accepted": 11250
                        }
                    ],
                    "editors": [
                        {
                            "name": "vscode",
                            "total_engaged_users": 60,
                            "total_code_suggestions": 7000,
                            "total_code_acceptances": 5250,
                            "total_code_lines_suggested": 35000,
                            "total_code_lines_accepted": 26250,
                            "models": [
                                {
                                    "name": "gpt-4",
                                    "is_custom_model": false,
                                    "total_engaged_users": 60,
                                    "total_code_suggestions": 4000,
                                    "total_code_acceptances": 3000,
                                    "total_code_lines_suggested": 20000,
                                    "total_code_lines_accepted": 15000
                                },
                                {
                                    "name": "custom-model-v2",
                                    "is_custom_model": true,
                                    "custom_model_training_date": "2025-09-01",
                                    "total_engaged_users": 20,
                                    "total_code_suggestions": 3000,
                                    "total_code_acceptances": 2250,
                                    "total_code_lines_suggested": 15000,
                                    "total_code_lines_accepted": 11250
                                }
                            ]
                        },
                        {
                            "name": "neovim",
                            "total_engaged_users": 15,
                            "total_code_suggestions": 1000,
                            "total_code_acceptances": 750,
                            "total_code_lines_suggested": 5000,
                            "total_code_lines_accepted": 3750,
                            "models": [
                                {
                                    "name": "gpt-4",
                                    "is_custom_model": false,
                                    "total_engaged_users": 15,
                                    "total_code_suggestions": 1000,
                                    "total_code_acceptances": 750,
                                    "total_code_lines_suggested": 5000,
                                    "total_code_lines_accepted": 3750
                                }
                            ]
                        }
                    ]
                },
                "copilot_ide_chat": {
                    "total_engaged_users": 50,
                    "editors": [
                        {
                            "name": "vscode",
                            "total_engaged_users": 45,
                            "models": [
                                {
                                    "name": "gpt-4",
                                    "is_custom_model": false,
                                    "total_engaged_users": 45,
                                    "total_chats": 800,
                                    "total_chat_insertion_events": 250,
                                    "total_chat_copy_events": 400
                                }
                            ]
                        }
                    ]
                },
                "copilot_dotcom_chat": {
                    "total_engaged_users": 35,
                    "models": [
                        {
                            "name": "gpt-4",
                            "is_custom_model": false,
                            "total_engaged_users": 30,
                            "total_chats": 600
                        },
                        {
                            "name": "claude-3",
                            "is_custom_model": false,
                            "total_engaged_users": 15,
                            "total_chats": 200
                        }
                    ]
                },
                "copilot_dotcom_pull_requests": {
                    "total_engaged_users": 25,
                    "repositories": [
                        {
                            "name": "main-repo",
                            "total_engaged_users": 20,
                            "models": [
                                {
                                    "name": "gpt-4",
                                    "is_custom_model": false,
                                    "total_engaged_users": 20,
                                    "total_pr_summaries_created": 150
                                }
                            ]
                        },
                        {
                            "name": "docs-repo",
                            "total_engaged_users": 10,
                            "models": [
                                {
                                    "name": "gpt-4",
                                    "is_custom_model": false,
                                    "total_engaged_users": 10,
                                    "total_pr_summaries_created": 50
                                }
                            ]
                        }
                    ]
                }
            }
        ]"#;

        let response: Vec<GitHubMetricsDay> = serde_json::from_str(json).unwrap();
        assert_eq!(response.len(), 1);
        
        let day = &response[0];
        assert_eq!(day.date, "2025-09-29");
        assert_eq!(day.total_active_users, 100);
        assert_eq!(day.total_engaged_users, 85);
        
        // Verify code completions with nested data
        assert_eq!(day.copilot_ide_code_completions.total_engaged_users, 75);
        assert_eq!(day.copilot_ide_code_completions.languages.len(), 2);
        assert_eq!(day.copilot_ide_code_completions.languages[0].name, "rust");
        assert_eq!(day.copilot_ide_code_completions.languages[0].total_code_suggestions, 5000);
        
        assert_eq!(day.copilot_ide_code_completions.editors.len(), 2);
        assert_eq!(day.copilot_ide_code_completions.editors[0].name, "vscode");
        assert_eq!(day.copilot_ide_code_completions.editors[0].models.len(), 2);
        assert_eq!(day.copilot_ide_code_completions.editors[0].models[1].name, "custom-model-v2");
        assert_eq!(day.copilot_ide_code_completions.editors[0].models[1].is_custom_model, true);
        assert_eq!(day.copilot_ide_code_completions.editors[0].models[1].custom_model_training_date, Some("2025-09-01".to_string()));
        
        // Verify IDE chat
        let ide_chat = day.copilot_ide_chat.as_ref().unwrap();
        assert_eq!(ide_chat.total_engaged_users, 50);
        assert_eq!(ide_chat.editors.len(), 1);
        assert_eq!(ide_chat.editors[0].models[0].total_chats, 800);
        
        // Verify dotcom chat
        let dotcom_chat = day.copilot_dotcom_chat.as_ref().unwrap();
        assert_eq!(dotcom_chat.total_engaged_users, 35);
        assert_eq!(dotcom_chat.models.len(), 2);
        assert_eq!(dotcom_chat.models[1].name, "claude-3");
        
        // Verify PR summaries
        let pr = day.copilot_dotcom_pull_requests.as_ref().unwrap();
        assert_eq!(pr.total_engaged_users, 25);
        assert_eq!(pr.repositories.len(), 2);
        assert_eq!(pr.repositories[0].name, "main-repo");
        assert_eq!(pr.repositories[0].models[0].total_pr_summaries_created, 150);
    }

    #[test]
    fn test_deserialize_github_metrics_day_with_missing_optional_fields() {
        // RED: Test that optional fields can be completely omitted from JSON
        let json = r#"{
            "date": "2025-09-30",
            "total_active_users": 50,
            "total_engaged_users": 45,
            "copilot_ide_code_completions": {
                "total_engaged_users": 40,
                "languages": [
                    {
                        "name": "rust",
                        "total_engaged_users": 20,
                        "total_code_suggestions": 1000,
                        "total_code_acceptances": 750,
                        "total_code_lines_suggested": 5000,
                        "total_code_lines_accepted": 3750
                    }
                ],
                "editors": []
            }
        }"#;

        let day: GitHubMetricsDay = serde_json::from_str(json).unwrap();
        assert_eq!(day.date, "2025-09-30");
        assert_eq!(day.total_active_users, 50);
        assert_eq!(day.total_engaged_users, 45);
        
        // Required field should be present
        assert_eq!(day.copilot_ide_code_completions.total_engaged_users, 40);
        assert_eq!(day.copilot_ide_code_completions.languages.len(), 1);
        
        // Optional fields should be None when omitted
        assert!(day.copilot_ide_chat.is_none());
        assert!(day.copilot_dotcom_chat.is_none());
        assert!(day.copilot_dotcom_pull_requests.is_none());
    }

    #[test]
    fn test_deserialize_github_metrics_day_with_partial_optional_fields() {
        // RED: Test with only some optional fields present
        let json = r#"{
            "date": "2025-09-30",
            "total_active_users": 100,
            "total_engaged_users": 80,
            "copilot_ide_code_completions": {
                "total_engaged_users": 70,
                "languages": [],
                "editors": []
            },
            "copilot_ide_chat": {
                "total_engaged_users": 30,
                "editors": []
            }
        }"#;

        let day: GitHubMetricsDay = serde_json::from_str(json).unwrap();
        assert_eq!(day.date, "2025-09-30");
        
        // IDE chat should be present
        assert!(day.copilot_ide_chat.is_some());
        assert_eq!(day.copilot_ide_chat.as_ref().unwrap().total_engaged_users, 30);
        
        // Other optional fields should be None
        assert!(day.copilot_dotcom_chat.is_none());
        assert!(day.copilot_dotcom_pull_requests.is_none());
    }

    #[test]
    fn test_deserialize_array_response_with_mixed_optional_fields() {
        // RED: Test array response where different days have different optional fields
        let json = r#"[
            {
                "date": "2025-09-29",
                "total_active_users": 100,
                "total_engaged_users": 85,
                "copilot_ide_code_completions": {
                    "total_engaged_users": 75,
                    "languages": [],
                    "editors": []
                },
                "copilot_ide_chat": {
                    "total_engaged_users": 50,
                    "editors": []
                },
                "copilot_dotcom_chat": {
                    "total_engaged_users": 35,
                    "models": []
                },
                "copilot_dotcom_pull_requests": {
                    "total_engaged_users": 25,
                    "repositories": []
                }
            },
            {
                "date": "2025-09-30",
                "total_active_users": 90,
                "total_engaged_users": 75,
                "copilot_ide_code_completions": {
                    "total_engaged_users": 65,
                    "languages": [],
                    "editors": []
                }
            },
            {
                "date": "2025-10-01",
                "total_active_users": 95,
                "total_engaged_users": 80,
                "copilot_ide_code_completions": {
                    "total_engaged_users": 70,
                    "languages": [],
                    "editors": []
                },
                "copilot_dotcom_chat": {
                    "total_engaged_users": 30,
                    "models": []
                }
            }
        ]"#;

        let response: Vec<GitHubMetricsDay> = serde_json::from_str(json).unwrap();
        assert_eq!(response.len(), 3);
        
        // First day has all optional fields
        let day1 = &response[0];
        assert_eq!(day1.date, "2025-09-29");
        assert!(day1.copilot_ide_chat.is_some());
        assert!(day1.copilot_dotcom_chat.is_some());
        assert!(day1.copilot_dotcom_pull_requests.is_some());
        
        // Second day has no optional fields
        let day2 = &response[1];
        assert_eq!(day2.date, "2025-09-30");
        assert!(day2.copilot_ide_chat.is_none());
        assert!(day2.copilot_dotcom_chat.is_none());
        assert!(day2.copilot_dotcom_pull_requests.is_none());
        
        // Third day has only dotcom chat
        let day3 = &response[2];
        assert_eq!(day3.date, "2025-10-01");
        assert!(day3.copilot_ide_chat.is_none());
        assert!(day3.copilot_dotcom_chat.is_some());
        assert_eq!(day3.copilot_dotcom_chat.as_ref().unwrap().total_engaged_users, 30);
        assert!(day3.copilot_dotcom_pull_requests.is_none());
    }

    #[test]
    fn test_github_client_new() {
        // GREEN: Test that GitHubClient can be constructed with token and default base URL
        let token = "ghp_test_token_123";
        let client = GitHubClient::new(token.to_string())
            .expect("Failed to create GitHubClient");
        
        // We can't directly test private fields, but we can verify the client exists
        // In the next cycle, we'll add methods to verify functionality
        assert_eq!(client.token, token);
        assert_eq!(client.base_url, "https://api.github.com");
    }

    #[test]
    fn test_github_client_new_creates_client_with_token() {
        // RED: Test that GitHubClient::new() properly creates a client
        // We'll extend this test to verify headers once we can make actual requests
        let token = "ghp_test_token_123";
        let result = GitHubClient::new(token.to_string())
            .expect("Failed to create GitHubClient");
        
        // Should successfully create client with token stored
        assert_eq!(result.token, token);
        
        // The next test will verify that the client is configured with proper headers
        // by using ClientBuilder with default_headers()
    }

    #[test]
    fn test_github_client_new_with_headers() {
        // RED: This test expects GitHubClient::new to return a Result
        // because header configuration with ClientBuilder can fail
        let token = "ghp_test_token_123";
        
        // This should fail compilation because new() returns Self, not Result
        let client = GitHubClient::new(token.to_string()).expect("Failed to create client");
        
        // Once we fix it to return Result, this will pass
        assert_eq!(client.token, token);
    }
}
