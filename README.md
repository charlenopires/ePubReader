# ePubReader

An ePub Reader built with Rust and Tauri that automatically translates content upon opening. Users can specify their preferred reading language in the settings, and all translated books are saved locally for offline reading.

## Features

- 📚 **ePub File Support** - Open and read standard ePub files
- 🌍 **Automatic Translation** - Translate books using Google Translate API
- 💾 **Local Storage** - Translated books are saved to `~/.epubreader/ebooks`
- ⚙️ **Settings Management** - Configure target language and API keys
- 📖 **Chapter Navigation** - Easy navigation between chapters
- 📱 **Modern UI** - Clean, responsive interface built with web technologies

## Prerequisites

Before building and running ePubReader, ensure you have:

1. **Rust** (latest stable version)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

2. **Node.js** (for Tauri frontend dependencies)
   ```bash
   # Using homebrew on macOS
   brew install node
   
   # Or download from https://nodejs.org/
   ```

3. **Tauri CLI**
   ```bash
   cargo install tauri-cli
   ```

4. **Google Translate API Key**
   - Get an API key from [Google Cloud Console](https://console.cloud.google.com/)
   - Enable the Google Translate API for your project

## Building the Application

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd ePubReader
   ```

2. **Configure Environment Variables (Optional)**
   ```bash
   # Copy the example environment file
   cp .env.example .env
   
   # Edit .env and add your Google Translate API key
   # GOOGLE_TRANSLATE_API_KEY=your_api_key_here
   ```

3. **Build the release executable**
   ```bash
   cd src-tauri
   cargo build --release
   ```

4. **The executable will be created at:**
   ```
   src-tauri/target/release/epubreader
   ```

## Running the Application

### Method 1: Direct Executable
```bash
./src-tauri/target/release/epubreader
```

### Method 2: Using Tauri CLI (Development)
```bash
cargo tauri dev
```

### Method 3: Build and Run Tauri Bundle
```bash
cargo tauri build
```

## Setup and Usage

1. **Launch the application**
   - Run the executable using one of the methods above

2. **Configure Google Translate API**
   
   You can configure the Google Translate API key in two ways:
   
   **Method A: Environment Variable (Recommended)**
   - Set `GOOGLE_TRANSLATE_API_KEY` in your `.env` file
   - The application will automatically use this key
   
   **Method B: Application Settings**
   - Click the "Settings" button in the sidebar
   - Enter your Google Translate API key manually
   - Select your preferred target language
   - Save the settings

3. **Open an ePub File**
   - Click "Open ePub File" button
   - Select an ePub file from your system
   - The book will load and begin automatic translation (if enabled)

4. **Navigate and Read**
   - Use Previous/Next buttons to navigate chapters
   - Use keyboard shortcuts: Arrow keys for navigation, Ctrl+T to translate current chapter
   - All translated content is automatically saved to `~/.epubreader/ebooks`

5. **Access Saved Books**
   - Previously translated books appear in the "Saved Books" section
   - Click any saved book to reload it instantly

## Environment Variables

The application supports the following environment variables:

| Variable | Description | Required | Default |
|----------|-------------|----------|---------|
| `GOOGLE_TRANSLATE_API_KEY` | Google Translate API key for automatic translation | Yes (for translation) | None |
| `RUST_LOG` | Controls logging verbosity | No | `info,epubreader=debug` |

### Environment Variable Setup

1. Copy the example file:
   ```bash
   cp .env.example .env
   ```

2. Edit `.env` and add your API key:
   ```bash
   GOOGLE_TRANSLATE_API_KEY=your_actual_api_key_here
   RUST_LOG=info,epubreader=debug
   ```

3. The application will automatically load these variables on startup

## Project Structure

```
ePubReader/
├── src/                    # Frontend assets (HTML, CSS, JS)
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── main.rs        # Application entry point
│   │   ├── epub_parser.rs # ePub file parsing
│   │   ├── translation.rs # Google Translate integration
│   │   ├── file_manager.rs# File management and storage
│   │   └── config.rs      # Settings management
│   └── Cargo.toml         # Rust dependencies
├── CLAUDE.md              # Development guidance
└── README.md              # This file
```

## Dependencies

### Rust Crates
- `tauri` - Desktop app framework
- `epub` - ePub file parsing
- `reqwest` - HTTP client for translation API
- `tokio` - Async runtime
- `serde` - Serialization
- `uuid` - Unique identifiers
- `dirs` - Directory management
- `anyhow` - Error handling
- `dotenvy` - Environment variable loading
- `tracing` - Structured logging
- `tracing-subscriber` - Logging configuration
- `tracing-appender` - File logging with rotation

## Troubleshooting

### Build Issues
- Ensure Rust is up to date: `rustup update`
- Clear build cache: `cargo clean`
- Verify Tauri dependencies are installed

### Runtime Issues
- Check that Google Translate API key is valid
- Ensure internet connection for translation features
- Verify ePub file is not corrupted

### Translation Issues
- Confirm API key has proper permissions
- Check Google Cloud Console for API usage limits
- Verify the target language code is supported

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.
