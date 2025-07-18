use std::sync::Arc;
use anyhow::Result;
use slint::{ModelRc, VecModel, SharedString};
use tokio::runtime::Runtime;

mod models;
mod services;
mod utils;

use models::*;
use services::*;
use utils::image_cache::ImageCache;

slint::include_modules!();

/// Main application structure
struct EbookReaderApp {
    rt: Runtime,
    book_service: Arc<BookService>,
    database: Arc<DatabaseService>,
    image_cache: Arc<ImageCache>,
    ui: AppWindow,
}

impl EbookReaderApp {
    /// Create a new application instance
    pub fn new() -> Result<Self> {
        let rt = Runtime::new()?;
        
        // Initialize services with improved error handling
        let database = Arc::new(rt.block_on(async {
            match DatabaseService::new().await {
                Ok(db) => {
                    println!("✅ Database initialized successfully");
                    Ok(db)
                }
                Err(e) => {
                    eprintln!("❌ Database initialization failed: {}", e);
                    eprintln!("💡 This might be due to:");
                    eprintln!("   - Insufficient permissions in the app data directory");
                    eprintln!("   - Corrupted database file");
                    eprintln!("   - Another instance of the app running");
                    eprintln!("\n🔧 Trying fallback initialization...");
                    
                    // Try with a temporary database as last resort
                    match PathResolver::get_temp_directory() {
                        Ok(temp_dir) => {
                            let temp_db_path = temp_dir.join("library.db");
                            eprintln!("📁 Using temporary database: {}", temp_db_path.display());
                            DatabaseService::new_with_path(Some(temp_db_path)).await
                        }
                        Err(temp_err) => {
                            eprintln!("❌ Fallback also failed: {}", temp_err);
                            Err(e)
                        }
                    }
                }
            }
        })?);
        
        let cache_dir = PathResolver::get_cache_directory()
            .unwrap_or_else(|_| std::env::temp_dir().join("ebook-reader-cache"));
        let image_cache = Arc::new(ImageCache::new(cache_dir)?);
        let book_service = Arc::new(BookService::new(database.clone(), image_cache.clone()));
        
        // Create UI
        let ui = AppWindow::new()?;
        
        Ok(Self {
            rt,
            book_service,
            database,
            image_cache,
            ui,
        })
    }

    /// Initialize the application
    pub fn initialize(&self) -> Result<()> {
        // Set up UI callbacks
        self.setup_callbacks()?;
        
        // Load initial data
        self.load_library()?;
        
        Ok(())
    }

    /// Set up UI callbacks
    fn setup_callbacks(&self) -> Result<()> {
        let book_service = self.book_service.clone();
        let rt_handle = self.rt.handle().clone();
        
        // Handle book selection
        let ui_weak = self.ui.as_weak();
        let book_service_clone = book_service.clone();
        let rt_handle_clone = rt_handle.clone();
        self.ui.on_book_selected(move |book_view_model| {
            let book_service = book_service_clone.clone();
            let ui = ui_weak.clone();
            
            rt_handle_clone.spawn(async move {
                if let Ok(book) = book_service.get_book_by_id(&book_view_model.id).await {
                    let book_title = book.title.clone();
                    let book_author = book.author.clone();
                    let book_progress = book.reading_progress;
                    
                    // Switch to reading view using invoke_from_event_loop
                    slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui.upgrade() {
                            ui.set_current_view(SharedString::from("reading"));
                            ui.set_current_book_title(SharedString::from(book_title));
                            ui.set_current_book_author(SharedString::from(book_author));
                            ui.set_current_book_progress(book_progress);
                        }
                    }).unwrap();
                }
            });
        });

        // Handle file opening
        let ui_weak = self.ui.as_weak();
        let book_service_clone = book_service.clone();
        let rt_handle_clone = rt_handle.clone();
        self.ui.on_open_file(move || {
            let book_service = book_service_clone.clone();
            let ui = ui_weak.clone();
            
            rt_handle_clone.spawn(async move {
                if let Some(file_path) = rfd::AsyncFileDialog::new()
                    .add_filter("eBooks", &["epub", "pdf", "mobi"])
                    .pick_file()
                    .await
                {
                    slint::invoke_from_event_loop({
                        let ui = ui.clone();
                        move || {
                            if let Some(ui) = ui.upgrade() {
                                ui.set_loading(true);
                            }
                        }
                    }).unwrap();
                    
                    match book_service.add_book(&file_path.path()).await {
                        Ok(book_id) => {
                            // Refresh library
                            if let Ok(books) = book_service.get_library_books().await {
                                slint::invoke_from_event_loop({
                                    let ui = ui.clone();
                                    move || {
                                        if let Some(ui) = ui.upgrade() {
                                            let book_models = books.into_iter().map(|book| {
                                                slint_generatedAppWindow::BookViewModel {
                                                    id: SharedString::from(book.id),
                                                    title: SharedString::from(book.title),
                                                    author: SharedString::from(book.author),
                                                    cover: if let Some(cover_path) = book.cover_path {
                                                        slint::Image::load_from_path(&cover_path).unwrap_or_default()
                                                    } else {
                                                        slint::Image::default()
                                                    },
                                                    progress: book.progress,
                                                    status: SharedString::from(book.status),
                                                    is_favorite: book.is_favorite,
                                                    rating: book.rating.unwrap_or(0) as i32,
                                                    last_opened: if let Some(last_opened) = book.last_opened {
                                                        SharedString::from(last_opened.format("%Y-%m-%d").to_string())
                                                    } else {
                                                        SharedString::from("")
                                                    },
                                                    added_date: SharedString::from(book.added_date.format("%Y-%m-%d").to_string()),
                                                }
                                            }).collect::<Vec<_>>();
                                            
                                            ui.set_books(ModelRc::new(VecModel::from(book_models)));
                                        }
                                    }
                                }).unwrap();
                            }
                        }
                        Err(e) => {
                            eprintln!("Error adding book: {}", e);
                        }
                    }
                    
                    slint::invoke_from_event_loop({
                        let ui = ui.clone();
                        move || {
                            if let Some(ui) = ui.upgrade() {
                                ui.set_loading(false);
                            }
                        }
                    }).unwrap();
                }
            });
        });

        // Handle search
        let ui_weak = self.ui.as_weak();
        let book_service_clone = book_service.clone();
        let rt_handle_clone = rt_handle.clone();
        self.ui.on_search_books(move |query| {
            let book_service = book_service_clone.clone();
            let ui = ui_weak.clone();
            let query = query.to_string();
            
            rt_handle_clone.spawn(async move {
                if query.is_empty() {
                    // Show all books
                    if let Ok(books) = book_service.get_library_books().await {
                        slint::invoke_from_event_loop({
                            let ui = ui.clone();
                            move || {
                                if let Some(ui) = ui.upgrade() {
                                    let book_models = books.into_iter().map(|book| {
                                        slint_generatedAppWindow::BookViewModel {
                                            id: SharedString::from(book.id),
                                            title: SharedString::from(book.title),
                                            author: SharedString::from(book.author),
                                            cover: if let Some(cover_path) = book.cover_path {
                                                slint::Image::load_from_path(&cover_path).unwrap_or_default()
                                            } else {
                                                slint::Image::default()
                                            },
                                            progress: book.progress,
                                            status: SharedString::from(book.status),
                                            is_favorite: book.is_favorite,
                                            rating: book.rating.unwrap_or(0) as i32,
                                            last_opened: if let Some(last_opened) = book.last_opened {
                                                SharedString::from(last_opened.format("%Y-%m-%d").to_string())
                                            } else {
                                                SharedString::from("")
                                            },
                                            added_date: SharedString::from(book.added_date.format("%Y-%m-%d").to_string()),
                                        }
                                    }).collect::<Vec<_>>();
                                    
                                    ui.set_books(ModelRc::new(VecModel::from(book_models)));
                                }
                            }
                        }).unwrap();
                    }
                } else {
                    // Search books
                    if let Ok(books) = book_service.search_books(&query).await {
                        slint::invoke_from_event_loop({
                            let ui = ui.clone();
                            move || {
                                if let Some(ui) = ui.upgrade() {
                                    let book_models = books.into_iter().map(|book| {
                                        slint_generatedAppWindow::BookViewModel {
                                            id: SharedString::from(book.id),
                                            title: SharedString::from(book.title),
                                            author: SharedString::from(book.author),
                                            cover: if let Some(cover_path) = book.cover_path {
                                                slint::Image::load_from_path(&cover_path).unwrap_or_default()
                                            } else {
                                                slint::Image::default()
                                            },
                                            progress: book.progress,
                                            status: SharedString::from(book.status),
                                            is_favorite: book.is_favorite,
                                            rating: book.rating.unwrap_or(0) as i32,
                                            last_opened: if let Some(last_opened) = book.last_opened {
                                                SharedString::from(last_opened.format("%Y-%m-%d").to_string())
                                            } else {
                                                SharedString::from("")
                                            },
                                            added_date: SharedString::from(book.added_date.format("%Y-%m-%d").to_string()),
                                        }
                                    }).collect::<Vec<_>>();
                                    
                                    ui.set_books(ModelRc::new(VecModel::from(book_models)));
                                }
                            }
                        }).unwrap();
                    }
                }
            });
        });

        // Handle view mode changes
        let ui_weak = self.ui.as_weak();
        self.ui.on_change_view_mode(move |view_mode| {
            let ui = ui_weak.upgrade().unwrap();
            ui.set_current_view_mode(view_mode);
        });

        // Handle theme changes
        let ui_weak = self.ui.as_weak();
        self.ui.on_change_theme(move |theme| {
            let ui = ui_weak.upgrade().unwrap();
            ui.set_current_theme(theme);
        });

        Ok(())
    }

    /// Load the book library
    fn load_library(&self) -> Result<()> {
        let book_service = self.book_service.clone();
        let ui = self.ui.as_weak();
        
        self.rt.spawn(async move {
            match book_service.get_library_books().await {
                Ok(books) => {
                    let book_models = books.into_iter().map(|book| {
                        slint_generatedAppWindow::BookViewModel {
                            id: SharedString::from(book.id),
                            title: SharedString::from(book.title),
                            author: SharedString::from(book.author),
                            cover: if let Some(cover_path) = book.cover_path {
                                slint::Image::load_from_path(&cover_path).unwrap_or_default()
                            } else {
                                slint::Image::default()
                            },
                            progress: book.progress,
                            status: SharedString::from(book.status),
                            is_favorite: book.is_favorite,
                            rating: book.rating.unwrap_or(0) as i32,
                            last_opened: if let Some(last_opened) = book.last_opened {
                                SharedString::from(last_opened.format("%Y-%m-%d").to_string())
                            } else {
                                SharedString::from("")
                            },
                            added_date: SharedString::from(book.added_date.format("%Y-%m-%d").to_string()),
                        }
                    }).collect::<Vec<_>>();
                    
                    if let Some(ui) = ui.upgrade() {
                        ui.set_books(ModelRc::new(VecModel::from(book_models)));
                    }
                }
                Err(e) => {
                    eprintln!("Error loading library: {}", e);
                }
            }
        });
        
        Ok(())
    }

    /// Run the application
    pub fn run(self) -> Result<()> {
        self.ui.run()?;
        Ok(())
    }
}

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Create and run the application
    let app = EbookReaderApp::new()?;
    app.initialize()?;
    app.run()?;
    
    Ok(())
}