// ePub Reader Library - Main Application

const { invoke } = window.__TAURI__.core;
const { open } = window.__TAURI__.dialog;
const { shell } = window.__TAURI__;

class EpubReaderApp {
    constructor() {
        this.books = [];
        this.languages = [];
        this.selectedLanguage = '';
        this.ollamaStatus = null;
        this.init();
    }

    async init() {
        console.log('Initializing ePub Reader Library...');
        
        // Show loading
        this.showLoading();
        
        try {
            // Load supported languages
            await this.loadLanguages();
            
            // Check Ollama status
            await this.checkOllamaStatus();
            
            // Load books
            await this.loadBooks();
            
            // Setup event listeners
            this.setupEventListeners();
            
            console.log('Application initialized successfully');
        } catch (error) {
            console.error('Failed to initialize application:', error);
            this.showError('Failed to initialize application: ' + error);
        }
    }

    showLoading() {
        document.getElementById('loading').classList.remove('hidden');
        document.getElementById('empty-state').classList.add('hidden');
        document.getElementById('books-grid').classList.add('hidden');
    }

    hideLoading() {
        document.getElementById('loading').classList.add('hidden');
    }

    async loadLanguages() {
        try {
            this.languages = await invoke('get_supported_languages');
            this.populateLanguageSelector();
        } catch (error) {
            console.error('Failed to load languages:', error);
        }
    }

    populateLanguageSelector() {
        const select = document.getElementById('target-language');
        select.innerHTML = '<option value="">Select Language</option>';
        
        this.languages.forEach(lang => {
            const option = document.createElement('option');
            option.value = lang.code;
            option.textContent = `${lang.native_name} (${lang.name})`;
            select.appendChild(option);
        });
    }

    async checkOllamaStatus() {
        try {
            this.ollamaStatus = await invoke('check_ollama_status');
            
            if (!this.ollamaStatus.is_running) {
                this.showOllamaModal();
            }
        } catch (error) {
            console.error('Failed to check Ollama status:', error);
            this.showOllamaModal();
        }
    }

    showOllamaModal() {
        document.getElementById('ollama-modal').classList.remove('hidden');
    }

    hideOllamaModal() {
        document.getElementById('ollama-modal').classList.add('hidden');
    }

    async loadBooks() {
        try {
            this.books = await invoke('get_books');
            this.renderBooks();
        } catch (error) {
            console.error('Failed to load books:', error);
            this.showError('Failed to load books: ' + error);
        }
    }

    renderBooks() {
        this.hideLoading();
        
        const booksGrid = document.getElementById('books-grid');
        const emptyState = document.getElementById('empty-state');
        
        if (this.books.length === 0) {
            emptyState.classList.remove('hidden');
            booksGrid.classList.add('hidden');
            return;
        }
        
        emptyState.classList.add('hidden');
        booksGrid.classList.remove('hidden');
        
        booksGrid.innerHTML = '';
        
        this.books.forEach(book => {
            const bookCard = this.createBookCard(book);
            booksGrid.appendChild(bookCard);
        });
    }

    createBookCard(book) {
        const card = document.createElement('div');
        card.className = 'book-card';
        card.dataset.bookId = book.id;
        
        const statusClass = `status-${book.translation_status.toLowerCase().replace('_', '-')}`;
        const statusText = book.translation_status.replace('_', ' ');
        
        card.innerHTML = `
            <div class="book-cover">
                ${book.cover_path ? 
                    `<img src="${book.cover_path}" alt="${book.title} cover">` : 
                    '<div class="book-cover-placeholder">📖</div>'
                }
                <div class="translation-status ${statusClass}">${statusText}</div>
            </div>
            <div class="book-info">
                <div class="book-title">${book.title}</div>
                <div class="book-author">${book.author}</div>
                <div class="book-language">${book.language}</div>
            </div>
            <div class="book-actions">
                <button class="action-btn" onclick="app.openBook('${book.id}')">Read</button>
                ${this.ollamaStatus?.is_running ? 
                    `<button class="action-btn secondary" onclick="app.translateBook('${book.id}')">Translate</button>` : 
                    ''
                }
            </div>
        `;
        
        return card;
    }

    setupEventListeners() {
        // Add book button
        document.getElementById('add-book-btn').addEventListener('click', () => {
            this.addBook();
        });
        
        // Add first book button (empty state)
        document.getElementById('add-first-book-btn').addEventListener('click', () => {
            this.addBook();
        });
        
        // Language selector
        document.getElementById('target-language').addEventListener('change', (e) => {
            this.selectedLanguage = e.target.value;
        });
        
        // Ollama modal buttons
        document.getElementById('check-ollama-btn').addEventListener('click', () => {
            this.checkOllamaStatus();
        });
        
        document.getElementById('continue-without-ollama-btn').addEventListener('click', () => {
            this.hideOllamaModal();
        });
        
        // Translation modal
        document.getElementById('cancel-translation-btn').addEventListener('click', () => {
            this.hideTranslationModal();
        });
    }

    async addBook() {
        try {
            const selected = await open({
                multiple: false,
                filters: [{
                    name: 'EPUB Files',
                    extensions: ['epub']
                }]
            });
            
            if (selected) {
                console.log('Selected file:', selected);
                this.showLoading();
                
                const book = await invoke('add_book', { filePath: selected });
                console.log('Book added:', book);
                
                // Reload books
                await this.loadBooks();
            }
        } catch (error) {
            console.error('Failed to add book:', error);
            this.showError('Failed to add book: ' + error);
            this.hideLoading();
        }
    }

    async openBook(bookId) {
        try {
            console.log('Opening book:', bookId);
            
            // Generate HTML for the book
            const htmlPath = await invoke('generate_book_html', { bookId });
            console.log('Generated HTML at:', htmlPath);
            
            // Open the book in default browser
            await shell.open(htmlPath);
        } catch (error) {
            console.error('Failed to open book:', error);
            this.showError('Failed to open book: ' + error);
        }
    }

    async translateBook(bookId) {
        if (!this.selectedLanguage) {
            this.showError('Please select a target language first');
            return;
        }
        
        if (!this.ollamaStatus?.is_running) {
            this.showError('Ollama is not running. Please start Ollama first.');
            return;
        }
        
        try {
            console.log('Translating book:', bookId, 'to', this.selectedLanguage);
            
            this.showTranslationModal();
            this.updateTranslationProgress(0, 'Starting translation...');
            
            // Start translation
            await invoke('translate_book', { 
                bookId, 
                targetLanguage: this.selectedLanguage 
            });
            
            this.updateTranslationProgress(100, 'Translation completed!');
            
            setTimeout(() => {
                this.hideTranslationModal();
                this.loadBooks(); // Refresh books to show updated status
            }, 2000);
            
        } catch (error) {
            console.error('Failed to translate book:', error);
            this.showError('Failed to translate book: ' + error);
            this.hideTranslationModal();
        }
    }

    showTranslationModal() {
        document.getElementById('translation-modal').classList.remove('hidden');
    }

    hideTranslationModal() {
        document.getElementById('translation-modal').classList.add('hidden');
    }

    updateTranslationProgress(percent, status) {
        document.getElementById('translation-progress').style.width = percent + '%';
        document.getElementById('translation-status').textContent = status;
    }

    showError(message) {
        // Simple error display - in a real app you'd want a proper notification system
        alert(message);
    }
}

// Initialize the application when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    window.app = new EpubReaderApp();
});

// Global functions for onclick handlers
window.openBook = (bookId) => window.app.openBook(bookId);
window.translateBook = (bookId) => window.app.translateBook(bookId);