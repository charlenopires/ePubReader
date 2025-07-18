// Example integration showing how to use the high-performance library components

use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;

use crate::services::{
    VirtualLibraryService, 
    AsyncImageLoader, 
    PerformanceMonitor, 
    BookSearchService,
    SyncService,
    LoadPriority
};
use crate::models::book::Book;
use crate::models::library::BookStatus;

/// High-performance library controller integrating all services
pub struct HighPerformanceLibraryController {
    virtual_library: Arc<VirtualLibraryService>,
    image_loader: Arc<AsyncImageLoader>,
    performance_monitor: Arc<PerformanceMonitor>,
    search_service: Arc<BookSearchService>,
    sync_service: Arc<SyncService>,
    
    // State
    current_scroll_offset: f32,
    viewport_height: f32,
    is_scrolling: bool,
    last_scroll_time: std::time::Instant,
}

impl HighPerformanceLibraryController {
    /// Create new high-performance library controller
    pub fn new() -> Self {
        let virtual_library = Arc::new(VirtualLibraryService::new(100)); // 100MB cache
        let image_loader = Arc::new(AsyncImageLoader::new(8, 30)); // 8 concurrent, 30s timeout
        let performance_monitor = Arc::new(PerformanceMonitor::new(None));
        let search_service = Arc::new(BookSearchService::new());
        let sync_service = Arc::new(SyncService::new(
            Arc::new(crate::services::AnnotationService::new()),
            Arc::new(crate::services::LibraryService::new()),
            "device123".to_string(),
            "My Reading Device".to_string(),
        ));
        
        Self {
            virtual_library,
            image_loader,
            performance_monitor,
            search_service,
            sync_service,
            current_scroll_offset: 0.0,
            viewport_height: 600.0,
            is_scrolling: false,
            last_scroll_time: std::time::Instant::now(),
        }
    }
    
    /// Initialize the library with books
    pub async fn initialize(&mut self, books: Vec<Book>) -> Result<(), Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        
        // Start performance monitoring
        self.performance_monitor.start_monitoring().await;
        
        // Initialize virtual library
        self.virtual_library.initialize(books.clone()).await?;
        
        // Initialize sync service
        self.sync_service.initialize().await?;
        
        // Record startup time
        let startup_time = start_time.elapsed();
        self.performance_monitor.record_startup_time(startup_time).await;
        
        // Start background tasks
        self.start_background_tasks().await;
        
        println!("✅ Library initialized with {} books in {:?}", books.len(), startup_time);
        
        Ok(())
    }
    
    /// Start background tasks for performance optimization
    async fn start_background_tasks(&self) {
        let virtual_library = self.virtual_library.clone();
        let image_loader = self.image_loader.clone();
        let performance_monitor = self.performance_monitor.clone();
        
        // Image preloading task
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(100));
            
            loop {
                interval.tick().await;
                
                // Process image loading queue
                if let Err(e) = image_loader.process_queue().await {
                    eprintln!("Error processing image queue: {}", e);
                }
                
                // Update performance metrics
                let stats = image_loader.get_stats().await;
                // Update metrics based on loading stats
            }
        });
        
        // Performance monitoring task
        let perf_monitor = self.performance_monitor.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(5));
            
            loop {
                interval.tick().await;
                
                // Generate performance report
                let report = perf_monitor.generate_report().await;
                
                // Log performance issues
                for alert in &report.alerts {
                    match alert.severity {
                        crate::services::AlertSeverity::Critical => {
                            eprintln!("🚨 Critical: {}", alert.message);
                        }
                        crate::services::AlertSeverity::Warning => {
                            eprintln!("⚠️  Warning: {}", alert.message);
                        }
                        crate::services::AlertSeverity::Info => {
                            println!("ℹ️  Info: {}", alert.message);
                        }
                    }
                }
                
                // Print performance score
                println!("📊 Performance Score: {:.1}/100", report.overall_score);
            }
        });
    }
    
    /// Handle viewport update (called when user scrolls)
    pub async fn handle_viewport_update(&mut self, scroll_offset: f32, viewport_height: f32) -> Result<(), Box<dyn std::error::Error>> {
        let frame_start = std::time::Instant::now();
        
        // Update scroll state
        self.current_scroll_offset = scroll_offset;
        self.viewport_height = viewport_height;
        self.is_scrolling = true;
        self.last_scroll_time = std::time::Instant::now();
        
        // Update virtual library viewport
        self.virtual_library.update_viewport(scroll_offset, viewport_height).await?;
        
        // Get visible books for preloading
        let visible_books = self.virtual_library.get_visible_books().await?;
        
        // Preload covers for visible books
        let cover_urls: Vec<String> = visible_books
            .iter()
            .filter_map(|book| book.cover_url.clone())
            .collect();
        
        if !cover_urls.is_empty() {
            self.image_loader.preload_for_scroll(
                cover_urls,
                crate::services::ScrollDirection::Down, // Detect direction based on scroll delta
            ).await;
        }
        
        // Update performance metrics
        let frame_time = frame_start.elapsed();
        self.performance_monitor.record_frame_time(frame_time).await;
        
        // Update library stats
        let total_books = self.virtual_library.get_total_items().await;
        self.performance_monitor.update_library_stats(total_books, visible_books.len()).await;
        
        Ok(())
    }
    
    /// Handle book selection
    pub async fn handle_book_selection(&self, book_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("📖 Book selected: {}", book_id);
        
        // Record selection event
        let selection_time = std::time::Instant::now();
        
        // Update sync service with reading activity
        let reading_session = self.sync_service.start_reading_session(book_id.to_string()).await?;
        println!("📝 Started reading session: {}", reading_session);
        
        Ok(())
    }
    
    /// Handle book opening
    pub async fn handle_book_open(&self, book_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        
        println!("📚 Opening book: {}", book_id);
        
        // Simulate book opening process
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        // Record open time
        let open_time = start_time.elapsed();
        self.performance_monitor.record_book_open_time(open_time).await;
        
        // Update reading progress
        use crate::models::sync::BookProgress;
        let progress = BookProgress::new(book_id.to_string());
        self.sync_service.update_book_progress(book_id.to_string(), progress).await?;
        
        println!("✅ Book opened in {:?}", open_time);
        
        Ok(())
    }
    
    /// Handle search request
    pub async fn handle_search(&self, query: &str) -> Result<Vec<crate::services::SearchResult>, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        
        println!("🔍 Searching for: '{}'", query);
        
        // Perform search (assuming we have a current book)
        let search_options = crate::services::SearchOptions {
            case_sensitive: false,
            whole_words: false,
            regex_mode: false,
            context_length: 100,
            max_results: 50,
        };
        
        // For demonstration, search in first book
        let books = self.virtual_library.get_visible_books().await?;
        let results = if let Some(book) = books.first() {
            self.search_service.search_in_book(&book.id, query, &search_options).await?
        } else {
            Vec::new()
        };
        
        // Record search time
        let search_time = start_time.elapsed();
        self.performance_monitor.record_search_time(search_time).await;
        
        println!("✅ Search completed in {:?}, found {} results", search_time, results.len());
        
        Ok(results)
    }
    
    /// Handle filter change
    pub async fn handle_filter_change(&self, filter: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔽 Filter changed to: {}", filter);
        
        let status_filter = match filter {
            "reading" => Some(BookStatus::CurrentlyReading),
            "finished" => Some(BookStatus::Finished),
            "unread" => Some(BookStatus::Unread),
            _ => None,
        };
        
        // Apply filter
        self.virtual_library.filter_books("", status_filter).await?;
        
        Ok(())
    }
    
    /// Handle grid configuration change
    pub async fn handle_grid_config_change(&self, items_per_row: usize) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔧 Grid config changed: {} items per row", items_per_row);
        
        // Update grid configuration
        self.virtual_library.update_grid_config(items_per_row, 260.0).await?;
        
        Ok(())
    }
    
    /// Handle performance monitoring toggle
    pub async fn handle_performance_toggle(&self, enabled: bool) -> Result<(), Box<dyn std::error::Error>> {
        if enabled {
            println!("📊 Performance monitoring enabled");
            self.performance_monitor.start_monitoring().await;
        } else {
            println!("📊 Performance monitoring disabled");
            self.performance_monitor.stop_monitoring().await;
        }
        
        Ok(())
    }
    
    /// Handle library refresh
    pub async fn handle_library_refresh(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔄 Refreshing library...");
        
        // Clear caches
        self.virtual_library.clear_caches().await;
        self.image_loader.cancel_all().await;
        
        // Perform sync
        self.sync_service.perform_full_sync().await?;
        
        println!("✅ Library refreshed");
        
        Ok(())
    }
    
    /// Get current performance metrics
    pub async fn get_performance_metrics(&self) -> crate::services::PerformanceMetrics {
        self.performance_monitor.get_metrics().await
    }
    
    /// Get visible books for UI
    pub async fn get_visible_books(&self) -> Result<Vec<Book>, Box<dyn std::error::Error>> {
        Ok(self.virtual_library.get_visible_books().await?)
    }
    
    /// Load book cover
    pub async fn load_book_cover(&self, book_id: &str, cover_url: &str) -> Result<bool, Box<dyn std::error::Error>> {
        // Check if already cached
        if self.virtual_library.is_image_cached(cover_url).await {
            return Ok(true);
        }
        
        // Load with high priority if currently visible
        let priority = LoadPriority::High; // Would determine based on visibility
        
        match self.image_loader.load_image(cover_url, priority).await {
            Ok(loaded_image) => {
                // Record load time
                self.performance_monitor.record_image_load_time(loaded_image.load_time).await;
                
                // Cache the image
                self.virtual_library.load_cover(book_id, cover_url).await?;
                
                Ok(true)
            }
            Err(_) => {
                // Add to preload queue
                self.image_loader.preload_for_scroll(
                    vec![cover_url.to_string()],
                    crate::services::ScrollDirection::None,
                ).await;
                
                Ok(false)
            }
        }
    }
    
    /// Check if scrolling has stopped
    pub async fn check_scroll_stop(&mut self) {
        if self.is_scrolling && self.last_scroll_time.elapsed() > Duration::from_millis(150) {
            self.is_scrolling = false;
            
            // Trigger preloading after scroll stops
            let visible_books = self.virtual_library.get_visible_books().await.unwrap_or_default();
            let cover_urls: Vec<String> = visible_books
                .iter()
                .filter_map(|book| book.cover_url.clone())
                .collect();
            
            if !cover_urls.is_empty() {
                self.image_loader.preload_for_scroll(
                    cover_urls,
                    crate::services::ScrollDirection::None,
                ).await;
            }
        }
    }
    
    /// Generate performance report
    pub async fn generate_performance_report(&self) -> crate::services::PerformanceReport {
        self.performance_monitor.generate_report().await
    }
    
    /// Cleanup resources
    pub async fn cleanup(&self) {
        println!("🧹 Cleaning up resources...");
        
        // Stop monitoring
        self.performance_monitor.stop_monitoring().await;
        
        // Cancel all downloads
        self.image_loader.cancel_all().await;
        
        // Clear caches
        self.virtual_library.clear_caches().await;
        
        println!("✅ Cleanup completed");
    }
}

// Example usage in main application
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting High-Performance ePub Reader");
    
    // Create controller
    let mut controller = HighPerformanceLibraryController::new();
    
    // Create sample books
    let books = create_sample_books(1000);
    
    // Initialize
    controller.initialize(books).await?;
    
    // Simulate user interactions
    simulate_user_interactions(&mut controller).await?;
    
    // Generate final report
    let report = controller.generate_performance_report().await;
    println!("\n📊 Final Performance Report:");
    println!("Overall Score: {:.1}/100", report.overall_score);
    
    for recommendation in &report.recommendations {
        println!("💡 {}", recommendation);
    }
    
    // Cleanup
    controller.cleanup().await;
    
    Ok(())
}

/// Create sample books for testing
fn create_sample_books(count: usize) -> Vec<Book> {
    let mut books = Vec::new();
    
    for i in 0..count {
        books.push(Book {
            id: format!("book_{}", i),
            title: format!("Sample Book {}", i + 1),
            author: format!("Author {}", (i % 100) + 1),
            cover_url: Some(format!("https://picsum.photos/150/220?random={}", i)),
            status: match i % 3 {
                0 => BookStatus::Unread,
                1 => BookStatus::CurrentlyReading,
                _ => BookStatus::Finished,
            },
            ..Default::default()
        });
    }
    
    books
}

/// Simulate user interactions for testing
async fn simulate_user_interactions(controller: &mut HighPerformanceLibraryController) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🎭 Simulating user interactions...");
    
    // Simulate scrolling
    for i in 0..10 {
        let scroll_offset = i as f32 * 100.0;
        controller.handle_viewport_update(scroll_offset, 600.0).await?;
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    
    // Simulate book selection
    controller.handle_book_selection("book_0").await?;
    
    // Simulate book opening
    controller.handle_book_open("book_0").await?;
    
    // Simulate search
    let _results = controller.handle_search("sample").await?;
    
    // Simulate filter change
    controller.handle_filter_change("reading").await?;
    
    // Simulate grid config change
    controller.handle_grid_config_change(4).await?;
    
    // Wait for background tasks
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Check scroll stop
    controller.check_scroll_stop().await;
    
    println!("✅ User interactions completed");
    
    Ok(())
}

/// Performance test function
pub async fn performance_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏁 Running performance tests...");
    
    let mut controller = HighPerformanceLibraryController::new();
    
    // Test with different library sizes
    for size in [100, 500, 1000, 2000] {
        println!("\n📚 Testing with {} books", size);
        
        let books = create_sample_books(size);
        let start_time = std::time::Instant::now();
        
        controller.initialize(books).await?;
        
        let init_time = start_time.elapsed();
        println!("Initialization: {:?}", init_time);
        
        // Test scrolling performance
        let scroll_start = std::time::Instant::now();
        for i in 0..50 {
            controller.handle_viewport_update(i as f32 * 50.0, 600.0).await?;
        }
        let scroll_time = scroll_start.elapsed();
        println!("Scrolling 50 steps: {:?}", scroll_time);
        
        // Test search performance
        let search_start = std::time::Instant::now();
        let _results = controller.handle_search("sample").await?;
        let search_time = search_start.elapsed();
        println!("Search: {:?}", search_time);
        
        // Get final metrics
        let metrics = controller.get_performance_metrics().await;
        println!("Final FPS: {:.1}", metrics.current_fps);
        println!("Memory Usage: {:.1}MB", metrics.memory_usage_mb);
        
        // Check targets
        let targets_met = metrics.startup_time_ms < 2000 && 
                         metrics.search_time_ms < 500 && 
                         metrics.current_fps > 50.0;
        
        println!("Performance targets met: {}", targets_met);
        
        controller.cleanup().await;
    }
    
    println!("\n✅ Performance tests completed");
    
    Ok(())
}