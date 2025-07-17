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
    document.getElementById('bookAuthor').textContent = `by ${currentBook.author}`;
    document.getElementById('chapterInfo').textContent = `${currentChapter + 1}/${currentBook.chapters.length}`;
    
    const progress = ((currentChapter + 1) / currentBook.chapters.length) * 100;
    document.getElementById('progressFill').style.width = progress + '%';
    
    // Enable/disable navigation buttons
    document.getElementById('prevBtn').disabled = currentChapter === 0;
    document.getElementById('nextBtn').disabled = currentChapter === currentBook.chapters.length - 1;
    document.getElementById('translateBtn').disabled = false;
}

function displayCurrentChapter() {
    if (!currentBook || !currentBook.chapters[currentChapter]) {
        return;
    }
    
    const chapter = currentBook.chapters[currentChapter];
    const readingArea = document.getElementById('readingArea');
    
    // Use translated content if available, otherwise original
    const content = translatedContent[chapter.id] || chapter.content;
    
    readingArea.innerHTML = `
        <h2>${chapter.title}</h2>
        <div style="margin-top: 20px; white-space: pre-wrap;">${content}</div>
    `;
}

function showLoading(message) {
    document.getElementById('readingArea').innerHTML = `
        <div class="loading">${message}</div>
    `;
}

function showError(message) {
    const readingArea = document.getElementById('readingArea');
    readingArea.innerHTML = `
        <div class="error">${message}</div>
    `;
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

// Font size slider handler
document.getElementById('fontSize').addEventListener('input', function() {
    document.getElementById('fontSizeValue').textContent = this.value + 'px';
});

// Saved books
async function loadSavedBooks() {
    try {
        const books = await invoke('get_saved_books');
        const container = document.getElementById('savedBooks');
        
        if (books.length === 0) {
            container.innerHTML = '<p style="opacity: 0.6;">No saved books yet</p>';
            return;
        }
        
        container.innerHTML = books.map(book => `
            <div class="book-item" onclick="loadSavedBook('${book.id}')">
                <div class="book-title">${book.title}</div>
                <div class="book-author">${book.author}</div>
                <div style="font-size: 11px; opacity: 0.6;">
                    ${book.original_language} → ${book.translated_language}
                </div>
            </div>
        `).join('');
        
    } catch (error) {
        console.error('Failed to load saved books:', error);
    }
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