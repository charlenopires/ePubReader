# Design Document

## Overview

This design simplifies the ebook reader interface by removing unnecessary UI elements and focusing on core functionality. The main changes include defaulting to grid view only, using dark theme exclusively, adding reading configuration options, and ensuring proper book display.

## Architecture

### UI Structure Changes

```
AppWindow
├── Header (Simplified)
│   ├── Logo/Title
│   ├── Search Bar
│   ├── Add Book Button
│   └── Settings Button (New)
├── Main Content
│   ├── Library View (Grid Only)
│   ├── Reading View (Enhanced)
│   └── Settings View (New)
└── Footer/Status Bar
```

### Removed Components

1. **View Mode Toggles**: Remove Grid/List/Large buttons
2. **Theme Selector**: Remove Light/Dark/Sepia buttons
3. **Unnecessary UI Complexity**: Simplify header layout

### New Components

1. **Reading Settings Panel**: Font size, font type, paragraph settings
2. **Enhanced Book Grid**: Better book display and error handling
3. **Settings View**: Centralized configuration

## Components and Interfaces

### ReadingSettings Component

```slint
export component ReadingSettings {
    // Font settings
    in-out property <int> font-size: 16;
    in-out property <string> font-family: "Georgia";
    in-out property <float> line-height: 1.6;
    in-out property <float> paragraph-spacing: 1.2;
    in-out property <float> margin-size: 20;
    
    // Callbacks
    callback font-size-changed(int);
    callback font-family-changed(string);
    callback line-height-changed(float);
    callback paragraph-spacing-changed(float);
    callback margin-size-changed(float);
}
```

### Enhanced BookGrid Component

```slint
export component EnhancedBookGrid {
    in-out property <[BookViewModel]> books;
    in-out property <bool> loading;
    
    // Error handling
    in-out property <string> error-message: "";
    in-out property <bool> show-error: false;
    
    // Callbacks
    callback book-selected(BookViewModel);
    callback retry-load();
}
```

### BookViewModel Updates

```slint
export struct BookViewModel {
    id: string,
    title: string,
    author: string,
    cover: image,
    progress: float,
    status: string,
    is-favorite: bool,
    rating: int,
    last-opened: string,
    added-date: string,
    // New fields
    has-cover: bool,
    load-error: bool,
    file-path: string,
}
```

## Data Models

### ReadingConfiguration

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingConfiguration {
    pub font_size: u32,
    pub font_family: String,
    pub line_height: f32,
    pub paragraph_spacing: f32,
    pub margin_size: f32,
    pub theme: String, // Always "dark"
}

impl Default for ReadingConfiguration {
    fn default() -> Self {
        Self {
            font_size: 16,
            font_family: "Georgia".to_string(),
            line_height: 1.6,
            paragraph_spacing: 1.2,
            margin_size: 20.0,
            theme: "dark".to_string(),
        }
    }
}
```

### BookDisplayInfo

```rust
#[derive(Debug, Clone)]
pub struct BookDisplayInfo {
    pub id: String,
    pub title: String,
    pub author: String,
    pub cover_path: Option<PathBuf>,
    pub has_cover: bool,
    pub load_error: bool,
    pub progress: f32,
    pub status: String,
    pub is_favorite: bool,
    pub rating: Option<u8>,
    pub last_opened: Option<DateTime<Utc>>,
    pub added_date: DateTime<Utc>,
    pub file_path: PathBuf,
}
```

## Error Handling

### Book Display Errors

1. **Missing Cover**: Show default placeholder with book icon
2. **Corrupted File**: Show error indicator with retry option
3. **Permission Issues**: Show warning with file path info
4. **Database Errors**: Show error message with refresh option

### Error Recovery

```rust
pub enum BookLoadError {
    FileNotFound(PathBuf),
    PermissionDenied(PathBuf),
    CorruptedFile(PathBuf),
    DatabaseError(String),
    CoverLoadError(PathBuf),
}

impl BookLoadError {
    pub fn user_message(&self) -> String {
        match self {
            Self::FileNotFound(path) => format!("Book file not found: {}", path.display()),
            Self::PermissionDenied(path) => format!("Cannot access book file: {}", path.display()),
            Self::CorruptedFile(path) => format!("Book file is corrupted: {}", path.display()),
            Self::DatabaseError(msg) => format!("Database error: {}", msg),
            Self::CoverLoadError(path) => format!("Cannot load book cover: {}", path.display()),
        }
    }
    
    pub fn can_retry(&self) -> bool {
        matches!(self, Self::DatabaseError(_) | Self::CoverLoadError(_))
    }
}
```

## UI Theme System

### Dark Theme Only

```slint
export global DarkTheme {
    // Colors
    out property <color> background: #1a1a1a;
    out property <color> surface: #2d2d2d;
    out property <color> surface-variant: #404040;
    out property <color> text-primary: #ffffff;
    out property <color> text-secondary: #b3b3b3;
    out property <color> text-tertiary: #808080;
    out property <color> accent: #4a9eff;
    out property <color> accent-hover: #6bb0ff;
    out property <color> border: #404040;
    out property <color> error: #ff6b6b;
    out property <color> warning: #ffd93d;
    out property <color> success: #6bcf7f;
    
    // Shadows
    out property <color> shadow-light: #00000020;
    out property <color> shadow-medium: #00000040;
    
    // Book grid specific
    out property <color> book-card-background: #2d2d2d;
    out property <color> book-card-hover: #404040;
    out property <color> book-placeholder: #404040;
    out property <color> book-placeholder-text: #808080;
}
```

## Reading Configuration System

### Configuration Storage

```rust
pub struct ConfigurationManager {
    config_path: PathBuf,
    current_config: ReadingConfiguration,
}

impl ConfigurationManager {
    pub async fn load_configuration() -> Result<ReadingConfiguration>;
    pub async fn save_configuration(&self, config: &ReadingConfiguration) -> Result<()>;
    pub fn get_font_options() -> Vec<String>;
    pub fn validate_font_size(size: u32) -> bool;
    pub fn validate_spacing(spacing: f32) -> bool;
}
```

### Font Management

```rust
pub struct FontManager;

impl FontManager {
    pub fn get_available_fonts() -> Vec<String> {
        vec![
            "Georgia".to_string(),
            "Times New Roman".to_string(),
            "Arial".to_string(),
            "Helvetica".to_string(),
            "Verdana".to_string(),
            "Calibri".to_string(),
            "Open Sans".to_string(),
            "Roboto".to_string(),
        ]
    }
    
    pub fn is_font_available(font_name: &str) -> bool;
    pub fn get_fallback_font() -> String;
}
```

## Book Loading and Display

### Enhanced Book Loading

```rust
impl BookService {
    pub async fn get_library_books_with_display_info(&self) -> Result<Vec<BookDisplayInfo>> {
        let books = self.database.get_all_books().await?;
        let mut display_books = Vec::new();
        
        for book in books {
            let display_info = self.create_display_info(book).await;
            display_books.push(display_info);
        }
        
        Ok(display_books)
    }
    
    async fn create_display_info(&self, book: Book) -> BookDisplayInfo {
        let has_cover = book.cover_path.as_ref()
            .map(|path| path.exists())
            .unwrap_or(false);
            
        let load_error = !book.file_path.exists();
        
        BookDisplayInfo {
            id: book.id,
            title: book.title,
            author: book.author,
            cover_path: book.cover_path,
            has_cover,
            load_error,
            progress: book.reading_progress,
            status: book.reading_status.to_string(),
            is_favorite: book.is_favorite,
            rating: book.rating,
            last_opened: book.last_opened,
            added_date: book.added_date,
            file_path: book.file_path,
        }
    }
}
```

## Testing Strategy

### UI Component Tests

1. **Grid Display**: Test book grid with various book states
2. **Error Handling**: Test error display and recovery
3. **Settings Panel**: Test configuration changes
4. **Theme Application**: Verify dark theme consistency

### Integration Tests

1. **Book Loading**: Test complete book loading flow
2. **Configuration Persistence**: Test settings save/load
3. **Error Recovery**: Test error scenarios and recovery
4. **Performance**: Test with large book libraries

## Migration Strategy

### UI File Updates

1. Remove view mode toggle components
2. Remove theme selector components
3. Add settings panel components
4. Update book grid for better error handling
5. Simplify header layout

### Configuration Migration

1. Create default reading configuration
2. Migrate existing theme settings to dark
3. Initialize font settings with defaults
4. Save configuration to user data directory

## Performance Considerations

1. **Book Grid Rendering**: Virtualize large book collections
2. **Cover Loading**: Lazy load book covers with placeholders
3. **Configuration Updates**: Debounce setting changes
4. **Error Recovery**: Implement retry with exponential backoff