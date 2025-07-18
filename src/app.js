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