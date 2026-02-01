# Changelog

All notable changes to SSH Key Fixer will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.0] - 2026-02-01

### Added
- Interactive terminal UI to browse and modify flags through an interactive menu
- Toggle both User Presence (UP) and User Verification (UV) flags independently
- Visual flag display as checkboxes with `[O]` for enabled and `[ ]` for disabled
- Flexible save options: no password, same password, new password, or quit without saving
- In-place UI updates that clear and redraw the screen after each action (no scrolling)
- Confirmation feedback messages appearing below the input prompt

### Changed
- Workflow changed from automatic flag setting to interactive menu-driven operation
- Users now have full control over which flags to modify and how to save the key
- Output simplified with visual flag states and status messages
- Uses ANSI escape codes for in-place screen clearing and cursor repositioning

## [1.0.0] - 2026-01-15

### Added
- Automatic UV_REQUIRED flag setting for FIDO2/U2F security keys
- Support for multiple key types: `SkEd25519` and `SkEcdsaSha2NistP256`
- Encryption handling: supports both encrypted and unencrypted SSH keys
- Hidden password input (using `rpassword` crate)
- Non-destructive in-place key modification
- Smart flag detection to skip unnecessary modifications
- Original line ending format preservation (LF, CRLF, CR)
- Clear error messages for common issues
- Custom patched `ssh-key` dependency with flag bit setting support
