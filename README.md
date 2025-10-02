# COSMIC OpenCode Usage Tracker

A COSMIC Desktop applet for tracking OpenCode/Claude Code usage metrics and token consumption.

## Features

- **Real-time Token Tracking**: Monitor your OpenCode token usage from local storage files
- **Aggregated Metrics**: View total input, output, reasoning, and cache tokens
- **Cost Tracking**: Track estimated costs based on token usage
- **System Panel Integration**: Quick access from the COSMIC panel
- **Historical Data Viewer**: Dedicated viewer application for detailed usage statistics and charts
- **One-Click Access**: Launch the viewer directly from the applet with the "View Stats" button
- **Lightweight**: Minimal resource usage with efficient caching

## What is OpenCode?

[OpenCode](https://github.com/sst/opencode) (formerly Claude Code) is an AI-powered coding assistant. This applet reads usage data from your local OpenCode installation to help you track token consumption and costs.

## Requirements

- COSMIC Desktop Environment
- OpenCode installed with usage data in `~/.local/share/opencode/storage/part/`
- Rust toolchain (for building)

## Install

To install the COSMIC OpenCode Usage Tracker, you will need [just](https://github.com/casey/just). If you're on Pop!\_OS, you can install it with:

```sh
sudo apt install just
```

After you install it, you can run the following commands to build and install the applet:

```sh
just build-release
sudo just install
```

## Usage

After installation, the applet will appear in your COSMIC panel. Click it to view:
- Total token usage (input, output, reasoning, cache)
- Number of interactions
- Estimated costs
- Last updated timestamp

## Utility Tools

The project includes several utility examples for database management:

### Backfill Historical Data
If you're starting fresh or need to populate historical data:
```sh
cargo run --example backfill_history
```
This scans your OpenCode storage files and creates daily snapshots based on file modification times, enabling proper week-over-week comparisons in the viewer.

### Other Utilities
- `check_database` - View database contents and verify snapshots
- `collect_now` - Manually trigger a data collection
- `clean_database` - Remove all snapshots from the database
- `database_usage` - Check database file size and statistics

## Development

This project follows Test-Driven Development (TDD) principles. See the `features/` directory for detailed specifications and implementation tasks.

Refer to the [COSMIC documentation](https://pop-os.github.io/libcosmic/cosmic/) for more information on building COSMIC applets.

## License

GPL-3.0
