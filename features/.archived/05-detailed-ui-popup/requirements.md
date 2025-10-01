# Feature: Detailed UI - Popup Menu

## Overview
Create a popup menu that appears when the user clicks the applet, displaying a comprehensive breakdown of all Copilot usage metrics using libcosmic UI components.

## Requirements

### Popup Activation
- WHEN the user clicks the panel icon THEN the system SHALL display a popup menu
- The system SHALL position the popup appropriately relative to the panel icon
- WHEN the user clicks outside the popup THEN the system SHALL close it
- WHEN the user presses Escape THEN the system SHALL close the popup

### Metrics Display
- The system SHALL display the following metrics in the popup:
  - Total suggestions shown (code completions)
  - Total suggestions accepted
  - Acceptance rate (percentage with visual indicator)
  - Total chat interactions
  - Active users count
  - Date range of displayed metrics
  - Last update timestamp
- The system SHALL format all numbers for readability
- The system SHALL use visual indicators (progress bars, icons) for key metrics

### Visual Organization
- The system SHALL organize metrics into logical sections:
  - **Code Completions** (suggestions, acceptances, rate)
  - **Chat Usage** (interactions count)
  - **Organization** (active users)
  - **Status** (last update, next refresh)
- The system SHALL use libcosmic layout components for proper spacing
- The system SHALL display section headers with clear typography

### Interactive Elements
- The system SHALL provide a "Refresh Now" button to manually trigger data fetch
- The system SHALL provide a "Settings" button to open configuration UI
- WHEN "Refresh Now" is clicked THEN the system SHALL fetch new data immediately
- WHEN "Settings" is clicked THEN the system SHALL open the configuration dialog
- The system SHALL disable "Refresh Now" button while fetch is in progress

### Status Information
- The system SHALL display last successful update timestamp
- WHEN data is stale THEN the system SHALL indicate age with visual cue
- WHEN an error occurred THEN the system SHALL display error message in popup
- The system SHALL show next scheduled refresh time

### Responsive Design
- The popup SHALL have a fixed, reasonable width (300-400px)
- The popup SHALL adapt height based on content
- The system SHALL use libcosmic scrolling if content exceeds maximum height
- The system SHALL maintain readability on both light and dark themes

## Acceptance Criteria
- [ ] Popup appears on icon click
- [ ] Popup positioned correctly relative to panel
- [ ] All metrics displayed with proper formatting
- [ ] Metrics organized into clear sections
- [ ] Acceptance rate shown with visual indicator
- [ ] "Refresh Now" button triggers immediate fetch
- [ ] "Settings" button opens configuration
- [ ] Last update timestamp displayed
- [ ] Error messages shown when applicable
- [ ] Popup closes on outside click or Escape
- [ ] Popup width and height appropriate
- [ ] Readable in both light and dark themes
- [ ] Uses libcosmic components throughout

## Technical Notes
- Use libcosmic::widget for UI components
- Use libcosmic::Element for layout
- Consider using cosmic::iced components for progress bars
- Follow COSMIC popup menu patterns from other applets
- Ensure popup respects panel position (top/bottom)

## Dependencies
- Feature 01: Project Setup & Boilerplate Applet
- Feature 02: Configuration & Authentication (for Settings button)
- Feature 03: GitHub API Client
- Feature 04: Basic UI - Panel Display
