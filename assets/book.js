// Book Reader JavaScript

class BookReader {
    constructor() {
        this.currentChapter = window.currentChapter || 0;
        this.bookTitle = window.bookTitle || '';
        this.totalChapters = this.getTotalChapters();
        this.init();
    }
    
    init() {
        this.setupKeyboardNavigation();
        this.setupScrollProgress();
        this.loadReadingPosition();
    }
    
    setupKeyboardNavigation() {
        document.addEventListener('keydown', (e) => {
            switch(e.key) {
                case 'ArrowLeft':
                    this.previousChapter();
                    break;
                case 'ArrowRight':
                    this.nextChapter();
                    break;
                case 'Home':
                    this.goToIndex();
                    break;
                case 'Escape':
                    this.goToIndex();
                    break;
            }
        });
    }
    
    setupScrollProgress() {
        if (window.location.pathname.includes('chapter_')) {
            window.addEventListener('scroll', () => {
                this.updateProgress();
                this.saveReadingPosition();
            });
        }
    }
    
    updateProgress() {
        const scrollTop = window.pageYOffset;
        const docHeight = document.documentElement.scrollHeight - window.innerHeight;
        const scrollPercent = (scrollTop / docHeight) * 100;
        
        const progressFill = document.querySelector('.progress-fill');
        if (progressFill) {
            progressFill.style.width = Math.min(scrollPercent, 100) + '%';
        }
    }
    
    saveReadingPosition() {
        const position = {
            chapter: this.currentChapter,
            scroll: window.pageYOffset,
            timestamp: Date.now()
        };
        
        localStorage.setItem(`reading_position_${this.bookTitle}`, JSON.stringify(position));
    }
    
    loadReadingPosition() {
        const saved = localStorage.getItem(`reading_position_${this.bookTitle}`);
        if (saved) {
            const position = JSON.parse(saved);
            if (position.chapter === this.currentChapter) {
                setTimeout(() => {
                    window.scrollTo(0, position.scroll);
                }, 100);
            }
        }
    }
    
    getTotalChapters() {
        // This would be set by the backend when generating the HTML
        return window.totalChapters || 10;
    }
    
    startReading() {
        window.location.href = 'chapter_0.html';
    }
    
    previousChapter() {
        if (this.currentChapter > 0) {
            window.location.href = `chapter_${this.currentChapter - 1}.html`;
        } else {
            this.goToIndex();
        }
    }
    
    nextChapter() {
        if (this.currentChapter < this.totalChapters - 1) {
            window.location.href = `chapter_${this.currentChapter + 1}.html`;
        }
    }
    
    goToIndex() {
        window.location.href = 'index.html';
    }
    
    toggleFullscreen() {
        if (!document.fullscreenElement) {
            document.documentElement.requestFullscreen();
        } else {
            document.exitFullscreen();
        }
    }
    
    adjustFontSize(delta) {
        const content = document.querySelector('.chapter-content');
        if (content) {
            const currentSize = parseFloat(window.getComputedStyle(content).fontSize);
            const newSize = Math.max(12, Math.min(24, currentSize + delta));
            content.style.fontSize = newSize + 'px';
            
            localStorage.setItem('reader_font_size', newSize);
        }
    }
    
    loadFontSize() {
        const savedSize = localStorage.getItem('reader_font_size');
        if (savedSize) {
            const content = document.querySelector('.chapter-content');
            if (content) {
                content.style.fontSize = savedSize + 'px';
            }
        }
    }
    
    toggleTheme() {
        document.body.classList.toggle('dark-theme');
        const isDark = document.body.classList.contains('dark-theme');
        localStorage.setItem('reader_theme', isDark ? 'dark' : 'light');
    }
    
    loadTheme() {
        const savedTheme = localStorage.getItem('reader_theme');
        if (savedTheme === 'dark') {
            document.body.classList.add('dark-theme');
        }
    }
}

// Global functions for button clicks
function startReading() {
    bookReader.startReading();
}

function previousChapter() {
    bookReader.previousChapter();
}

function nextChapter() {
    bookReader.nextChapter();
}

function goToIndex() {
    bookReader.goToIndex();
}

// Initialize when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    window.bookReader = new BookReader();
    bookReader.loadFontSize();
    bookReader.loadTheme();
});

// Add reading controls
document.addEventListener('DOMContentLoaded', () => {
    if (window.location.pathname.includes('chapter_')) {
        addReadingControls();
    }
});

function addReadingControls() {
    const controls = document.createElement('div');
    controls.className = 'reading-controls';
    controls.innerHTML = `
        <div class="controls-panel">
            <button onclick="bookReader.adjustFontSize(-1)" title="Decrease font size">A-</button>
            <button onclick="bookReader.adjustFontSize(1)" title="Increase font size">A+</button>
            <button onclick="bookReader.toggleTheme()" title="Toggle theme">🌓</button>
            <button onclick="bookReader.toggleFullscreen()" title="Toggle fullscreen">⛶</button>
        </div>
    `;
    
    document.body.appendChild(controls);
    
    // Add CSS for controls
    const style = document.createElement('style');
    style.textContent = `
        .reading-controls {
            position: fixed;
            top: 20px;
            right: 20px;
            z-index: 1000;
        }
        
        .controls-panel {
            background: rgba(255, 255, 255, 0.9);
            border-radius: 8px;
            padding: 10px;
            box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
            display: flex;
            gap: 5px;
        }
        
        .controls-panel button {
            background: #3498db;
            color: white;
            border: none;
            padding: 8px 12px;
            border-radius: 4px;
            cursor: pointer;
            font-size: 14px;
        }
        
        .controls-panel button:hover {
            background: #2980b9;
        }
        
        @media (prefers-color-scheme: dark) {
            .controls-panel {
                background: rgba(45, 45, 45, 0.9);
            }
        }
        
        .dark-theme .controls-panel {
            background: rgba(45, 45, 45, 0.9);
        }
    `;
    
    document.head.appendChild(style);
}