# Changelog

All notable changes to Whisprtypr will be documented in this file.

This project follows the spirit of [Keep a Changelog](https://keepachangelog.com/) and uses semantic version tags when releases are published.

## 1.0.1

### Fixed
- **macOS Launch Crash (`DYLD: Library missing`)**: Resolved a fatal crash on macOS (especially Intel x86_64) at startup caused by missing `libonnxruntime.1.23.2.dylib` and absent `LC_RPATH` headers.
  - Bundled `libonnxruntime.1.23.2.dylib` directly into `Whisprtypr.app/Contents/Frameworks/`.
  - Embedded `@executable_path/../Frameworks` into the Mach-O binary runtime search paths.
  - Resolved `___isPlatformVersionAtLeast` undefined symbol linker errors on macOS by linking Apple Clang `compiler-rt` and providing runtime availability check shims.
  - Aligned macOS deployment target to 13.4 to match ONNX Runtime 1.23.2 and Metal requirements.
  - Added automated CI verification to assert `LC_RPATH` headers and framework presence on macOS builds.

### Added
- **Offline Grammar Checking**: Integrated local, privacy-focused grammar checking using Harper.
- **Security & Integrity Enhancements**: Strengthened trial integrity mechanisms, salt hashing, and encrypted credential storage.

## 1.0.0

feat(ui): release v1.0.0 with dashboard, custom vocabulary, and redesigned UI

This major release introduces a complete overhaul of the user interface and
adds several high-impact features to improve the dictation experience.

Key changes include:
- **New Dashboard Layout**: Replaced the single-view interface with a
  multi-page dashboard featuring an Overview, History, Models,
  Vocabulary, and Advanced views.
- **Custom Vocabulary**: Added a new feature allowing users to define
  domain-specific term replacements (e.g., "next js" -> "Next.js")
  using case-insensitive, word-boundary-aware regex matching.
- **Redesigned Visual Identity**: Implemented a new "Zapier-aligned"
  color palette and a more modern, editorial design system using
  container queries and improved typography.
- **Enhanced Recording Overlay**: Replaced the edge-glow with a
  versatile "notch-pill" overlay that can be positioned in various
  screen locations (top-center, bottom-right, etc.).
- **Improved License Management**: Optimized the license verification
  flow with smart caching to ensure hotkey-driven dictation remains
  responsive and works offline during grace periods.
- **Data Management**: Added ability to export transcription history
  as JSON files.
- **Automated Updates**: Integrated background update checking with
  user notifications.
- **Model Capabilities**: Refactored model metadata to include
  explicit language support and auto-detection capabilities.

BREAKING CHANGE: The application structure has changed from a single-view
setup to a dashboard-based navigation system. Existing settings are
migrated to include new fields for custom vocabulary and overlay
positioning.
