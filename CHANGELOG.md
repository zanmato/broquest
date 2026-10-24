# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Features

- Settings, Appearance has a VS Code Themes group that searches Open VSX for
  VS Code color themes, previews them, lets you try one across the app before
  installing, and installs them converted to Broquest's theme format
- OAuth2 tokens are kept between requests, per collection and environment,
  refreshed when they expire and fetched again once on 401
- OAuth2 authorization code grant in the Auth tab, with PKCE, `state` and a
  configurable redirect URL
- OAuth2 token requests can send the client credentials as a Basic auth
  header and carry extra fields such as `audience`
- Choose which OAuth2 token to send (access, ID or both), the header and the
  prefix
- Get Token and Clear Token buttons, and `bro.resetOAuth2Token()` in scripts

### Fixes

- OAuth2 auth is saved as `type = "oauth2"` instead of `o_auth2`. Files with
  the old name still load

## [0.1.13] - 2026-07-07

### Features

- Added partial support for the opencollection format
- Added a history panel
- Added a command palette

### Style

- Updated app icon

## [0.1.12] - 2026-04-15

### Features

- Add settings view
- Add panel layout setting
- Add image support (detect image mime and render inline)
- Detect PDF mime and offer to open with OS
- WSDL import
- Add XML formatting

### Fixes

- Allow overriding Content-Type header
- Fix maximizing
- Fix tab index

### Style

- Wrap URL bar
- Change tab badge colors

## [0.1.11] - 2026-03-26

### Features

- Add auth editor with support for Basic, Digest, JWT and OAuth2 CC
- Switched to jemalloc on Linux
- Allow cancelling requests by clicking the send button while it is loading

### Fixes

- Fixed scrolling in the various query, path, auth editors
- Increased request timeout

## [0.1.10] - 2026-03-05

### Features

- Add self-update functionality for the app
- Add simple script linting for pre-request and post-response scripts

## [0.1.9] - 2026-02-13

### Features

- Create collection from OpenAPI spec
- Add JSON path filtering
- Add simple completions for pre-request and post response scripts

### Fixes

- Update URL grammar to work with variables
- Fix macOS quit app on window close

## [0.1.8] - 2025-12-26

### Features

- Add drag and drop in the tree to move requests
- Add request dirty check and indicator
- Add save keybindings for collection, group & request editors

### Fixes

- Fix script editor layout issues
- Correct content type on load

## [0.1.7] - 2025-12-22

### Features

- Read env secrets and show them masked

### Build

- macOS .app bundle and Linux binary

### Refactor

- Remove tab loading

## [0.1.6] - 2025-12-08

### Features

- Support multipart form data
- Add param count badges
- Simple sort of collections, requests and groups

### Fixes

- Fix replacing requests when saving existing collection
- Actually refresh collections

## [0.1.5] - 2025-12-02

### Fixes

- Fix saving and loading requests

## [0.1.4] - 2025-12-02

### Fixes

- Fix highlighting query
- Do not show console on Windows

## [0.1.3] - 2025-12-02

### Features

- Add XML highlighting

## [0.1.2] - 2025-12-02

### Features

- Add simple group support (sub folders)
- Use llrt_modules for improved node compatibility

## [0.1.1] - 2025-11-27

### Features

- Send on Enter
- Improved errors
- Add URL highlighting
- Reactive name editing
- Two-way query param binding

## [0.1.0] - 2025-11-26

### Features

- Initial version

[0.1.10]: https://github.com/zanmato/broquest/compare/v0.1.9...v0.1.10
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
