# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.3] - 2024-12-19

### Security
- Added comprehensive input/output sanitization to prevent prompt injection attacks
- Implemented file path validation to prevent directory traversal attacks
- Added size limits (1MB) for input and output to prevent memory exhaustion
- Added detection and logging of dangerous patterns in input/output
- Added null byte removal and line ending normalization
- Added shell character escaping in output to prevent command injection

### Changed
- Enhanced security posture for production use
- Improved logging for security-related events

## [0.1.2] - 2024-12-19

### Added
- Enhanced service call logging with detailed request/response information
- Improved error handling and user feedback
- Better configuration management

### Changed
- Updated dependencies to latest versions
- Improved error messages and logging

## [0.1.1] - 2024-12-19

### Added
- Basic functionality for OpenAI-compatible API interaction
- File input support
- Streaming response support
- Configuration file support

### Changed
- Initial release
