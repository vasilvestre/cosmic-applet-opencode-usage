# Feature: Basic UI - Panel Display

## Overview
Integrate the GitHub API client with the COSMIC applet to display a key metric in the panel, providing at-a-glance visibility of Copilot usage.

## Requirements

### Panel Icon Display
- The system SHALL display a distinctive icon representing GitHub Copilot in the COSMIC panel
- The system SHALL use an icon that is clearly visible against both light and dark themes
- The system SHALL update the icon appearance to indicate data status (loading, success, error)
- WHEN data is being fetched THEN the system SHALL show a loading indicator

### Metric Display
- The system SHALL display one primary metric alongside or overlaying the icon
- The system SHALL support displaying:
  - Total completions count (default)
  - OR acceptance rate percentage
- The system SHALL format numbers for readability (e.g., "1.2K" for 1200)
- WHEN no data is available THEN the system SHALL display a placeholder (e.g., "--")

### Data Integration
- WHEN the applet starts THEN the system SHALL immediately fetch metrics from the API
- The system SHALL display fetched metrics in the panel within 2 seconds of successful API response
- WHEN API fetch fails THEN the system SHALL display an error indicator in the panel
- The system SHALL update the panel display when new data arrives

### Visual States
- The system SHALL support the following visual states:
  - **Initial/Loading**: Loading indicator visible
  - **Success**: Metric displayed with normal styling
  - **Error**: Error icon/indicator displayed
  - **Stale**: Visual indicator when data is outdated (for future caching)
- WHEN state changes THEN the system SHALL update the panel display

### Accessibility
- The system SHALL provide tooltip text on hover showing last update time
- The system SHALL ensure text contrast meets accessibility standards
- The system SHALL properly handle libcosmic theme changes

## Acceptance Criteria
- [ ] Icon appears in COSMIC panel
- [ ] Icon visible in both light and dark themes
- [ ] Primary metric (completions count) displayed
- [ ] Numbers formatted for readability
- [ ] Loading state displayed during fetch
- [ ] Error state displayed on fetch failure
- [ ] Placeholder displayed when no data available
- [ ] Tooltip shows last update time
- [ ] Panel updates within 2 seconds of data arrival
- [ ] Visual appearance follows COSMIC design guidelines

## Technical Notes
- Use libcosmic icon components
- Consider using cosmic_text for text rendering
- Use libcosmic theming system
- Store last fetch time for tooltip display
- Panel space is limited - keep display concise

## Dependencies
- Feature 01: Project Setup & Boilerplate Applet
- Feature 02: Configuration & Authentication
- Feature 03: GitHub API Client
