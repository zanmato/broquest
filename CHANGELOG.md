# Changelog

All notable changes to this project will be documented in this file.

## [0.1.9] - 2025-02-15

### Features
- Create collection from OpenAPI spec
- Add JSON path filtering
- Add simple completions for pre-request and post response scripts

### Fixes
- Update URL grammar to work with variables
- Fix macOS quit app on window close

## [0.1.8] - 2025-02-14

### Features
- Add drag and drop in the tree to move requests
- Add request dirty check and indicator
- Add save keybindings for collection, group & request editors

### Fixes
- Fix script editor layout issues
- Correct content type on load

## [0.1.7] - 2025-02-13

### Features
- Read env secrets and show them masked

### Build
- macOS .app bundle and Linux binary

### Refactor
- Replace zed-reqwest and gpui_tokio usage with reqwest and async-compat
- Remove tab loading

## [0.1.6] - 2025-02-12

### Features
- Support multipart form data
- Add param count badges
- Simple sort of collections, requests and groups

### Fixes
- Fix replacing requests when saving existing collection
- Actually refresh collections

## [0.1.5] - 2025-02-11

### Fixes
- Fix saving and loading requests

## [0.1.4] - 2025-02-10

### Fixes
- Fix highlighting query
- Do not show console on Windows

## [0.1.3] - 2025-02-09

### Features
- Add XML highlighting

## [0.1.2] - 2025-02-08

### Features
- Add simple group support (sub folders)
- Use llrt_modules for improved node compatibility

## [0.1.1] - 2025-02-07

### Features
- Send on Enter
- Improved errors
- Add URL highlighting
- Reactive name editing
- Two-way query param binding

## [0.1.0] - 2025-02-06

### Features
- Initial version

[0.1.9]: https://github.com/zanmato/broquest/compare/v0.1.8...v0.1.9
[0.1.8]: https://github.com/zanmato/broquest/compare/v0.1.7...v0.1.8
[0.1.7]: https://github.com/zanmato/broquest/compare/v0.1.6...v0.1.7
[0.1.6]: https://github.com/zanmato/broquest/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/zanmato/broquest/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/zanmato/broquest/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/zanmato/broquest/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/zanmato/broquest/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/zanmato/broquest/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/zanmato/broquest/releases/tag/v0.1.0
