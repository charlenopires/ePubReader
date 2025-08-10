# 🔧 ePub Reader Library - Troubleshooting Guide

## 🚨 Common Issues and Solutions

### 1. 🤖 Ollama Issues

#### ❌ "Ollama is not running"
**Symptoms:**
- Modal appears on startup
- "Translate" button doesn't appear on books
- Error when trying to translate

**Solutions:**
```bash
# 1. Check if Ollama is installed
ollama --version
# 2. Start Ollama service
ollama serve
# 3. In another terminal, check if it's working
curl http://localhost:11434/api/tags
# 4. If it doesn't work, reinstall Ollama
curl -fsSL https://ollama.ai/install.sh | sh
```

#### ❌ "No suitable model available for translation"
**Symptoms:**
- Ollama is running but translation fails
- Error about model not available

**Solutions:**
```bash
# 1. List installed models
ollama list
# 2. Install recommended model
ollama pull llama3.1:8b
# 3. Check if model was installed
ollama list | grep llama3.1
# 4. Test the model
ollama run llama3.1:8b "Hello, how are you?"
```

#### ❌ Ollama consumes too much memory
**Symptoms:**
- System becomes slow during translation
- Application freezes or closes

**Solutions:**
```bash
# 1. Use smaller model
ollama pull llama3:8b
# 2. Or even smaller model
ollama pull mistral:7b
# 3. Limit memory usage (Linux/macOS)
OLLAMA_MAX_LOADED_MODELS=1 ollama serve
# 4. Close other applications during translation
```

### 2. 📚 Book Issues

#### ❌ Error adding EPUB book
**Symptoms:**
- Error when selecting EPUB file
- Application freezes when processing book
- Book doesn't appear in library

**Diagnosis:**
```bash
# 1. Check if it's a valid EPUB file
file your_book.epub
# Should show: "EPUB document"
# 2. Try opening in another EPUB reader
# - Calibre
# - Adobe Digital Editions
# - Apple Books
```

**Solutions:**
```bash
# 1. Check file permissions
ls -la your_book.epub
# 2. Try with another EPUB file
# Download a free test EPUB
# 3. Check disk space
df -h ~/.epubreader/ebooks/
# 4. Check application logs
RUST_LOG=debug cargo tauri dev
```

#### ❌ DRM-protected book doesn't work
**Symptoms:**
- Error processing purchased book
- Message about copy protection

**Solutions:**
- ⚠️ **Important**: Only DRM-free books are supported
- Use books from Project Gutenberg (free)
- Remove DRM with legal tools (if you own the book)
- Convert other formats to EPUB with Calibre

#### ❌ Images don't appear in book
**Symptoms:**
- Text appears but images don't load
- Empty spaces where images should be

**Solutions:**
```bash
# 1. Check if images were extracted
ls ~/.epubreader/ebooks/"Book Name"/images/
# 2. Check folder permissions
ls -la ~/.epubreader/ebooks/"Book Name"/
# 3. Reprocess the book (remove and add again)
```

### 3. 🌍 Translation Issues

#### ❌ Translation fails or stops midway
**Symptoms:**
- Status stays "In Progress" indefinitely
- Status changes to "Failed"
- Incomplete translation

**Solutions:**
```bash
# 1. Check if Ollama is still running
ps aux | grep ollama
# 2. Check Ollama logs
ollama logs
# 3. Restart Ollama
pkill ollama
ollama serve
# 4. Try with smaller model
ollama pull mistral:7b
# 5. Check disk space
df -h ~/.epubreader/ebooks/
```

#### ❌ Poor translation quality
**Symptoms:**
- Translated text doesn't make sense
- Mixed languages
- Lost formatting

**Solutions:**
```bash
# 1. Use larger and more recent model
ollama pull llama3.1:8b
# 2. Check if source language is correct
# 3. Try translating chapter by chapter manually
# 4. Adjust translation prompt (for developers)
```

### 4. 📖 Reader Issues

#### ❌ Book doesn't open in browser
**Symptoms:**
- Clicking "Read" does nothing
- Error opening HTML file
- Blank page in browser

**Solutions:**
```bash
# 1. Check if HTML file was generated
ls ~/.epubreader/ebooks/"Book Name"/html/
# 2. Try opening manually
open ~/.epubreader/ebooks/"Book Name"/html/index.html
# 3. Check default browser
# macOS: Safari, Chrome, Firefox
# Linux: firefox, chromium
# Windows: Edge, Chrome, Firefox
# 4. Check permissions
chmod -R 755 ~/.epubreader/ebooks/"Book Name"/html/
```

#### ❌ Broken formatting in reader
**Symptoms:**
- Text without formatting
- CSS doesn't load
- Messy layout

**Solutions:**
```bash
# 1. Check if CSS files exist
ls ~/.epubreader/ebooks/"Book Name"/html/styles.css
# 2. Check CSS content
cat ~/.epubreader/ebooks/"Book Name"/html/styles.css
# 3. Regenerate book HTML
# (remove html folder and click "Read" again)
rm -rf ~/.epubreader/ebooks/"Book Name"/html/
```

#### ❌ Keyboard navigation doesn't work
**Symptoms:**
- Arrow keys don't change chapters
- Shortcuts don't respond

**Solutions:**
- Click on page to give focus
- Check if JavaScript is enabled
- Try in another browser
- Check browser console (F12)

### 5. 💾 Database Issues

#### ❌ "Database is locked"
**Symptoms:**
- Error adding books
- Application doesn't start
- Data doesn't save

**Solutions:**
```bash
# 1. Close all application instances
pkill epub-reader-library
# 2. Check if database is not corrupted
sqlite3 ~/.epubreader/ebooks/library.db "PRAGMA integrity_check;"
# 3. Backup and recreate database
cp ~/.epubreader/ebooks/library.db ~/library_backup.db
rm ~/.epubreader/ebooks/library.db
# Restart application
```

#### ❌ Lost or corrupted data
**Symptoms:**
- Books disappeared from library
- Lost translations
- Error accessing database

**Solutions:**
```bash
# 1. Check if files still exist
ls ~/.epubreader/ebooks/
# 2. Try repairing SQLite database
sqlite3 ~/.epubreader/ebooks/library.db ".recover" > recovered.sql
sqlite3 new_library.db < recovered.sql
# 3. Restore from backup
cp ~/library_backup.db ~/.epubreader/ebooks/library.db
```

### 6. 🖥️ System Issues

#### ❌ Application doesn't start
**Symptoms:**
- Error running `cargo tauri dev`
- Window doesn't open
- Immediate crash

**Solutions:**
```bash
# 1. Check dependencies
cargo check
# 2. Clear Cargo cache
cargo clean
# 3. Update dependencies
cargo update
# 4. Check logs
RUST_LOG=debug cargo tauri dev
# 5. Check supported operating system
uname -a
```

#### ❌ Poor performance
**Symptoms:**
- Slow interface
- Very slow translation
- High CPU/memory usage

**Solutions:**
```bash
# 1. Close other applications
# 2. Use smaller Ollama model
ollama pull mistral:7b
# 3. Check system resources
# macOS: Activity Monitor
# Linux: htop, top
# Windows: Task Manager
# 4. Optimized build
cargo tauri build --release
```

### 7. 🔍 Diagnostic Tools

#### Check General Status
```bash
#!/bin/bash
echo "=== ePub Reader Library Diagnostics ==="
echo
echo "1. Rust Version:"
rustc --version
echo "2. Cargo Version:"
cargo --version
echo "3. Ollama Status:"
if command -v ollama &> /dev/null; then
    echo "✅ Ollama installed"
    if pgrep -x "ollama" > /dev/null; then
        echo "✅ Ollama running"
        echo "Models:"
        ollama list
    else
        echo "❌ Ollama not running"
    fi
else
    echo "❌ Ollama not installed"
fi
echo "4. Data Directory:"
if [ -d ~/.epubreader/ebooks ]; then
    echo "✅ Data directory exists"
    echo "Size: $(du -sh ~/.epubreader/ebooks | cut -f1)"
    echo "Books: $(ls ~/.epubreader/ebooks | grep -v library.db | wc -l)"
else
    echo "❌ Data directory missing"
fi
echo "5. Database:"
if [ -f ~/.epubreader/ebooks/library.db ]; then
    echo "✅ Database exists"
    echo "Size: $(ls -lh ~/.epubreader/ebooks/library.db | cut -d' ' -f5)"
else
    echo "❌ Database missing"
fi
echo "6. System Resources:"
echo "Memory: $(free -h | grep Mem | awk '{print $3 "/" $2}')"
echo "Disk: $(df -h ~ | tail -1 | awk '{print $3 "/" $2 " (" $5 " used)"}')"
```

#### Detailed Logs
```bash
# Run with maximum logs
RUST_LOG=trace cargo tauri dev 2>&1 | tee debug.log
# Check Ollama logs
ollama logs 2>&1 | tee ollama.log
# System logs (Linux)
journalctl -u ollama -f
# System logs (macOS)
log stream --predicate 'process == "ollama"'
```

### 8. 🆘 When to Ask for Help

Before reporting a problem, collect the following information:

#### System Information
```bash
# Operating system
uname -a
# Rust version
rustc --version
# Ollama version
ollama --version
# Installed models
ollama list
```

#### Error Information
- Exact error message
- Steps to reproduce
- Log files
- Screenshots (if applicable)

#### Book Information
- File format (EPUB)
- File size
- File origin (purchased, free, etc.)
- If it worked before

### 9. 🔄 Complete Reset

If nothing else works, complete reset:

```bash
# 1. Backup data
cp -r ~/.epubreader/ebooks ~/backup-ebooks-$(date +%Y%m%d)
# 2. Stop all processes
pkill ollama
pkill epub-reader-library
# 3. Remove application data
rm -rf ~/.epubreader/
# 4. Reinstall Ollama
curl -fsSL https://ollama.ai/install.sh | sh
# 5. Reinstall model
ollama serve &
sleep 5
ollama pull llama3.1:8b
# 6. Clear Cargo cache
cargo clean
# 7. Recompile application
cargo tauri dev
```

### 10. 📞 Support

If the problem persists:

1. **Check** if it's not a known issue in the README
2. **Collect** all diagnostic information
3. **Create** an issue in the project repository
4. **Include** logs, screenshots and steps to reproduce

---

## 💡 Prevention Tips

- **Regular backups** of `~/.epubreader/ebooks/` folder
- **Keep Ollama updated** regularly
- **Use valid EPUB files** without DRM
- **Monitor system resources** during translations
- **Close other heavy applications** during intensive use

**Remember**: Most problems can be solved by restarting Ollama and the application! 🔄