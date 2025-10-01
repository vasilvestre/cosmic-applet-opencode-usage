# Feature: GitHub API Client

## Overview
Develop an asynchronous Rust module to authenticate with GitHub's API, fetch Copilot usage metrics, and parse responses into strongly-typed Rust structures.

## Requirements

### API Authentication
- The system SHALL authenticate with GitHub API using Personal Access Token (PAT)
- The system SHALL include the PAT in the Authorization header as "Bearer <token>"
- The system SHALL set appropriate User-Agent header
- WHEN authentication fails THEN the system SHALL return a specific error indicating auth failure

### API Endpoint Integration
- The system SHALL fetch data from GitHub Copilot Metrics API endpoint:
  - `GET /orgs/{org}/copilot/metrics` for organization aggregate metrics
- The system SHALL support the following query parameters:
  - `since`: ISO 8601 timestamp (maximum 100 days ago)
  - `until`: ISO 8601 timestamp
  - `page`: pagination page number (default: 1)
  - `per_page`: days per page (default/max: 100)
- The system SHALL handle pagination following GitHub's pagination spec
- The system SHALL make requests asynchronously using tokio
- The system SHALL set API version header: `X-GitHub-Api-Version: 2022-11-28`
- The system SHALL set Accept header: `application/vnd.github+json`

### Data Parsing
- The system SHALL parse JSON responses into strongly-typed Rust structs
- The system SHALL extract metrics from the API response including:
  - `date`: Metrics date (YYYY-MM-DD format)
  - `total_active_users`: Total users with active Copilot licenses
  - `total_engaged_users`: Total users who engaged with Copilot features
  - **Code Completions** (`copilot_ide_code_completions`):
    - Total engaged users
    - Breakdown by language (name, engaged users)
    - Breakdown by editor (name, engaged users, models)
    - Per-language metrics: suggestions, acceptances, lines suggested/accepted
  - **IDE Chat** (`copilot_ide_chat`):
    - Total engaged users
    - Breakdown by editor and model
    - Total chats, insertion events, copy events
  - **Dotcom Chat** (`copilot_dotcom_chat`):
    - Total engaged users
    - Total chats by model
  - **Pull Request Summaries** (`copilot_dotcom_pull_requests`):
    - Total engaged users
    - Breakdown by repository
    - Total PR summaries created
- The system SHALL support custom model metadata (is_custom_model, training_date)
- WHEN JSON parsing fails THEN the system SHALL return a descriptive error
- The system SHALL validate required fields are present in the response

### Error Handling
- The system SHALL handle network connectivity errors
- The system SHALL handle HTTP error responses (400, 401, 403, 404, 500, etc.)
- The system SHALL handle rate limiting (429 responses)
- WHEN rate limited THEN the system SHALL extract retry-after information
- The system SHALL handle malformed JSON responses
- The system SHALL provide specific error types for different failure modes

### Request Management
- The system SHALL implement request timeouts (30 seconds default)
- The system SHALL support cancellation of in-flight requests
- The system SHALL log API requests and responses (excluding PAT) for debugging

## Acceptance Criteria
- [ ] Async HTTP client configured (reqwest or similar)
- [ ] Authentication with PAT implemented
- [ ] Metrics API endpoint called successfully
- [ ] JSON response parsed into Rust structs
- [ ] All required metrics extracted
- [ ] Network errors handled gracefully
- [ ] HTTP error codes handled with specific error types
- [ ] Rate limiting detected and handled
- [ ] Unit tests for parsing logic
- [ ] Integration tests with mock API responses

## Technical Notes
- Use reqwest crate for HTTP client
- Use serde and serde_json for JSON parsing
- Use tokio for async runtime
- Define custom Error enum for specific failure modes
- Consider using thiserror crate for error handling
- GitHub API documentation: https://docs.github.com/en/rest/copilot

## Dependencies
- Feature 01: Project Setup & Boilerplate Applet
- Feature 02: Configuration & Authentication (for PAT retrieval)
