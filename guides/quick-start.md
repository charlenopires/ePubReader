# ⚡ Quick Start - ePub Reader Library

## 🚀 In 5 Minutes

### 1. Automatic Installation
```bash
./setup.sh
```

### 2. Run Application
```bash
cargo tauri dev
```

### 3. Add First Book
- Click **"Add Book"**
- Select an `.epub` file
- Wait for processing

### 4. Translate (Optional)
- Select language in header: **"Translate to:"**
- Click **"Translate"** on book card
- Wait for completion (green status)

### 5. Read
- Click **"Read"** on book card
- Use arrows ← → to navigate
- Adjust font with **A-** / **A+**

---

## 🔧 Quick Troubleshooting

### Ollama not running?
```bash
ollama serve
ollama pull llama3.1:8b
```

### Book not appearing?
- Check if it's a valid `.epub` file
- Wait for complete processing

### Translation too slow?
```bash
ollama pull llama2:7b  # Faster model
```

### Error opening book?
- Check if you have a default browser configured
- Try clicking "Read" again

---

## 📱 Quick Interface

```
┌─────────────────────────────────────────────────────────────┐
│ 📚 ePub Reader    [Translate to: English ▼]    [+ Add Book] │
├─────────────────────────────────────────────────────────────┤
│  ┌─────┐  ┌─────┐  ┌─────┐                                  │
│  │📖   │  │📖   │  │📖   │                                  │
│  │Book │  │Book │  │Book │                                  │
│  │ 1   │  │ 2   │  │ 3   │                                  │
│  │🟢   │  │🟡   │  │🔘   │  ← Status: Green=Translated     │
│  └─────┘  └─────┘  └─────┘     Yellow=Translating          │
│                                 Gray=Not translated         │
└─────────────────────────────────────────────────────────────┘
```

**Hover on books shows:**
- **"Read"** - Read the book
- **"Translate"** - Translate (if Ollama active)

---

## 🌍 Available Languages

| Code | Language | Code | Language |
|------|----------|------|----------|
| `pt` | Português | `en` | English |
| `es` | Español | `fr` | Français |
| `de` | Deutsch | `it` | Italiano |
| `ja` | 日本語 | `ko` | 한국어 |
| `zh` | 中文 | `ru` | Русский |
| `ar` | العربية | `hi` | हिन्दी |

---

## 📁 Where Data is Stored

```
~/.epubreader/ebooks/
├── library.db           # Database
├── [Book Name]/
│   ├── images/         # Covers and images
│   └── html/           # Reading version
```

---

## ⌨️ Reading Shortcuts

- **← →** - Navigate chapters
- **Home/Esc** - Back to index
- **A- / A+** - Adjust font
- **🌓** - Toggle theme
- **⛶** - Fullscreen

---

## 📞 Need Help?

👉 **Check the complete [User Guide](user-guide.md)**

**Common issues:**
- Ollama not running → `ollama serve`
- Slow translation → Use smaller model
- Book won't open → Check default browser

---

**🎉 Ready! Your smart digital library is working!**