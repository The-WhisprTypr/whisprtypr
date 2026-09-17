# Changelog

All notable changes to Whisprtypr will be documented in this file.

This project follows the spirit of [Keep a Changelog](https://keepachangelog.com/) and uses semantic version tags when releases are published.

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
