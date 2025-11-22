# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-11-22

### Added

#### 🎯 Powerful CLI Interface
- **New `mokku` binary** - Modern, intuitive command-line interface
- **Interactive mode** - Guided workflows using `inquire` when no arguments provided
- **Quick mock creation** - `mokku mock POST /users 201 '{"id": 42}'`
- **OpenAPI import from CLI** - `mokku import swagger.yaml --start --open`
- **Browser auto-open** - `--open` flag to automatically open dashboard
- **Colored terminal output** - Beautiful, user-friendly CLI experience
- **Session replay placeholder** - `mokku replay` command structure (implementation coming soon)

#### 📦 New Dependencies
- `inquire` - Interactive CLI prompts
- `colored` - Terminal color output
- `anyhow` - Improved error handling
- `thiserror` - Custom error types
- `open` - Browser auto-launch
- `serde_yaml` - YAML file support for OpenAPI import

### Changed

#### 🏗️ Architecture Improvements
- **Refactored codebase** - Split `src/main.rs` into library (`src/lib.rs`) and binaries
- **Dual binary support** - Both `RustMock` (legacy) and `mokku` (new CLI) available
- **Public API** - Exposed server functions for reuse in both binaries
- **Modular design** - Better code organization for maintainability

#### 📝 Documentation
- **Updated README** - CLI section prominently featured at the top
- **Interactive mode examples** - Clear usage instructions
- **Command reference table** - Quick lookup for all commands
- **Feature list enhanced** - CLI highlighted as primary feature

### Backward Compatibility

- **Legacy `RustMock` binary preserved** - Existing users can continue using original CLI unchanged
- **All existing features maintained** - No breaking changes to server functionality
- **API endpoints unchanged** - Full compatibility with existing integrations

### Commands Reference

```bash
mokku                              # Interactive mode
mokku server -p 3000 --open        # Start server
mokku import spec.yaml --start     # Import OpenAPI
mokku mock GET /health 200         # Quick mock
mokku --proxy https://api.prod.com # With proxy
```

---

## [0.1.0] - Previous releases

See commit history for changes before CLI implementation.

[0.2.0]: https://github.com/arthurkowalsky/Rust-Mock/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/arthurkowalsky/Rust-Mock/releases/tag/v0.1.0
