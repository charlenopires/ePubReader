# ePub Reader Library

A modern ebook library with translation capabilities using Ollama AI.

## Features

- 📚 **Library Management**: Organize your EPUB books in a modern interface
- 🌍 **Smart Translation**: Translate books to different languages using Ollama
- 📖 **Integrated Reader**: Read your books in optimized HTML/CSS/JS
- 💾 **Local Storage**: Data saved in `~/.epubreader/ebooks`
- 🎨 **Modern Interface**: Dark and responsive design

## Prerequisites

1. **Rust** (version 1.70+)
2. **Node.js** (version 16+)
3. **Ollama** (for translation features)

### Ollama Installation

```bash
# macOS/Linux
curl -fsSL https://ollama.ai/install.sh | sh

# Windows
# Download from https://ollama.ai/download

# Start Ollama
ollama serve

# Install a recommended model
ollama pull llama3.1:8b
```

## Installation

### Quick Start
```bash
# Run the automated setup script
./setup.sh

# Start the application
cargo tauri dev
```

### Manual Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd epub-reader-library
```

2. Install Tauri dependencies:
```bash
cargo install tauri-cli
```

3. Run in development mode:
```bash
cargo tauri dev
```

4. For production build:
```bash
cargo tauri build
```

## Usage

### Adding Books

1. Click the "Add Book" button in the top right corner
2. Select an EPUB file from your system
3. The book will be processed and added to your library

### Translating Books

1. Make sure Ollama is running
2. Select the target language in the header selector
3. Click "Translate" on the book card
4. Wait for the translation process to complete

### Reading Books

1. Click "Read" on the book card
2. The book will open in your default browser
3. Use arrow keys to navigate between chapters
4. Use reading controls to adjust font and theme

## 📚 Documentation

### Quick Links
- 🚀 **[Quick Start Guide](guides/quick-start.md)** - Get up and running in 5 minutes
- 👤 **[User Guide](guides/user-guide.md)** - Complete feature documentation
- 🎨 **[Visual Guide](guides/visual-guide.md)** - Interface mockups and workflows
- 🔧 **[Troubleshooting](guides/troubleshooting.md)** - Common issues and solutions

### Full Documentation
Visit the **[guides/](guides/)** directory for comprehensive documentation including:
- Installation and setup instructions
- Feature tutorials and workflows
- Visual interface guides
- Troubleshooting and FAQ
- Configuration options

## Project Structure

```
epub-reader-library/
├── src/                    # Rust code (backend)
│   ├── main.rs            # Main entry point
│   ├── commands.rs        # Tauri commands
│   ├── database.rs        # SQLite database management
│   ├── epub_processor.rs  # EPUB file processing
│   ├── ollama_client.rs   # Ollama API client
│   └── models.rs          # Data structures
├── src-tauri/             # Tauri configuration
├── assets/                # Assets for generated books
├── guides/                # Documentation guides
├── index.html             # Main interface
├── styles.css             # Application styles
├── app.js                 # Main JavaScript
└── README.md              # This file
```

## Supported Languages

- 🇺🇸 English
- 🇧🇷 Português
- 🇪🇸 Español
- 🇫🇷 Français
- 🇩🇪 Deutsch
- 🇮🇹 Italiano
- 🇯🇵 日本語
- 🇰🇷 한국어
- 🇨🇳 中文
- 🇷🇺 Русский
- 🇸🇦 العربية
- 🇮🇳 हिन्दी

## Data Storage

Data is stored in:
- **macOS**: `~/Library/Application Support/.epubreader/ebooks/`
- **Linux**: `~/.local/share/.epubreader/ebooks/`
- **Windows**: `%APPDATA%\.epubreader\ebooks\`

Each book has:
- SQLite database with translated text
- Directory of extracted images
- Generated HTML/CSS/JS files

## Development

### Backend Structure (Rust)

- **Database**: SQLite with tables for books, chapters and images
- **EPUB Processing**: Text, metadata and image extraction
- **Ollama Integration**: HTTP client for AI translation
- **Tauri Commands**: Interface between frontend and backend

### Frontend Structure

- **Vanilla HTML/CSS/JS**: Simple and fast interface
- **Grid Layout**: Library visualization in grid
- **Modal System**: Dialogs for status and progress
- **Responsive Design**: Works on different screen sizes

## Contributing

1. Fork the project
2. Create a feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Known Issues

- Translation can be slow depending on the Ollama model used
- Some EPUB files with DRM are not supported
- The reading interface is basic (improvements planned)

## Roadmap

- [ ] Support for more formats (PDF, MOBI)
- [ ] Cloud synchronization
- [ ] Annotations and bookmarks
- [ ] Integrated reader in application
- [ ] Customizable themes
- [ ] Reading statistics

## Getting Help

- 📖 **[Read the guides](guides/)** for comprehensive documentation
- 🐛 **[Report issues](https://github.com/charlenopires/ePubReader/issues)** on GitHub
- 💬 **[Start a discussion](https://github.com/charlenopires/ePubReader/discussions)** for questions

---

**Enjoy your smart digital library! 📚✨**