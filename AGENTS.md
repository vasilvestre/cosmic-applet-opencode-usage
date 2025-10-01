# AGENTS.md — Agentic Coding Guidelines

## Build, Lint, and Test Commands

- **Build (release):** `just build-release`
- **Build (debug):** `just build-debug`
- **Run:** `just run`
- **Format:** `cargo +nightly fmt --all`
- **Lint:** `cargo clippy --all-features -- -W clippy::pedantic`
- **Test all:** `cargo test`
- **Test single:** `cargo test <test_name>`
- **Check (CI):** `just check` or see `.github/workflows/ci.yml` for CI steps

## Code Style Guidelines

- **Imports:** Group std, external, and internal; use explicit paths.
- **Formatting:** Enforced by `rustfmt` (see CI); always run before commit.
- **Types:** Prefer explicit types; use `Result<T, E>` for fallible functions.
- **Naming:** Use `snake_case` for functions/variables, `CamelCase` for types, `SCREAMING_SNAKE_CASE` for constants.
- **Error Handling:** Use `thiserror` for custom errors; propagate errors with `?`.
- **Modules:** Organize by feature (see `src/core/`, `src/ui/`); keep files small and focused.
- **Comments:** Use doc comments (`///`) for public items; keep inline comments concise.
- **Tests:** Place integration tests in `tests/`; use descriptive test names.
- **Clippy:** Fix all warnings, especially `clippy::pedantic`.
- **Licensing:** All files must start with SPDX header (`// SPDX-License-Identifier: GPL-3.0-only`).

## Resources

- **COSMIC Panel Applets:** Refer to https://pop-os.github.io/libcosmic-book/panel-applets.html for applet development guidelines and patterns.

---

No Cursor or Copilot rules detected. Update this file if such rules are added.
