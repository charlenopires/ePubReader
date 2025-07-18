const { invoke } = window.__TAURI__.tauri;
const { open } = window.__TAURI__.dialog;

let currentBook = null;
let currentChapter = 0;
let translatedContent = {};
let settings = {};

// Initialize app
document.addEventListener('DOMContentLoaded', async () => {
    await loadSettings();
    await loadSavedBooks();
    loadReadingStyle();
    updateFontSize();
});

// File operations
async function openFile() {
    try {
        const selected = await open({
            multiple: false,
            filters: [{
                name: 'ePub',
                extensions: ['epub']
            }]
        });

        if (selected) {
            await loadEpub(selected);
        }
    } catch (error) {
        showError('Failed to open file: ' + error);
    }
}

async function loadEpub(path) {
    try {
        showLoading('Loading ePub file...');
        
        currentBook = await invoke('open_epub', { path });
        currentChapter = 0;
        translatedContent = {};
        
        updateBookInfo();
        displayCurrentChapter();
        
        if (settings.auto_translate && settings.google_api_key) {
            await translateBook();
        }
        
    } catch (error) {
        showError('Failed to load ePub: ' + error);
    }
}

// Display functions
function updateBookInfo() {
    document.getElementById('bookTitle').textContent = currentBook.title;
    document.getElementById('bookAuthor').textContent = currentBook.author;
    document.getElementById('chapterInfo').textContent = `Chapter ${currentChapter + 1} of ${currentBook.chapters.length}`;
    
    const progress = ((currentChapter + 1) / currentBook.chapters.length) * 100;
    document.getElementById('progressFill').style.width = progress + '%';
    
    // Show reading interface
    document.getElementById('readingHeader').style.display = 'flex';
    document.getElementById('readingProgress').style.display = 'flex';
    
    // Enable/disable navigation buttons
    document.getElementById('prevBtn').disabled = currentChapter === 0;
    document.getElementById('nextBtn').disabled = currentChapter === currentBook.chapters.length - 1;
    document.getElementById('translateBtn').disabled = false;
    
    // Update reading stats
    updateReadingStats();
}

function displayCurrentChapter() {
    if (!currentBook || !currentBook.chapters[currentChapter]) {
        return;
    }
    
    const chapter = currentBook.chapters[currentChapter];
    const readingArea = document.getElementById('readingArea');
    
    // Use translated content if available, otherwise original
    const content = translatedContent[chapter.id] || chapter.content;
    
    // Format content as paragraphs
    const paragraphs = content.split('\n\n').filter(p => p.trim().length > 0);
    const formattedContent = paragraphs.map(p => `<p>${p.trim()}</p>`).join('');
    
    readingArea.innerHTML = `
        <div class="reading-content">
            <h1 class="chapter-title">${chapter.title}</h1>
            <div class="chapter-content">
                ${formattedContent}
            </div>
        </div>
    `;
    
    // Scroll to top
    readingArea.scrollTop = 0;
}

function showLoading(message) {
    document.getElementById('readingArea').innerHTML = `
        <div class="loading">
            <div class="loading-spinner"></div>
            ${message}
        </div>
    `;
}

function showError(message) {
    const readingArea = document.getElementById('readingArea');
    readingArea.innerHTML = `
        <div class="reading-content">
            <div class="error">${message}</div>
        </div>
    `;
}

function updateReadingStats() {
    if (!currentBook || !currentBook.chapters[currentChapter]) {
        return;
    }
    
    const chapter = currentBook.chapters[currentChapter];
    const content = translatedContent[chapter.id] || chapter.content;
    
    // Calculate word count
    const words = content.split(/\s+/).filter(word => word.length > 0).length;
    document.getElementById('wordsCount').textContent = `${words.toLocaleString()} words`;
    
    // Estimate reading time (average 200 words per minute)
    const readingTime = Math.ceil(words / 200);
    document.getElementById('readingTime').textContent = `${readingTime} min read`;
}

// Navigation
function previousChapter() {
    if (currentChapter > 0) {
        currentChapter--;
        updateBookInfo();
        displayCurrentChapter();
    }
}

function nextChapter() {
    if (currentChapter < currentBook.chapters.length - 1) {
        currentChapter++;
        updateBookInfo();
        displayCurrentChapter();
    }
}

// Translation
async function translateCurrent() {
    if (!currentBook || !settings.google_api_key) {
        showError('Please configure Google Translate API key in settings');
        return;
    }
    
    try {
        const chapter = currentBook.chapters[currentChapter];
        
        if (translatedContent[chapter.id]) {
            displayCurrentChapter();
            return;
        }
        
        showLoading('Translating chapter...');
        
        const translated = await invoke('translate_text', {
            text: chapter.content,
            targetLang: settings.target_language,
            apiKey: settings.google_api_key
        });
        
        translatedContent[chapter.id] = translated;
        displayCurrentChapter();
        
    } catch (error) {
        showError('Translation failed: ' + error);
    }
}

async function translateBook() {
    if (!currentBook || !settings.google_api_key) {
        return;
    }
    
    try {
        showLoading('Translating entire book...');
        
        for (let i = 0; i < currentBook.chapters.length; i++) {
            const chapter = currentBook.chapters[i];
            
            if (!translatedContent[chapter.id]) {
                const translated = await invoke('translate_text', {
                    text: chapter.content,
                    targetLang: settings.target_language,
                    apiKey: settings.google_api_key
                });
                
                translatedContent[chapter.id] = translated;
            }
            
            showLoading(`Translating... ${i + 1}/${currentBook.chapters.length} chapters`);
        }
        
        // Save translated book
        await invoke('save_translated_epub', {
            epubInfo: currentBook,
            translatedContent: translatedContent
        });
        
        await loadSavedBooks();
        displayCurrentChapter();
        
    } catch (error) {
        showError('Translation failed: ' + error);
    }
}

// Settings
async function loadSettings() {
    try {
        settings = await invoke('get_settings');
        
        document.getElementById('targetLang').value = settings.target_language;
        document.getElementById('apiKey').value = settings.google_api_key;
        document.getElementById('fontSize').value = settings.font_size;
        document.getElementById('fontSizeValue').textContent = settings.font_size + 'px';
        document.getElementById('autoTranslate').checked = settings.auto_translate;
        
    } catch (error) {
        console.error('Failed to load settings:', error);
        settings = {
            target_language: 'en',
            google_api_key: '',
            auto_translate: true,
            font_size: 16,
            theme: 'light'
        };
    }
}

function showSettings() {
    document.getElementById('settingsPanel').style.display = 'block';
}

function hideSettings() {
    document.getElementById('settingsPanel').style.display = 'none';
}

async function saveSettings() {
    try {
        settings.target_language = document.getElementById('targetLang').value;
        settings.google_api_key = document.getElementById('apiKey').value;
        settings.font_size = parseInt(document.getElementById('fontSize').value);
        settings.auto_translate = document.getElementById('autoTranslate').checked;
        
        await invoke('save_settings', { settings });
        updateFontSize();
        hideSettings();
        
    } catch (error) {
        showError('Failed to save settings: ' + error);
    }
}

function updateFontSize() {
    document.getElementById('readingArea').style.fontSize = settings.font_size + 'px';
    document.getElementById('fontSizeValue').textContent = settings.font_size + 'px';
}

// Reading Style Settings
let currentReadingStyle = {
    theme: 'light',
    fontFamily: 'serif',
    fontSize: 18,
    lineSpacing: 1.8,
    width: 680,
    margin: 48
};

function showReadingSettings() {
    const panel = document.getElementById('readingStylePanel');
    panel.style.display = 'block';
    setTimeout(() => panel.classList.add('active'), 10);
}

function hideReadingSettings() {
    const panel = document.getElementById('readingStylePanel');
    panel.classList.remove('active');
    setTimeout(() => panel.style.display = 'none', 300);
}

function setTheme(theme) {
    currentReadingStyle.theme = theme;
    
    // Update theme options
    document.querySelectorAll('.theme-option').forEach(option => {
        option.classList.remove('active');
    });
    document.querySelector(`.theme-${theme}`).classList.add('active');
    
    // Apply theme
    const root = document.documentElement;
    const body = document.body;
    
    // Remove existing theme classes
    body.classList.remove('theme-light', 'theme-sepia', 'theme-dark', 'theme-night');
    body.classList.add(`theme-${theme}`);
    
    switch(theme) {
        case 'light':
            root.style.setProperty('--bg-color', '#ffffff');
            root.style.setProperty('--text-color', '#2d3748');
            root.style.setProperty('--text-muted', '#718096');
            root.style.setProperty('--border-color', 'rgba(0,0,0,0.06)');
            break;
        case 'sepia':
            root.style.setProperty('--bg-color', '#f7f3e9');
            root.style.setProperty('--text-color', '#5d4e37');
            root.style.setProperty('--text-muted', '#8b7355');
            root.style.setProperty('--border-color', 'rgba(93,78,55,0.15)');
            break;
        case 'dark':
            root.style.setProperty('--bg-color', '#1a202c');
            root.style.setProperty('--text-color', '#f7fafc');
            root.style.setProperty('--text-muted', '#a0aec0');
            root.style.setProperty('--border-color', 'rgba(255,255,255,0.1)');
            break;
        case 'night':
            root.style.setProperty('--bg-color', '#0f0f0f');
            root.style.setProperty('--text-color', '#e2e8f0');
            root.style.setProperty('--text-muted', '#a0aec0');
            root.style.setProperty('--border-color', 'rgba(255,255,255,0.05)');
            break;
    }
    
    // Update main content and reading area
    document.querySelector('.main-content').style.background = `var(--bg-color)`;
    
    saveReadingStyle();
}

function setFontFamily(family) {
    currentReadingStyle.fontFamily = family;
    
    // Update font options
    document.querySelectorAll('.style-option[data-font]').forEach(option => {
        option.classList.remove('active');
    });
    document.querySelector(`[data-font="${family}"]`).classList.add('active');
    
    // Apply font
    const root = document.documentElement;
    let fontStack;
    
    switch(family) {
        case 'serif':
            fontStack = "'Georgia', 'Times New Roman', serif";
            break;
        case 'sans-serif':
            fontStack = "-apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Helvetica Neue', Arial, sans-serif";
            break;
        case 'lora':
            fontStack = "'Lora', Georgia, serif";
            break;
        case 'crimson':
            fontStack = "'Crimson Text', Georgia, serif";
            break;
    }
    
    root.style.setProperty('--reading-font', fontStack);
    saveReadingStyle();
}

function setFontSize(size) {
    currentReadingStyle.fontSize = parseInt(size);
    document.getElementById('fontSizeValue').textContent = size + 'px';
    document.documentElement.style.setProperty('--reading-font-size', size + 'px');
    saveReadingStyle();
}

function setLineSpacing(spacing) {
    currentReadingStyle.lineSpacing = parseFloat(spacing);
    document.getElementById('lineSpacingValue').textContent = spacing;
    document.documentElement.style.setProperty('--reading-line-height', spacing);
    saveReadingStyle();
}

function setReadingWidth(width) {
    currentReadingStyle.width = parseInt(width);
    document.getElementById('widthValue').textContent = width + 'px';
    document.documentElement.style.setProperty('--reading-width', width + 'px');
    saveReadingStyle();
}

function setReadingMargin(margin) {
    currentReadingStyle.margin = parseInt(margin);
    document.getElementById('marginValue').textContent = margin + 'px';
    document.documentElement.style.setProperty('--reading-margin', margin + 'px');
    saveReadingStyle();
}

function saveReadingStyle() {
    localStorage.setItem('epubReader_readingStyle', JSON.stringify(currentReadingStyle));
}

function loadReadingStyle() {
    const saved = localStorage.getItem('epubReader_readingStyle');
    if (saved) {
        currentReadingStyle = JSON.parse(saved);
        
        // Apply all saved settings
        setTheme(currentReadingStyle.theme);
        setFontFamily(currentReadingStyle.fontFamily);
        
        // Update sliders
        document.getElementById('fontSizeSlider').value = currentReadingStyle.fontSize;
        document.getElementById('lineSpacingSlider').value = currentReadingStyle.lineSpacing;
        document.getElementById('widthSlider').value = currentReadingStyle.width;
        document.getElementById('marginSlider').value = currentReadingStyle.margin;
        
        // Apply values
        setFontSize(currentReadingStyle.fontSize);
        setLineSpacing(currentReadingStyle.lineSpacing);
        setReadingWidth(currentReadingStyle.width);
        setReadingMargin(currentReadingStyle.margin);
    }
}

// Saved books
async function loadSavedBooks() {
    try {
        const books = await invoke('get_saved_books');
        const container = document.getElementById('savedBooks');
        
        if (books.length === 0) {
            container.innerHTML = '<div class="no-books">Nenhum livro salvo ainda</div>';
            return;
        }
        
        container.innerHTML = books.map((book, index) => {
            const isNew = index < 2; // Mark first 2 books as new
            const savedDate = new Date(book.saved_date);
            const isRecent = (Date.now() - savedDate.getTime()) < (7 * 24 * 60 * 60 * 1000); // Less than 7 days
            
            return `
                <div class="book-card" onclick="loadSavedBook('${book.id}')">
                    ${(isNew || isRecent) ? '<div class="new-badge">New</div>' : ''}
                    <div class="book-cover">
                        ${getBookCoverElement(book)}
                        <div style="position: absolute; bottom: 5px; right: 5px; font-size: 10px; opacity: 0.7;">
                            ${book.original_language.toUpperCase()}→${book.translated_language.toUpperCase()}
                        </div>
                    </div>
                    <div class="book-info">
                        <div class="book-title">${book.title}</div>
                        <div class="book-author">${book.author}</div>
                        <div class="book-meta">
                            <span>${formatDate(book.saved_date)}</span>
                        </div>
                    </div>
                </div>
            `;
        }).join('');
        
    } catch (error) {
        console.error('Failed to load saved books:', error);
    }
}

function getBookCoverElement(book) {
    // For now, use the first letter of the title as a placeholder
    // In a real implementation, you'd extract the cover from the ePub
    const firstLetter = book.title.charAt(0).toUpperCase();
    return `<div style="font-size: 28px; font-weight: bold;">${firstLetter}</div>`;
}

function formatDate(dateString) {
    const date = new Date(dateString);
    const now = new Date();
    const diffTime = Math.abs(now - date);
    const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24));
    
    if (diffDays === 1) return 'Hoje';
    if (diffDays === 2) return 'Ontem';
    if (diffDays < 7) return `${diffDays} dias atrás`;
    if (diffDays < 30) return `${Math.ceil(diffDays / 7)} sem atrás`;
    return date.toLocaleDateString('pt-BR');
}

async function loadSavedBook(bookId) {
    try {
        showLoading('Loading saved book...');
        
        const savedBook = await invoke('load_saved_book', { bookId });
        
        currentBook = savedBook.epub_info;
        translatedContent = savedBook.translated_content;
        currentChapter = 0;
        
        updateBookInfo();
        displayCurrentChapter();
        
    } catch (error) {
        showError('Failed to load saved book: ' + error);
    }
}

// Keyboard navigation
document.addEventListener('keydown', function(e) {
    if (!currentBook) return;
    
    switch(e.key) {
        case 'ArrowLeft':
            previousChapter();
            break;
        case 'ArrowRight':
            nextChapter();
            break;
        case 't':
        case 'T':
            if (e.ctrlKey || e.metaKey) {
                e.preventDefault();
                translateCurrent();
            }
            break;
    }
});