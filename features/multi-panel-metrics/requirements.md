# Feature: Multiple Panel Metrics Selection

## Overview
Allow users to select and display multiple metrics simultaneously in the panel applet, replacing the current single-selection radio button interface with checkboxes for multi-selection.

## Requirements (EARS Format)

### Core Selection Behavior

**R1**: WHEN the user opens the settings panel THEN the system SHALL display checkboxes for each panel metric option (Cost, Interactions, Input Tokens, Output Tokens, Reasoning Tokens)

**R2**: WHEN the user saves configuration with multiple metrics selected THEN the system SHALL store the metrics as a list preserving the selection order

**R3**: WHEN no metrics are selected THEN the system SHALL display only the applet icon without any text

### Display Formatting

**R4**: WHEN the applet displays panel content with Cost metric selected THEN the system SHALL format it with the dollar sign and decimal (e.g., "$1.23")

**R5**: WHEN the applet displays panel content with Interactions metric selected THEN the system SHALL format it with the count and "x" suffix (e.g., "5x")

**R6**: WHEN the applet displays panel content with Input Tokens metric selected THEN the system SHALL format it as "IT: " followed by the formatted token count (e.g., "IT: 10k")

**R7**: WHEN the applet displays panel content with Output Tokens metric selected THEN the system SHALL format it as "OT: " followed by the formatted token count (e.g., "OT: 5k")

**R8**: WHEN the applet displays panel content with Reasoning Tokens metric selected THEN the system SHALL format it as "RT: " followed by the formatted token count (e.g., "RT: 2k")

**R9**: WHEN the applet displays panel content with multiple metrics selected THEN the system SHALL format them separated by " | " in the order: Cost, Interactions, IT, OT, RT (e.g., "$1.23 | 5x | IT: 10k | OT: 5k | RT: 2k")

**R10**: IF a selected metric's data is unavailable THEN the system SHALL display "N/A" for that metric's value

### Default Behavior

**R11**: The system SHALL set the default panel metrics to all metrics enabled (Cost, Interactions, Input Tokens, Output Tokens, Reasoning Tokens)

**R12**: WHEN the user clicks "Reset to Defaults" button THEN the system SHALL select all metrics (Cost, Interactions, Input Tokens, Output Tokens, Reasoning Tokens)

### Data Integrity

**R13**: The system SHALL display metrics in fixed order (Cost, Interactions, IT, OT, RT) regardless of which checkboxes the user checks first

**R14**: The system SHALL prevent duplicate metrics in the selection list

## Examples

### Display Examples
- All metrics: `"$1.23 | 5x | IT: 10k | OT: 5k | RT: 2k"`
- Cost only: `"$1.23"`
- Cost + Interactions: `"$1.23 | 5x"`
- Token metrics only: `"IT: 10k | OT: 5k | RT: 2k"`
- No metrics: `[icon only]`

### Edge Cases
- Missing cost data: `"N/A | 5x | IT: 10k | OT: 5k | RT: 2k"`
- All data missing: `"N/A | N/A | N/A | N/A | N/A"`
- Empty selection: Display icon only (no text)

## Out of Scope
- Migration from old single-value `panel_metric` configurations (not needed for current use case)
- Custom ordering of metrics (fixed order is sufficient)
- Limiting the number of selected metrics (allow all 5)
