# Whisprtypr

> Speak at the speed of thought.

Open-source, local-first voice typing for your desktop.

[![Latest Release](https://img.shields.io/github/v/release/The-Whisprtypr/whisprtypr?label=Release)](https://github.com/The-Whisprtypr/whisprtypr/releases/latest)
[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL--3.0-blue.svg)](LICENSE)

**Hold a hotkey. Speak naturally. Release.** Your words are transcribed, cleaned up, and inserted directly where your cursor is. No switching windows. No breaking your flow. Just speak.

[Download](https://whisprtypr.app) · [Documentation](https://github.com/The-Whisprtypr/whisprtypr/blob/main/README.md) · [Report a Bug](https://github.com/The-Whisprtypr/whisprtypr/issues/new?template=bug_report.yml) · [Contribute](CONTRIBUTING.md)

---

## Features

- **Dictate anywhere** — global hotkeys, push-to-talk, toggle recording, and automatic insertion at the active cursor.
- **Local transcription** — Whisper on macOS and Windows with downloadable models. No audio leaves your device by default.
- **Cloud transcription (optional)** — Groq, OpenAI, Deepgram, and Mistral providers available when you need them.
- **AI formatting** — clean up rough dictation with OpenAI, Anthropic, Gemini, or a custom OpenAI-compatible endpoint.
- **Custom vocabulary** — define domain-specific terms that local models consistently mangle, and have them replaced automatically.
- **File transcription** — transcribe audio and video files with local or cloud-based engines.
- **Transcript history** — search, filter, inspect metadata, compare original and formatted text, copy, save, or re-transcribe.
- **Voice commands** — editing and navigation actions triggered by speech, keeping your hands free.
- **Desktop integration** — system tray, background-ready, launch on startup, and recording overlays.
- **In-app updates** — background update checking with signed and verified installations.
- **Cross-platform** — Windows, macOS, and Linux desktop support.

## Quick Start

1. Download and install Whisprtypr for your platform.
2. Launch the app and choose a transcription model (local or cloud).
3. Set your recording hotkey in **Settings → Hotkey**.
4. Place your cursor in any text field.
5. Press the shortcut, speak, and release.
6. Whisprtypr inserts the transcript at your cursor and stores it in local history.

## Privacy and Data Flow

Whisprtypr is offline-first. The selected mode determines what leaves your computer:

| Mode | Data Flow |
|------|-----------|
| Local transcription | Audio and transcripts stay on your device. Models are downloaded once and stored locally. |
| Cloud transcription | Audio is sent to your selected cloud provider for transcription. |
| AI formatting | Transcript is sent to your configured AI provider for rewriting. |
| Diagnostics | Anonymous error counts only. No audio, transcripts, or personal data. Controlled in Settings. |

## Installation

### macOS

Requirements: macOS 13 or later, microphone permission, and Accessibility permission for cursor insertion.

1. Download the latest macOS package from [Releases](https://whisprtypr.app).
2. Open the DMG and drag Whisprtypr to Applications.
3. Launch, grant the requested permissions, and download a transcription model.
4. Set your hotkey and start dictating.

### Windows

Requirements: 64-bit Windows 10 build 19041 or later, or Windows 11.

1. Download the latest Windows installer from [Releases](https://whisprtypr.app).
2. Run the installer and complete the setup wizard.
3. Choose a model, set hotkeys, and grant microphone access.
4. Press the recording hotkey and speak.

> **Note:** Whisprtypr is not yet signed with a Windows code-signing certificate. Windows may show an "Unknown publisher" warning. Click **More info** → **Run anyway** to continue.

### Linux

1. Download the latest AppImage, DEB, or RPM from [Releases](https://whisprtypr.app).
2. **AppImage:** `chmod +x Whisprtypr_*.AppImage` then `./Whisprtypr_*.AppImage`
3. **DEB:** `sudo apt install ./whisprtypr_*.deb`
4. **RPM:** `sudo dnf install ./whisprtypr-*.rpm`
5. Complete the setup wizard and start dictating.

> **Note:** PulseAudio or ALSA is required for microphone access on Linux.

## Architecture

| Layer | Technology |
|-------|-----------|
| Desktop shell | Tauri v2 (windowing, menus, updater, permissions, packaging) |
| Frontend | React 19, TypeScript, Tailwind CSS |
| Backend | Rust (audio recording, transcription orchestration, hotkeys, history, cursor insertion) |
| Local engines | Whisper on macOS and Windows; Parakeet on Apple Silicon |
| Cloud engines | Groq, OpenAI, Deepgram, Mistral |

## Build from Source

Prerequisites:

- Node.js LTS and pnpm
- Rust stable toolchain
- Tauri v2 platform prerequisites for your operating system

```sh
git clone https://github.com/The-Whisprtypr/whisprtypr.git
cd whisprtypr
pnpm install
cd src-tauri
cargo build
```

Useful checks:

```sh
pnpm run typecheck
cd src-tauri && cargo test -j 1
```

## How It Works

1. **Record** with push-to-talk or toggle mode.
2. **Transcribe** speech into text with your selected model.
3. **Clean up** the output with optional post-processing.
4. **Insert or copy** the result into your current workflow.
5. **Reuse** through searchable local history.

## Who It Is For

- Writers capturing ideas quickly
- Developers dictating notes, paths, commands, and technical text
- Professionals replying to messages and drafting documents
- Support and operations teams handling repetitive text entry
- Privacy-conscious users who prefer local-first tools

## Contributing and Support

- Report reproducible bugs through [GitHub Issues](https://github.com/The-Whisprtypr/whisprtypr/issues).
- Review existing issues and pull requests before starting overlapping work.
- Read [CONTRIBUTING.md](CONTRIBUTING.md), [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md), and [SECURITY.md](SECURITY.md) before opening an issue or pull request.

## License

Whisprtypr is licensed under the [GNU Affero General Public License v3.0](LICENSE).

This repository includes vendored third-party code under `src-tauri/vendor/`. Those components keep their own upstream license files where provided.
