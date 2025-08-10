# 📚 User Guide - ePub Reader Library

## Table of Contents
1. [Installation and Setup](#installation-and-setup)
2. [First Run](#first-run)
3. [Main Interface](#main-interface)
4. [Adding Books](#adding-books)
5. [Translating Books](#translating-books)
6. [Reading Books](#reading-books)
7. [Advanced Settings](#advanced-settings)
8. [Troubleshooting](#troubleshooting)
9. [Tips and Tricks](#tips-and-tricks)

---

## 🚀 Installation and Setup

### Prerequisites
Before starting, make sure you have installed:

- **Rust** (version 1.70 or higher)
- **Node.js** (version 16 or higher)
- **Ollama** (for translation features)

### Automatic Installation

1. **Run the installation script:**
   ```bash
   ./setup.sh
   ```
   
   This script will:
   - ✅ Check if Rust and Node.js are installed
   - 📦 Install Tauri CLI
   - 🤖 Install and configure Ollama
   - 📥 Download the recommended translation model

### Manual Installation

If you prefer to install manually:

1. **Install Tauri CLI:**
   ```bash
   cargo install tauri-cli --locked
   ```

2. **Install Ollama:**
   ```bash
   # macOS/Linux
   curl -fsSL https://ollama.ai/install.sh | sh
   
   # Windows - download from https://ollama.ai/download
   ```

3. **Start Ollama:**
   ```bash
   ollama serve
   ```

4. **Install translation model:**
   ```bash
   ollama pull llama3.1:8b
   ```

---

## 🎯 First Run

### Running the Application

1. **Development Mode:**
   ```bash
   cargo tauri dev
   ```

2. **Production Build:**
   ```bash
   cargo tauri build
   ```

### Ollama Verification

When starting the application for the first time, you'll see one of the following situations:

#### ✅ **Ollama Working**
- The application will start normally
- You'll see the empty library
- Translation features will be available

#### ❌ **Ollama Not Detected**
- A modal will appear: "Ollama Setup Required"
- Available options:
  - **"Check Again"**: Checks again if Ollama is running
  - **"Continue Without Translation"**: Uses the app without translation

---

## 🖥️ Main Interface

### Application Layout

```
┌─────────────────────────────────────────────────────────────┐
│ 📚 ePub Reader    [Translate to: English ▼]    [+ Add Book] │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐                        │
│  │📖   │  │📖   │  │📖   │  │📖   │                        │
│  │Book │  │Book │  │Book │  │Book │                        │
│  │ 1   │  │ 2   │  │ 3   │  │ 4   │                        │
│  └─────┘  └─────┘  └─────┘  └─────┘                        │
│                                                             │
│  ┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐                        │
│  │📖   │  │📖   │  │📖   │  │📖   │                        │
│  │Book │  │Book │  │Book │  │Book │                        │
│  │ 5   │  │ 6   │  │ 7   │  │ 8   │                        │
│  └─────┘  └─────┘  └─────┘  └─────┘                        │
└─────────────────────────────────────────────────────────────┘
```

### Interface Elements

#### **Header (Top Bar)**
- **Logo**: ePub Reader (left corner)
- **Language Selector**: "Translate to:" (center)
- **Add Book Button**: Add new books (right)

#### **Books Grid**
Each book card shows:
- **Cover**: Book image or default icon
- **Translation Status**: Colored badge in top right corner
- **Title**: Book name
- **Author**: Author name
- **Language**: Original book language

#### **Translation Status**
- 🔘 **Not Started**: Gray - Translation not started
- 🟡 **In Progress**: Yellow - Translation in progress
- 🟢 **Completed**: Green - Translation completed
- 🔴 **Failed**: Red - Translation failed

---

## 📖 Adding Books

### Addition Process

1. **Click the "Add Book" button** (top right corner)

2. **Select an EPUB file:**
   - Browse your files
   - Select a file with `.epub` extension
   - Click "Open"

3. **Automatic Processing:**
   - ⏳ The app will show "Loading your library..."
   - 📊 The book will be processed automatically:
     - Metadata extraction (title, author, language)
     - Cover extraction
     - Chapter division
     - Image extraction
     - Database structure creation

4. **Result:**
   - ✅ The book will appear in your library
   - 📁 A directory will be created at `~/.epubreader/ebooks/[Book Name]/`

### Created Structure

For each added book:

```
~/.epubreader/ebooks/[Book Name]/
├── images/
│   ├── cover.jpg           # Extracted cover
│   ├── chapter_0_0.jpg     # Chapter 0 images
│   ├── chapter_0_1.jpg
│   └── ...
└── [SQLite database with text and metadata]
```

### Supported Formats

- ✅ **EPUB**: Fully supported
- ❌ **PDF**: Not supported (planned for future versions)
- ❌ **MOBI**: Not supported (planned for future versions)

---

## 🌍 Translating Books

### Translation Prerequisites

1. **Ollama must be running:**
   ```bash
   ollama serve
   ```

2. **Translation model installed:**
   ```bash
   ollama pull llama3.1:8b
   ```

### Translation Process

#### **1. Select Target Language**
- In the header, click the "Translate to:" selector
- Choose the desired language from the list

#### **2. Start Translation**
- Hover over a book card
- Click the **"Translate"** button that appears

#### **3. Monitor Progress**
- A "Translating Book" modal will appear
- Progress bar will show advancement
- Text status will indicate current step

#### **4. Completion**
- ✅ Status will change to "Completed" (green)
- 📚 The translated book will be available for reading

### Available Languages

| Code | Native Name | English Name |
|------|-------------|--------------|
| `en` | English | English |
| `pt` | Português | Portuguese |
| `es` | Español | Spanish |
| `fr` | Français | French |
| `de` | Deutsch | German |
| `it` | Italiano | Italian |
| `ja` | 日本語 | Japanese |
| `ko` | 한국어 | Korean |
| `zh` | 中文 | Chinese |
| `ru` | Русский | Russian |
| `ar` | العربية | Arabic |
| `hi` | हिन्दी | Hindi |

### Translation Time

Time varies according to:
- **Book size**: Larger books take longer
- **Model used**: Larger models are slower but more accurate
- **Hardware**: More powerful CPU/GPU speeds up the process

**Typical estimates:**
- Small book (100 pages): 5-15 minutes
- Medium book (300 pages): 15-45 minutes
- Large book (500+ pages): 45+ minutes

---

## 📚 Reading Books

### Opening a Book

1. **Hover over a book card**
2. **Click the "Read" button**
3. **The book will open in your default browser**

### Reader Interface

#### **Home Page (Cover)**
```
┌─────────────────────────────────────┐
│              [COVER]                │
│                                     │
│           Book Title                │
│            Author Name              │
│                                     │
│         Chapter Index               │
│         • Chapter 1                 │
│         • Chapter 2                 │
│         • ...                       │
│                                     │
│        [Start Reading]              │
└─────────────────────────────────────┘
```

#### **Chapter Page**
```
┌─────────────────────────────────────┐
│ [← Previous] [Contents] [Next →]    │
├─────────────────────────────────────┤
│                                     │
│         Chapter Title               │
│                                     │
│  Chapter content with formatted     │
│  text, paragraphs and images        │
│  inserted in correct positions.     │
│                                     │
│  [Chapter image, if any]            │
│                                     │
│  More chapter text...               │
│                                     │
├─────────────────────────────────────┤
│ ████████████░░░░░░░░░░░░░░░░ 60%    │
└─────────────────────────────────────┘
```

### Navigation Controls

#### **Keyboard**
- **← (Left Arrow)**: Previous chapter
- **→ (Right Arrow)**: Next chapter
- **Home**: Back to index
- **Escape**: Back to index

#### **Mouse/Touch**
- **Navigation buttons**: Previous, Contents, Next
- **Index links**: Click to go directly to chapter

### Reading Controls

In the top right corner of each chapter:

```
┌─────────────────┐
│ [A-] [A+] [🌓] [⛶] │
└─────────────────┘
```

- **A-**: Decrease font size
- **A+**: Increase font size
- **🌓**: Toggle theme (light/dark)
- **⛶**: Fullscreen

### Automatic Features

#### **Position Saving**
- ✅ Reading position saved automatically
- ✅ Resumes where you left off when reopening
- ✅ Works individually per book

#### **Progress Bar**
- 📊 Shows progress in current chapter
- 🔄 Updates as you scroll the page

#### **Responsiveness**
- 📱 Works on different screen sizes
- 🖥️ Optimized for desktop and mobile

---

## ⚙️ Advanced Settings

### Ollama Configuration

#### **Recommended Models (in order of preference):**
1. `llama3.1:8b` - Best quality/speed
2. `llama3:8b` - Good alternative
3. `llama2:7b` - Faster, lower quality
4. `mistral:7b` - Fast alternative

#### **Installing Specific Models:**
```bash
# Recommended model
ollama pull llama3.1:8b

# Faster model
ollama pull llama2:7b

# Multilingual model
ollama pull mistral:7b
```

#### **Performance Settings:**
```bash
# For machines with more RAM
ollama pull llama3.1:13b

# For machines with fewer resources
ollama pull llama3.1:8b-q4_0
```

### Storage Configuration

#### **Data Location:**
- **macOS**: `~/Library/Application Support/.epubreader/ebooks/`
- **Linux**: `~/.local/share/.epubreader/ebooks/`
- **Windows**: `%APPDATA%\.epubreader\ebooks\`

#### **Data Structure:**
```
.epubreader/ebooks/
├── library.db              # Main SQLite database
├── [Book 1]/
│   ├── images/             # Extracted images
│   └── html/               # HTML version (generated on demand)
├── [Book 2]/
│   ├── images/
│   └── html/
└── ...
```

### Development Configuration

#### **Environment Variables:**
```bash
# Log level
export RUST_LOG=info

# Tauri configuration
export TAURI_CONFIG=tauri.conf.json
```

#### **Database Configuration:**
The SQLite database is created automatically with the following tables:
- `books` - Book metadata
- `chapters` - Chapter content
- `images` - Image references
- `settings` - Application settings

---

## 🔧 Troubleshooting

### Common Issues

#### **1. "Ollama is not running"**

**Symptoms:**
- Modal appears on startup
- "Translate" button doesn't appear on books

**Solutions:**
```bash
# Check if Ollama is installed
ollama --version

# Start Ollama
ollama serve

# Check if it's running
curl http://localhost:11434/api/tags
```

#### **2. "No suitable model available"**

**Symptoms:**
- Translation fails immediately
- Error about model not found

**Solutions:**
```bash
# Install recommended model
ollama pull llama3.1:8b

# Check installed models
ollama list

# Test model
ollama run llama3.1:8b "Hello, how are you?"
```

#### **3. Book doesn't appear after adding**

**Symptoms:**
- EPUB file selected but doesn't appear in library
- Infinite loading

**Solutions:**
1. **Check file format:**
   - Make sure it's a valid `.epub` file
   - Test opening in another EPUB reader

2. **Check logs:**
   ```bash
   # Run with detailed logs
   RUST_LOG=debug cargo tauri dev
   ```

3. **Check permissions:**
   - Make sure the app can write to `~/.epubreader/`

#### **4. Translation very slow**

**Symptoms:**
- Translation takes hours
- System becomes slow during translation

**Solutions:**
1. **Use smaller model:**
   ```bash
   ollama pull llama2:7b
   ```

2. **Check system resources:**
   - Available RAM (minimum 8GB recommended)
   - CPU usage during translation

3. **Configure Ollama:**
   ```bash
   # Limit CPU usage
   export OLLAMA_NUM_PARALLEL=1
   ```

#### **5. Error opening book for reading**

**Symptoms:**
- Clicking "Read" doesn't open anything
- Browser error

**Solutions:**
1. **Check default browser:**
   - Make sure you have a browser configured

2. **Check HTML files:**
   ```bash
   # Check if HTML was generated
   ls ~/.epubreader/ebooks/[Book Name]/html/
   ```

3. **Regenerate HTML:**
   - Remove the book's `html` folder
   - Click "Read" again

### Logs and Debugging

#### **Enable Detailed Logs:**
```bash
# Development
RUST_LOG=debug cargo tauri dev

# Production
RUST_LOG=info ./target/release/epub-reader-library
```

#### **Log Locations:**
- **Console**: During development
- **System**: Operating system logs
- **File**: (if configured) `~/.epubreader/logs/`

### Cleanup and Maintenance

#### **Clear Cache:**
```bash
# Remove generated HTML files
rm -rf ~/.epubreader/ebooks/*/html/

# Clear Ollama cache
ollama rm [model]
ollama pull [model]
```

#### **Complete Reset:**
```bash
# CAUTION: Removes all data
rm -rf ~/.epubreader/
```

---

## 💡 Tips and Tricks

### Performance Optimization

#### **1. Ollama Model Choice**
- **For speed**: `llama2:7b` or `mistral:7b`
- **For quality**: `llama3.1:8b` or `llama3.1:13b`
- **For specific languages**: Specialized models

#### **2. Space Management**
- Translations take additional space
- Consider translating only books you'll read
- Remove old translations if necessary

#### **3. Library Organization**
- Use descriptive names for EPUB files
- Organize by author/series before importing
- Consider creating backups of original files

### Recommended Workflow

#### **For Casual Reading:**
1. Add books as needed
2. Translate only when you'll read
3. Use fast model (`llama2:7b`)

#### **For Intensive Reading:**
1. Add multiple books at once
2. Set up batch translation (future)
3. Use high-quality model (`llama3.1:8b`)

#### **For Development:**
1. Use development mode (`cargo tauri dev`)
2. Enable detailed logs (`RUST_LOG=debug`)
3. Test with small books first

### Shortcuts and Productivity

#### **Fast Navigation:**
- Use arrow keys to navigate between chapters
- `Home` to quickly return to index
- `Escape` as alternative to `Home`

#### **Comfortable Reading:**
- Adjust font size before starting
- Use fullscreen mode for immersion
- Toggle theme according to ambient lighting

#### **Translation Management:**
- Translate books in priority order
- Monitor progress through colored status
- Cancel unnecessary translations

### Integration with Other Systems

#### **Backup and Sync:**
```bash
# Library backup
tar -czf library-backup.tar.gz ~/.epubreader/

# Restore backup
tar -xzf library-backup.tar.gz -C ~/
```

#### **Data Export:**
- HTML files can be copied
- SQLite database can be queried directly
- Images are organized by book

---

## 📞 Support and Community

### Reporting Issues

1. **Check this guide first**
2. **Collect information:**
   - Operating system version
   - Rust/Tauri version
   - Error logs
   - Steps to reproduce

3. **Open an issue in the repository**

### Contributions

- Fork the project
- Implement improvements
- Test thoroughly
- Open Pull Request

### Future Roadmap

- [ ] Support for PDF and MOBI
- [ ] Cloud synchronization
- [ ] Annotations and bookmarks
- [ ] Integrated reader in application
- [ ] Customizable themes
- [ ] Reading statistics
- [ ] Batch translation
- [ ] Better offline support

---

## 📝 Conclusion

The ePub Reader Library is a powerful tool for managing and translating your digital library. With this guide, you should be able to:

- ✅ Install and configure the application
- ✅ Add and organize your books
- ✅ Translate books to your preferred language
- ✅ Read comfortably with advanced controls
- ✅ Solve common issues
- ✅ Optimize your reading experience

**Enjoy your new smart digital library!** 📚✨

---

*Last updated: December 2024*
*Guide version: 1.0*