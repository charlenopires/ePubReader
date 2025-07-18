// Tauri v2 API imports - with proper error handling
let invoke, open;
let isTauriEnvironment = false;

// Try to import Tauri APIs using modern import syntax
async function loadTauriAPIs() {
    try {
        console.log('🔄 Attempting to load Tauri APIs via import...');
        
        // Try to dynamically import the Tauri APIs
        const { invoke: tauriInvoke } = await import('@tauri-apps/api/core');
        console.log('✅ Core API loaded');
        
        const { open: tauriOpen } = await import('@tauri-apps/plugin-dialog');
        console.log('✅ Dialog plugin loaded');
        
        invoke = tauriInvoke;
        open = tauriOpen;
        isTauriEnvironment = true;
        
        console.log('✅ Tauri v2 APIs loaded via import');
        console.log('Available APIs:', { 
            hasInvoke: typeof invoke === 'function', 
            hasOpen: typeof open === 'function' 
        });
        return true;
    } catch (error) {
        console.log('Import method failed:', error.message);
        
        // Try alternative import methods
        try {
            console.log('🔄 Trying alternative import from @tauri-apps/api...');
            const tauriApi = await import('@tauri-apps/api');
            if (tauriApi.invoke) {
                invoke = tauriApi.invoke;
                console.log('✅ Invoke loaded from main API');
            }
            
            // Try different dialog import paths
            try {
                const dialogApi = await import('@tauri-apps/api/dialog');
                if (dialogApi.open) {
                    open = dialogApi.open;
                    console.log('✅ Dialog loaded from main API');
                }
            } catch (dialogError) {
                console.log('Dialog import from main API failed:', dialogError.message);
            }
            
            if (invoke) {
                isTauriEnvironment = true;
                return true;
            }
        } catch (altError) {
            console.log('Alternative import also failed:', altError.message);
        }
        
        return false;
    }
}

async function initTauriAPIs() {
    try {
        // First try the modern import method
        console.log('Trying modern import method...');
        if (await loadTauriAPIs()) {
            return;
        }
        
        // Fallback to window object detection
        console.log('Falling back to window object detection...');
        let attempts = 0;
        const maxAttempts = 10;
        
        while (attempts < maxAttempts) {
            console.log(`Attempt ${attempts + 1}: Checking for Tauri v2 environment...`);
            
            // Check for Tauri v2 global object structure
            if (window.__TAURI__) {
                console.log('window.__TAURI__ found!');
                console.log('Available APIs:', Object.keys(window.__TAURI__));
                
                // Check for required v2 APIs - updated structure for v2
                if (window.__TAURI__.core && window.__TAURI__.core.invoke) {
                    invoke = window.__TAURI__.core.invoke;
                    
                    // Check for dialog plugin - try multiple possible locations
                    if (window.__TAURI__.dialog && window.__TAURI__.dialog.open) {
                        open = window.__TAURI__.dialog.open;
                        console.log('✅ Found dialog API at window.__TAURI__.dialog.open');
                    } else if (window.__TAURI_PLUGIN_DIALOG__ && window.__TAURI_PLUGIN_DIALOG__.open) {
                        open = window.__TAURI_PLUGIN_DIALOG__.open;
                        console.log('✅ Found dialog API at window.__TAURI_PLUGIN_DIALOG__.open');
                    } else if (window.__TAURI__.pluginDialog && window.__TAURI__.pluginDialog.open) {
                        open = window.__TAURI__.pluginDialog.open;
                        console.log('✅ Found dialog API at window.__TAURI__.pluginDialog.open');
                    } else {
                        console.log('⚠️  Dialog API not found, checking for invoke availability...');
                        // If we have invoke, we can try to use it directly for file operations
                        if (invoke) {
                            console.log('✅ Invoke available, will use backend file operations');
                            isTauriEnvironment = true;
                            open = null; // We'll handle this differently
                            return;
                        }
                    }
                    
                    if (invoke) {
                        isTauriEnvironment = true;
                        console.log('✅ Tauri v2 environment detected and initialized successfully');
                        return;
                    }
                } else {
                    console.log('Tauri v2 core APIs not fully loaded yet...');
                }
            } else {
                console.log('window.__TAURI__ not found yet...');
            }
            
            attempts++;
            await new Promise(resolve => setTimeout(resolve, 200));
        }
        
        // Final check - maybe the APIs are available under different names
        if (window.tauri || window.__TAURI_INVOKE__) {
            console.log('Alternative Tauri APIs found');
            if (window.tauri) {
                invoke = window.tauri.invoke;
                open = window.tauri.dialog?.open;
            }
            
            if (invoke) {
                isTauriEnvironment = true;
                console.log('✅ Alternative Tauri APIs initialized successfully');
                return;
            }
        }
        
        console.warn('⚠️ Tauri APIs not available after all attempts - running in browser mode');
        isTauriEnvironment = false;
        
    } catch (error) {
        console.error('❌ Failed to initialize Tauri APIs:', error);
        isTauriEnvironment = false;
    }
}

// Additional aggressive detection method
async function forceDetectTauriEnvironment() {
    console.log('🔍 Force detecting Tauri environment...');
    
    // Check for any Tauri indicators
    const checks = [
        typeof window !== 'undefined',
        window.__TAURI__ !== undefined,
        window.tauri !== undefined,
        window.__TAURI_INVOKE__ !== undefined,
        window.__TAURI_PLUGIN_DIALOG__ !== undefined,
        window.location?.protocol === 'tauri:',
        navigator.userAgent?.includes('Tauri'),
        typeof document !== 'undefined' && document.location?.protocol === 'tauri:'
    ];
    
    console.log('🧪 Environment checks:', {
        hasWindow: checks[0],
        hasMainTauri: checks[1],
        hasTauriObj: checks[2],
        hasInvokeGlobal: checks[3],
        hasDialogPlugin: checks[4],
        isTauriProtocol: checks[5],
        isTauriUserAgent: checks[6],
        isTauriDocProtocol: checks[7]
    });
    
    // If any check passes, force environment to true
    if (checks.some(check => check)) {
        console.log('🚀 Tauri environment detected via force check');
        isTauriEnvironment = true;
        
        // Try to get the APIs one more time
        if (window.__TAURI__?.core?.invoke) {
            invoke = window.__TAURI__.core.invoke;
        }
        if (window.__TAURI__?.dialog?.open) {
            open = window.__TAURI__.dialog.open;
        }
        
        return true;
    }
    
    return false;
}

let currentBook = null;
let currentChapter = 0;
let translatedContent = {};
let settings = {};

// Initialize app
document.addEventListener('DOMContentLoaded', async () => {
    await initTauriAPIs();
    await loadSettings();
    await loadSavedBooks();
    loadReadingStyle();
    updateFontSize();
    updateWelcomeMessage();
    addBookCardEnhancements();
});

function updateWelcomeMessage() {
    const welcomeElement = document.getElementById('welcomeMessage');
    if (welcomeElement) {
        if (isTauriEnvironment) {
            welcomeElement.textContent = 'Select an ePub file from the sidebar to start reading with automatic translation capabilities.';
        } else {
            welcomeElement.innerHTML = `
                This is a preview of ePubReader running in your browser.<br>
                For full functionality including file opening and translation, please use the desktop application:<br>
                <code style="background: rgba(0,0,0,0.1); padding: 4px 8px; border-radius: 4px; margin-top: 8px; display: inline-block;">
                ./src-tauri/target/release/epubreader
                </code>
            `;
        }
    }
}

// File operations
async function openFile() {
    try {
        // Force re-initialization of Tauri APIs
        console.log('🔄 Forcing Tauri API re-initialization...');
        await initTauriAPIs();
        
        // If still not detected, try aggressive detection
        if (!isTauriEnvironment) {
            console.log('🔄 Trying aggressive Tauri detection...');
            await forceDetectTauriEnvironment();
        }
        
        // Final check with detailed logging
        const hasInvoke = typeof invoke === 'function';
        const hasWindow = typeof window !== 'undefined';
        const hasTauriGlobal = hasWindow && (window.__TAURI__ || window.tauri);
        
        console.log('🔍 Final environment check:', {
            hasInvoke,
            hasWindow,
            hasTauriGlobal,
            isTauriEnvironment,
            windowProtocol: window?.location?.protocol,
            userAgent: navigator?.userAgent?.includes('Tauri')
        });
        
        // Force environment detection based on any available indicator
        if (!isTauriEnvironment && (hasInvoke || hasTauriGlobal || window?.location?.protocol === 'tauri:')) {
            console.log('🚀 Forcing Tauri environment to true based on available APIs');
            isTauriEnvironment = true;
        }
        
        // Last resort: If we're not in a typical web browser environment, assume Tauri
        if (!isTauriEnvironment && typeof window !== 'undefined' && !window.location.href.startsWith('http')) {
            console.log('🔧 Non-HTTP environment detected, assuming Tauri');
            isTauriEnvironment = true;
        }
        
        if (!isTauriEnvironment && !hasInvoke) {
            showError('File opening is only available in the desktop application. Please execute: ./src-tauri/target/release/epubreader');
            return;
        }
        
        let selected;
        
        if (open) {
            // Use the dialog plugin API if available
            console.log('Using dialog plugin API...');
            selected = await open({
                multiple: false,
                filters: [{
                    name: 'ePub',
                    extensions: ['epub']
                }]
            });
        } else if (invoke) {
            // Fallback to a custom backend command for file selection
            console.log('Dialog plugin not available, using invoke for file selection...');
            try {
                selected = await invoke('open_file_dialog');
            } catch (invokeError) {
                console.error('Backend file dialog failed:', invokeError);
                showError('Unable to open file dialog. Please check the application setup.');
                return;
            }
        } else {
            throw new Error('No file dialog method available');
        }

        if (selected) {
            await loadEpub(selected);
        }
    } catch (error) {
        console.error('Open file error:', error);
        
        // Provide more specific error message
        if (error.message.includes('tauri') || error.message.includes('invoke')) {
            showError('Desktop application not detected. Please run: ./src-tauri/target/release/epubreader');
        } else {
            showError('Failed to open file: ' + error.message);
        }
    }
}

async function loadEpub(path) {
    try {
        if (!invoke) {
            throw new Error('Tauri invoke API not initialized');
        }
        
        showLoading('Loading ePub file...');
        
        currentBook = await invoke('open_epub', { path });
        currentChapter = 0;
        translatedContent = {};
        
        updateBookInfo();
        displayCurrentChapter();
        
        // Automatically translate if book is in English and we have API key
        if (currentBook.language === 'en' && settings.google_api_key) {
            showLoading('Translating book from English to Portuguese...');
            await translateBookToBrazilianPortuguese();
        }
        
    } catch (error) {
        console.error('Load ePub error:', error);
        showError('Failed to load ePub: ' + error.message);
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
    
    if (!invoke) {
        showError('Translation service not available');
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
        console.error('Translation error:', error);
        showError('Translation failed: ' + error.message);
    }
}

async function translateBook() {
    if (!currentBook || !settings.google_api_key || !invoke) {
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
        console.error('Book translation error:', error);
        showError('Translation failed: ' + error.message);
    }
}

async function translateBookToBrazilianPortuguese() {
    if (!currentBook || !settings.google_api_key || !invoke) {
        showError('Translation service not available');
        return;
    }
    
    if (currentBook.chapters.length === 0) {
        showError('No chapters found to translate');
        return;
    }
    
    try {
        showLoading('Starting automatic translation to Portuguese...');
        
        // Translate all chapters
        for (let i = 0; i < currentBook.chapters.length; i++) {
            const chapter = currentBook.chapters[i];
            
            showLoading(`Translating chapter ${i + 1} of ${currentBook.chapters.length}...`);
            
            if (chapter.content && chapter.content.trim().length > 0) {
                const translated = await invoke('translate_text', {
                    text: chapter.content,
                    targetLang: 'pt-BR',
                    apiKey: settings.google_api_key
                });
                
                translatedContent[chapter.id] = translated;
            }
            
            // Update progress display
            const progress = Math.round(((i + 1) / currentBook.chapters.length) * 100);
            showLoading(`Translation progress: ${progress}% (${i + 1}/${currentBook.chapters.length} chapters)`);
        }
        
        showLoading('Saving translated book with image preservation...');
        
        // Use the new enhanced translation function that preserves images
        const bookId = await invoke('translate_epub_with_images', {
            epubInfo: currentBook,
            targetLang: 'pt-BR',
            apiKey: settings.google_api_key
        });
        
        showLoading('Loading translated content...');
        
        // Reload saved books list
        await loadSavedBooks();
        
        // Display the translated content
        displayCurrentChapter();
        
        // Show success message briefly then display content
        showLoading('Translation completed successfully!');
        setTimeout(() => {
            displayCurrentChapter();
        }, 2000);
        
        console.log('Book translated and saved successfully with ID:', bookId);
        
    } catch (error) {
        console.error('Automatic translation error:', error);
        showError('Automatic translation failed: ' + error.message);
    }
}

// Settings
async function loadSettings() {
    try {
        if (isTauriEnvironment && invoke) {
            settings = await invoke('get_settings');
            
            document.getElementById('targetLang').value = settings.target_language;
            document.getElementById('apiKey').value = settings.google_api_key;
            document.getElementById('autoTranslate').checked = settings.auto_translate;
        } else {
            // Load from localStorage in browser mode
            const savedSettings = localStorage.getItem('epubReader_settings');
            if (savedSettings) {
                settings = JSON.parse(savedSettings);
            } else {
                throw new Error('Settings service not available');
            }
        }
        
    } catch (error) {
        console.error('Failed to load settings:', error);
        settings = {
            target_language: 'en',
            google_api_key: '',
            auto_translate: true,
            font_size: 16,
            theme: 'light'
        };
        
        // Update UI with default settings
        if (document.getElementById('targetLang')) {
            document.getElementById('targetLang').value = settings.target_language;
            document.getElementById('apiKey').value = settings.google_api_key;
            document.getElementById('autoTranslate').checked = settings.auto_translate;
        }
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
        settings.auto_translate = document.getElementById('autoTranslate').checked;
        
        if (isTauriEnvironment && invoke) {
            await invoke('save_settings', { settings });
        } else {
            // Save to localStorage in browser mode
            localStorage.setItem('epubReader_settings', JSON.stringify(settings));
        }
        
        hideSettings();
        
    } catch (error) {
        console.error('Save settings error:', error);
        showError('Failed to save settings: ' + error.message);
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
        const container = document.getElementById('savedBooks');
        
        if (!isTauriEnvironment || !invoke) {
            container.innerHTML = '<div class="no-books">Saved books available only in desktop app</div>';
            return;
        }
        
        const books = await invoke('get_saved_books');
        
        if (books.length === 0) {
            container.innerHTML = '<div class="no-books">Nenhum livro salvo ainda</div>';
            return;
        }
        
        container.innerHTML = books.map((book, index) => {
            const isNew = index < 2; // Mark first 2 books as new
            const savedDate = new Date(book.saved_date);
            const isRecent = (Date.now() - savedDate.getTime()) < (7 * 24 * 60 * 60 * 1000); // Less than 7 days
            
            return `
                <div class="book-card" 
                     data-book-id="${book.id}"
                     data-book-title="${book.title.replace(/"/g, '&quot;')}"
                     title="Clique esquerdo: abrir livro | Clique direito: opções">
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
        
        // Add event listeners after creating the HTML
        setTimeout(() => setupBookCardListeners(), 100);
        
        // Add "Delete All" button if there are books
        if (books.length > 0) {
            container.innerHTML += `
                <div class="delete-all-container">
                    <button class="delete-all-btn" onclick="confirmDeleteAllBooks()">
                        🗑️ Deletar Todos os Livros (${books.length})
                    </button>
                </div>
            `;
        }
        
    } catch (error) {
        console.error('Failed to load saved books:', error);
    }
}

// Delete book functions
async function confirmDeleteBook(bookId, bookTitle) {
    if (!isTauriEnvironment || !invoke) {
        showError('Delete function not available in browser mode');
        return;
    }
    
    const confirmed = confirm(`Tem certeza que deseja deletar o livro "${bookTitle}"?\n\nEsta ação não pode ser desfeita.`);
    
    if (confirmed) {
        await deleteBook(bookId, bookTitle);
    }
}

async function deleteBook(bookId, bookTitle) {
    try {
        showLoading(`Deletando "${bookTitle}"...`);
        
        await invoke('delete_saved_book', { bookId });
        
        // Reload the books list
        await loadSavedBooks();
        
        showSuccess(`Livro "${bookTitle}" deletado com sucesso!`);
        
        // Clear the reading area if this book was being read
        if (currentBook && currentBook.title === bookTitle) {
            currentBook = null;
            translatedContent = {};
            currentChapter = 0;
            document.getElementById('readingArea').innerHTML = `
                <div class="reading-content">
                    <h2>Livro deletado</h2>
                    <p>O livro que você estava lendo foi deletado.</p>
                </div>
            `;
        }
        
    } catch (error) {
        console.error('Failed to delete book:', error);
        showError(`Erro ao deletar o livro: ${error.message || error}`);
    }
}

async function confirmDeleteAllBooks() {
    if (!isTauriEnvironment || !invoke) {
        showError('Delete function not available in browser mode');
        return;
    }
    
    const confirmed = confirm('Tem certeza que deseja deletar TODOS os livros salvos?\n\nEsta ação não pode ser desfeita e removerá todos os ePubs traduzidos permanentemente.');
    
    if (confirmed) {
        const doubleConfirmed = confirm('ATENÇÃO: Isso deletará TODOS os seus livros traduzidos!\n\nDigite "DELETAR" para confirmar ou cancele para voltar.');
        
        if (doubleConfirmed) {
            await deleteAllBooks();
        }
    }
}

async function deleteAllBooks() {
    try {
        showLoading('Deletando todos os livros...');
        
        await invoke('delete_all_saved_books');
        
        // Reload the books list (should show empty now)
        await loadSavedBooks();
        
        showSuccess('Todos os livros foram deletados com sucesso!');
        
        // Clear current reading state
        currentBook = null;
        translatedContent = {};
        currentChapter = 0;
        document.getElementById('readingArea').innerHTML = `
            <div class="reading-content">
                <h2>Biblioteca limpa</h2>
                <p>Todos os livros traduzidos foram removidos.</p>
                <p>Abra um novo ePub para começar a ler.</p>
            </div>
        `;
        
    } catch (error) {
        console.error('Failed to delete all books:', error);
        showError(`Erro ao deletar todos os livros: ${error.message || error}`);
    }
}

function showSuccess(message) {
    const readingArea = document.getElementById('readingArea');
    readingArea.innerHTML = `
        <div class="reading-content">
            <div class="success" style="color: green; font-weight: bold; margin-bottom: 20px;">
                ✅ ${message}
            </div>
        </div>
    `;
    
    // Hide success message after 3 seconds
    setTimeout(() => {
        if (readingArea.innerHTML.includes('✅')) {
            readingArea.innerHTML = `
                <div class="reading-content">
                    <h2>Selecione um livro</h2>
                    <p>Escolha um livro da biblioteca ou abra um novo ePub para começar a ler.</p>
                </div>
            `;
        }
    }, 3000);
}

// Context menu functionality
let currentContextMenu = null;

function showContextMenu(event, bookId, encodedBookTitle) {
    event.preventDefault(); // Prevent default right-click menu
    event.stopPropagation();
    
    const bookTitle = decodeURIComponent(encodedBookTitle);
    
    // Remove any existing context menu
    hideContextMenu();
    
    // Create context menu
    const contextMenu = document.createElement('div');
    contextMenu.className = 'context-menu';
    
    // Store the book data on the menu element
    contextMenu.setAttribute('data-book-id', bookId);
    contextMenu.setAttribute('data-book-title', bookTitle);
    
    contextMenu.innerHTML = `
        <div class="context-menu-item" data-action="open">
            📖 Abrir Livro
        </div>
        <div class="context-menu-separator"></div>
        <div class="context-menu-item delete-item" data-action="delete">
            🗑️ Deletar Livro
        </div>
        <div class="context-menu-separator"></div>
        <div class="context-menu-item" data-action="info">
            ℹ️ Informações
        </div>
    `;
    
    // Add click listeners to menu items
    contextMenu.querySelectorAll('.context-menu-item[data-action]').forEach(item => {
        item.addEventListener('click', function(e) {
            e.stopPropagation();
            const action = this.getAttribute('data-action');
            handleContextMenuAction(action, bookId, bookTitle);
        });
    });
    
    // Position the menu
    document.body.appendChild(contextMenu);
    currentContextMenu = contextMenu;
    
    // Calculate position
    const x = event.clientX;
    const y = event.clientY;
    const menuRect = contextMenu.getBoundingClientRect();
    const windowWidth = window.innerWidth;
    const windowHeight = window.innerHeight;
    
    // Adjust position to keep menu on screen
    let menuX = x;
    let menuY = y;
    
    if (x + menuRect.width > windowWidth) {
        menuX = windowWidth - menuRect.width - 5;
    }
    
    if (y + menuRect.height > windowHeight) {
        menuY = windowHeight - menuRect.height - 5;
    }
    
    contextMenu.style.left = `${menuX}px`;
    contextMenu.style.top = `${menuY}px`;
    
    // Add event listener to hide menu when clicking elsewhere
    setTimeout(() => {
        document.addEventListener('click', hideContextMenu);
        document.addEventListener('contextmenu', hideContextMenu);
        document.addEventListener('keydown', handleContextMenuKeydown);
    }, 0);
    
    return false; // Prevent default context menu
}

function hideContextMenu() {
    if (currentContextMenu) {
        currentContextMenu.remove();
        currentContextMenu = null;
    }
    
    // Remove event listeners
    document.removeEventListener('click', hideContextMenu);
    document.removeEventListener('contextmenu', hideContextMenu);
    document.removeEventListener('keydown', handleContextMenuKeydown);
}

function handleContextMenuAction(action, bookId, bookTitle) {
    hideContextMenu();
    
    console.log('Context menu action:', action, 'for book:', bookTitle);
    
    switch(action) {
        case 'open':
            loadSavedBook(bookId);
            break;
        case 'delete':
            confirmDeleteBook(bookId, bookTitle);
            break;
        case 'info':
            showBookInfo(bookId, bookTitle);
            break;
        default:
            console.log('Unknown action:', action);
    }
}

function showBookInfo(bookId, bookTitle) {
    if (!isTauriEnvironment || !invoke) {
        showError('Book info not available in browser mode');
        return;
    }
    
    // Find book in the current loaded books
    invoke('get_saved_books').then(books => {
        const book = books.find(b => b.id === bookId);
        if (book) {
            const readingArea = document.getElementById('readingArea');
            readingArea.innerHTML = `
                <div class="reading-content">
                    <h2>📚 Informações do Livro</h2>
                    <div style="background: #f8f9fa; padding: 20px; border-radius: 8px; margin: 20px 0;">
                        <p><strong>Título:</strong> ${book.title}</p>
                        <p><strong>Autor:</strong> ${book.author}</p>
                        <p><strong>Idioma Original:</strong> ${book.original_language.toUpperCase()}</p>
                        <p><strong>Idioma Traduzido:</strong> ${book.translated_language.toUpperCase()}</p>
                        <p><strong>Data de Tradução:</strong> ${formatDate(book.saved_date)}</p>
                        <p><strong>ID do Livro:</strong> ${book.id}</p>
                        <p><strong>Localização:</strong> ${book.file_path}</p>
                    </div>
                    <button onclick="loadSavedBook('${bookId}')" style="
                        background: #667eea; 
                        color: white; 
                        border: none; 
                        padding: 10px 20px; 
                        border-radius: 4px; 
                        cursor: pointer;
                        margin-right: 10px;
                    ">📖 Abrir Livro</button>
                    <button onclick="document.getElementById('readingArea').innerHTML = '<div class=\\'reading-content\\'><h2>Selecione um livro</h2><p>Escolha um livro da biblioteca ou abra um novo ePub para começar a ler.</p></div>'" style="
                        background: #6c757d; 
                        color: white; 
                        border: none; 
                        padding: 10px 20px; 
                        border-radius: 4px; 
                        cursor: pointer;
                    ">❌ Fechar</button>
                </div>
            `;
        }
    }).catch(error => {
        console.error('Failed to get book info:', error);
        showError('Erro ao carregar informações do livro');
    });
}

function handleContextMenuKeydown(event) {
    if (event.key === 'Escape') {
        hideContextMenu();
    }
}

// Enhanced book card hover effects
function addBookCardEnhancements() {
    document.addEventListener('mouseover', function(event) {
        if (event.target.closest('.book-card')) {
            const card = event.target.closest('.book-card');
            card.style.transform = 'translateY(-2px)';
        }
    });
    
    document.addEventListener('mouseout', function(event) {
        if (event.target.closest('.book-card')) {
            const card = event.target.closest('.book-card');
            card.style.transform = 'translateY(0)';
        }
    });
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
        if (!invoke) {
            throw new Error('Book loading service not available');
        }
        
        showLoading('Loading saved book...');
        
        const savedBook = await invoke('load_saved_book', { bookId });
        
        currentBook = savedBook.epub_info;
        translatedContent = savedBook.translated_content;
        currentChapter = 0;
        
        updateBookInfo();
        displayCurrentChapter();
        
    } catch (error) {
        console.error('Load saved book error:', error);
        showError('Failed to load saved book: ' + error.message);
    }
}

// Book card event listeners setup
function setupBookCardListeners() {
    console.log('Setting up book card listeners...');
    
    const bookCards = document.querySelectorAll('.book-card[data-book-id]');
    console.log('Found', bookCards.length, 'book cards');
    
    bookCards.forEach(card => {
        const bookId = card.getAttribute('data-book-id');
        const bookTitle = card.getAttribute('data-book-title');
        
        if (!bookId || !bookTitle) {
            console.warn('Book card missing required attributes:', card);
            return;
        }
        
        console.log('Setting up listeners for book:', bookTitle, 'ID:', bookId);
        
        // Remove any existing listeners to prevent duplicates
        card.removeEventListener('click', card._clickHandler);
        card.removeEventListener('contextmenu', card._contextHandler);
        
        // Left click to open book
        card._clickHandler = function(e) {
            e.preventDefault();
            e.stopPropagation();
            console.log('Left click: Opening book:', bookTitle);
            loadSavedBook(bookId);
        };
        
        // Right click for context menu
        card._contextHandler = function(e) {
            e.preventDefault();
            e.stopPropagation();
            console.log('Right click: Showing context menu for book:', bookTitle);
            showContextMenu(e, bookId, encodeURIComponent(bookTitle));
        };
        
        card.addEventListener('click', card._clickHandler);
        card.addEventListener('contextmenu', card._contextHandler);
    });
    
    console.log('Book card listeners setup complete');
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