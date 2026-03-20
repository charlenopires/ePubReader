# ePub Reader Library

A modern desktop ePub reader with AI-powered translation capabilities, built with **Tauri 2.0**, **Rust**, **React**, and **TypeScript**.

## Features

- **In-app ePub reader** with customizable font, spacing, and theme (light / dark / sepia)
- **AI translation** of entire books via [LM Studio](https://lmstudio.ai) (OpenAI-compatible local API)
- **Apple Books-inspired UI** with dark mode, sidebar navigation, and book grid
- **Chapter-by-chapter translation** with real-time progress events
- **12 supported languages**: English, Portuguese, Spanish, French, German, Italian, Japanese, Korean, Chinese, Russian, Arabic, Hindi
- **Cover extraction** and metadata parsing from ePub files
- **SQLite** local library database

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop runtime | Tauri 2.0 |
| Backend | Rust (tokio, sqlx, reqwest) |
| Frontend | React 19 + TypeScript + Vite |
| UI components | shadcn/ui + TailwindCSS v4 |
| State management | Zustand |
| AI translation | LM Studio (OpenAI-compatible API) |
| Database | SQLite |
| ePub parsing | epub crate |

## Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Bun](https://bun.sh/) (package manager)
- [LM Studio](https://lmstudio.ai/) (for translation features)

## Getting Started

### 1. Clone and install dependencies

```bash
git clone https://github.com/fazapp/ePubReader.git
cd ePubReader
bun install
cd ui && bun install && cd ..
```

### 2. Run in development mode

```bash
bun run dev
```

This starts the Vite dev server on `http://localhost:5173` and launches the Tauri window.

### 3. Build for production

```bash
bun run build
```

## Translation Setup

1. Install and open [LM Studio](https://lmstudio.ai/)
2. Download a model (e.g., `qwen/qwen3.5-9b`)
3. Start the local server (default: `http://localhost:1234`)
4. The app auto-detects LM Studio and shows available models

### API Integration

LM Studio exposes an OpenAI-compatible API:

| Endpoint | Purpose |
|----------|---------|
| `GET /v1/models` | List loaded models |
| `POST /v1/chat/completions` | Translate text via chat completions |

## Project Structure

```
ePubReader/
├── src/                          # Rust backend
│   ├── main.rs                   # Tauri app entry + AppState
│   ├── commands.rs               # Tauri IPC commands
│   ├── lmstudio_client.rs        # LM Studio OpenAI-compatible client
│   ├── models.rs                 # Data structures
│   ├── database.rs               # SQLite database layer
│   └── epub_processor.rs         # ePub parsing + HTML generation
├── ui/                           # React frontend (Vite)
│   └── src/
│       ├── App.tsx               # Root component
│       ├── lib/tauri.ts          # Tauri API wrapper + types
│       ├── stores/app-store.ts   # Zustand state management
│       ├── hooks/                # React hooks
│       │   ├── use-books.ts      # Book CRUD operations
│       │   ├── use-lmstudio.ts   # LM Studio connection
│       │   ├── use-reader.ts     # Reader state + navigation
│       │   └── use-translation.ts # Translation progress
│       ├── components/
│       │   ├── layout/           # Sidebar, Header, MainLayout
│       │   ├── library/          # BookCard, BookGrid, EmptyState
│       │   ├── reader/           # ReaderView, ReaderSettings
│       │   └── translation/      # TranslateDialog
│       └── pages/                # Library, ReadingNow
├── assets/                       # Book reader assets (CSS/JS)
├── tests/                        # Integration tests
├── Cargo.toml                    # Rust dependencies
├── tauri.conf.json               # Tauri configuration
└── package.json                  # Root scripts
```

## Tauri Commands

| Command | Description |
|---------|-------------|
| `check_lmstudio_status` | Check if LM Studio is running |
| `get_available_models` | List loaded models |
| `set_lmstudio_model` | Select model for translation |
| `get_books` | List all books in library |
| `add_book` | Import ePub file |
| `delete_book` | Remove book from library |
| `get_book_content` | Get book chapters |
| `translate_book` | Translate entire book (emits progress events) |
| `get_supported_languages` | List 12 supported languages |

## Data Storage

Data is stored in `~/.epubreader/ebooks/`:
- SQLite database (`library.db`) with book metadata, chapters, and translations
- Extracted cover images and chapter images
- Generated HTML files for external reading

## License

MIT
