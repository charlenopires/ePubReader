# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

ePubReader is an ePub reader application built in Rust with automatic translation capabilities. Users can specify their preferred reading language in settings, and content is automatically translated when opening ePub files.

## Technology Stack

- **Language**: Rust (2021 edition)
- **GUI Framework**: Tauri (recommended for cross-platform desktop app with web technologies)
  - Alternative: Slint for native UI or egui for immediate-mode GUI
- **ePub Parsing**: `epub` crate (primary library for ePub file reading)
- **Translation**: DeepL API via `deeplx` crate or Google Translate API via `rust-translate`
- **HTTP Client**: `reqwest` with async support
- **Async Runtime**: `tokio` with full features
- **Serialization**: `serde` for JSON handling

## Project Structure

```
src/
├── main.rs              # Application entry point
├── lib.rs               # Library root
├── gui/                 # GUI components and handlers
│   ├── mod.rs
│   ├── main_window.rs   # Main reading interface
│   ├── settings.rs      # Settings dialog
│   └── file_manager.rs  # File selection and management
├── epub/                # ePub handling
│   ├── mod.rs
│   ├── parser.rs        # ePub file parsing
│   ├── renderer.rs      # Content rendering
│   └── metadata.rs      # Book metadata extraction
├── translation/         # Translation services
│   ├── mod.rs
│   ├── deepl.rs         # DeepL API integration
│   ├── google.rs        # Google Translate integration
│   └── cache.rs         # Translation caching
├── config/              # Configuration management
│   ├── mod.rs
│   ├── settings.rs      # User preferences
│   └── storage.rs       # Local data storage
└── utils/               # Utility functions
    ├── mod.rs
    ├── text_processing.rs
    └── error.rs         # Error handling
```

## Core Dependencies

Add to `Cargo.toml`:

```toml
[dependencies]
tauri = { version = "1.0", features = ["api-all"] }
epub = "2.0"
deeplx = "1.0"  # or rust-translate for Google Translate
reqwest = { version = "0.11", features = ["json"] }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## Common Commands

```bash
# Development
cargo run               # Run in development mode
cargo check            # Check code without building
cargo clippy           # Run linter
cargo fmt              # Format code

# Building
cargo build --release  # Build optimized release
cargo test              # Run tests

# Tauri specific (if using Tauri)
cargo tauri dev         # Run Tauri development server
cargo tauri build       # Build Tauri application
```

## Core Features

1. **ePub File Management**
   - Open and parse ePub files using `epub` crate
   - Extract metadata (title, author, language, etc.)
   - Navigate through chapters and pages

2. **Translation Engine**
   - Detect source language from ePub metadata or content
   - Translate text chunks using DeepL or Google Translate API
   - Cache translations locally to avoid redundant API calls
   - Handle translation errors gracefully

3. **User Interface**
   - Reading view with translated content
   - Settings panel for language preferences
   - Progress tracking and bookmarks
   - File browser for ePub selection

4. **Settings Management**
   - Target language selection
   - Translation service preference (DeepL/Google)
   - API key configuration
   - Reading preferences (font size, theme)

## Architecture Notes

- Use async/await pattern for file I/O and API calls
- Implement translation caching to reduce API costs
- Design modular translation service for easy provider switching
- Consider progressive translation (translate as user reads)
- Handle large ePub files efficiently with streaming
- Implement proper error handling for network failures

## API Integration

### DeepL API
- Requires API key configuration
- Rate limits: 500,000 characters/month (free tier)
- Supports 30+ languages

### Google Translate API
- Requires Google Cloud project and API key
- Pay-per-use pricing model
- Supports 100+ languages

## Development Guidelines

- Follow Rust naming conventions (snake_case for functions/variables)
- Use `Result<T, E>` for error handling
- Implement `Debug` and `Clone` for custom types where appropriate
- Write unit tests for translation and parsing logic
- Document public APIs with doc comments