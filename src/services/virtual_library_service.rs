use std::collections::HashMap;
use std::sync::Arc;
use std::ops::Range;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use anyhow::Result;
use lru::LruCache;

use crate::models::book::Book;
use crate::models::library::ReadingStatus;

/// Virtual scrolling grid for efficient rendering of large collections
#[derive(Debug, Clone)]
pub struct VirtualGrid {
    pub visible_range: Range<usize>,
    pub item_height: f32,
    pub items_per_row: usize,
    pub total_items: usize,
    pub viewport_height: f32,
    pub scroll_offset: f32,
}

impl VirtualGrid {
    pub fn new(total_items: usize, items_per_row: usize, item_height: f32, viewport_height: f32) -> Self {
        let visible_range = 0..0;
        
        Self {
            visible_range,
            item_height,
            items_per_row,
            total_items,
            viewport_height,
            scroll_offset: 0.0,
        }
    }
    
    /// Update visible range based on scroll position
    pub fn update_visible_range(&mut self, scroll_offset: f32) {
        self.scroll_offset = scroll_offset;
        
        let rows_visible = (self.viewport_height / self.item_height).ceil() as usize + 2; // +2 for buffer
        let start_row = (scroll_offset / self.item_height).floor() as usize;
        let end_row = (start_row + rows_visible).min(self.total_rows());
        
        let start_index = start_row * self.items_per_row;
        let end_index = (end_row * self.items_per_row).min(self.total_items);
        
        self.visible_range = start_index..end_index;
    }
    
    /// Get total number of rows
    pub fn total_rows(&self) -> usize {
        if self.total_items == 0 {
            0
        } else {
            (self.total_items + self.items_per_row - 1) / self.items_per_row
        }
    }
    
    /// Get total height of the virtual grid
    pub fn total_height(&self) -> f32 {
        self.total_rows() as f32 * self.item_height
    }
    
    /// Check if an item is in the visible range
    pub fn is_item_visible(&self, index: usize) -> bool {
        self.visible_range.contains(&index)
    }
    
    /// Get visible items count
    pub fn visible_items_count(&self) -> usize {
        self.visible_range.len()
    }
}

/// Image cache with LRU eviction and memory management
#[derive(Debug)]
pub struct ImageCache {
    cache: LruCache<String, CachedImage>,
    max_memory_mb: usize,
    current_memory_mb: usize,
    cache_hits: u64,
    cache_misses: u64,
    preload_queue: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CachedImage {
    pub data: Vec<u8>,
    pub format: ImageFormat,
    pub width: u32,
    pub height: u32,
    pub memory_size: usize,
    pub access_time: Instant,
    pub load_time: Duration,
}

#[derive(Debug, Clone)]
pub enum ImageFormat {
    Jpeg,
    Png,
    Webp,
    Avif,
}

impl ImageCache {
    pub fn new(max_memory_mb: usize) -> Self {
        let capacity = (max_memory_mb * 1024 * 1024) / 100000; // Estimate ~100KB per image
        
        Self {
            cache: LruCache::new(std::num::NonZero::new(capacity.try_into().unwrap_or(1000)).unwrap()),
            max_memory_mb,
            current_memory_mb: 0,
            cache_hits: 0,
            cache_misses: 0,
            preload_queue: Vec::new(),
        }
    }
    
    /// Get image from cache
    pub fn get(&mut self, key: &str) -> Option<&CachedImage> {
        if let Some(image) = self.cache.get_mut(key) {
            image.access_time = Instant::now();
            self.cache_hits += 1;
            Some(image)
        } else {
            self.cache_misses += 1;
            None
        }
    }
    
    /// Insert image into cache with memory management
    pub fn insert(&mut self, key: String, image: CachedImage) {
        // Check if we need to evict items to make room
        while self.current_memory_mb + (image.memory_size / 1024 / 1024) > self.max_memory_mb {
            if let Some((_, evicted)) = self.cache.pop_lru() {
                self.current_memory_mb -= evicted.memory_size / 1024 / 1024;
            } else {
                break;
            }
        }
        
        self.current_memory_mb += image.memory_size / 1024 / 1024;
        self.cache.put(key, image);
    }
    
    /// Check if image is cached
    pub fn contains(&self, key: &str) -> bool {
        self.cache.contains(key)
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            cache_hits: self.cache_hits,
            cache_misses: self.cache_misses,
            memory_usage_mb: self.current_memory_mb,
            cache_size: self.cache.len(),
            hit_rate: if self.cache_hits + self.cache_misses > 0 {
                self.cache_hits as f64 / (self.cache_hits + self.cache_misses) as f64
            } else {
                0.0
            },
        }
    }
    
    /// Clear cache
    pub fn clear(&mut self) {
        self.cache.clear();
        self.current_memory_mb = 0;
    }
    
    /// Add to preload queue
    pub fn queue_preload(&mut self, urls: Vec<String>) {
        for url in urls {
            if !self.contains(&url) && !self.preload_queue.contains(&url) {
                self.preload_queue.push(url);
            }
        }
    }
    
    /// Get next item to preload
    pub fn next_preload(&mut self) -> Option<String> {
        self.preload_queue.pop()
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub memory_usage_mb: usize,
    pub cache_size: usize,
    pub hit_rate: f64,
}

/// Performance metrics for monitoring
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub visible_items: usize,
    pub total_items: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub render_time_ms: u64,
    pub memory_usage_mb: usize,
    pub scroll_fps: f32,
    pub load_times: Vec<Duration>,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            visible_items: 0,
            total_items: 0,
            cache_hits: 0,
            cache_misses: 0,
            render_time_ms: 0,
            memory_usage_mb: 0,
            scroll_fps: 0.0,
            load_times: Vec::new(),
        }
    }
    
    /// Calculate average load time
    pub fn average_load_time(&self) -> Duration {
        if self.load_times.is_empty() {
            Duration::from_millis(0)
        } else {
            let total: Duration = self.load_times.iter().sum();
            total / self.load_times.len() as u32
        }
    }
    
    /// Check if performance targets are met
    pub fn meets_targets(&self) -> bool {
        self.render_time_ms < 16 && // 60 FPS target
        self.scroll_fps > 50.0 &&
        self.average_load_time() < Duration::from_secs(1)
    }
}

/// Virtual library service for high-performance book grid
pub struct VirtualLibraryService {
    virtual_grid: Arc<RwLock<VirtualGrid>>,
    image_cache: Arc<RwLock<ImageCache>>,
    performance_metrics: Arc<RwLock<PerformanceMetrics>>,
    books: Arc<RwLock<Vec<Book>>>,
    filtered_books: Arc<RwLock<Vec<Book>>>,
    loading_tasks: Arc<RwLock<HashMap<String, tokio::task::JoinHandle<Result<Vec<u8>>>>>>,
}

impl VirtualLibraryService {
    pub fn new(max_cache_memory_mb: usize) -> Self {
        Self {
            virtual_grid: Arc::new(RwLock::new(VirtualGrid::new(0, 6, 260.0, 600.0))),
            image_cache: Arc::new(RwLock::new(ImageCache::new(max_cache_memory_mb))),
            performance_metrics: Arc::new(RwLock::new(PerformanceMetrics::new())),
            books: Arc::new(RwLock::new(Vec::new())),
            filtered_books: Arc::new(RwLock::new(Vec::new())),
            loading_tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Initialize the service with books
    pub async fn initialize(&self, books: Vec<Book>) -> Result<()> {
        let start_time = Instant::now();
        
        // Store books
        let mut books_guard = self.books.write().await;
        *books_guard = books.clone();
        drop(books_guard);
        
        // Initialize filtered books (same as all books initially)
        let mut filtered_guard = self.filtered_books.write().await;
        *filtered_guard = books.clone();
        drop(filtered_guard);
        
        // Update virtual grid
        let mut grid = self.virtual_grid.write().await;
        grid.total_items = books.len();
        grid.update_visible_range(0.0);
        drop(grid);
        
        // Update performance metrics
        let mut metrics = self.performance_metrics.write().await;
        metrics.total_items = books.len();
        metrics.load_times.push(start_time.elapsed());
        
        Ok(())
    }
    
    /// Update virtual grid viewport
    pub async fn update_viewport(&self, scroll_offset: f32, viewport_height: f32) -> Result<()> {
        let mut grid = self.virtual_grid.write().await;
        grid.viewport_height = viewport_height;
        grid.update_visible_range(scroll_offset);
        
        // Update performance metrics
        let mut metrics = self.performance_metrics.write().await;
        metrics.visible_items = grid.visible_items_count();
        
        Ok(())
    }
    
    /// Get visible books for rendering
    pub async fn get_visible_books(&self) -> Result<Vec<Book>> {
        let grid = self.virtual_grid.read().await;
        let filtered_books = self.filtered_books.read().await;
        
        let visible_books = filtered_books
            .iter()
            .enumerate()
            .filter(|(i, _)| grid.is_item_visible(*i))
            .map(|(_, book)| book.clone())
            .collect();
        
        Ok(visible_books)
    }
    
    /// Load book cover asynchronously
    pub async fn load_cover(&self, book_id: &str, cover_url: &str) -> Result<bool> {
        let start_time = Instant::now();
        
        // Check cache first
        {
            let mut cache = self.image_cache.write().await;
            if cache.contains(cover_url) {
                return Ok(true);
            }
        }
        
        // Check if already loading
        {
            let loading_tasks = self.loading_tasks.read().await;
            if loading_tasks.contains_key(cover_url) {
                return Ok(false); // Still loading
            }
        }
        
        // Start loading task
        let url = cover_url.to_string();
        let cache = self.image_cache.clone();
        let loading_tasks = self.loading_tasks.clone();
        let metrics = self.performance_metrics.clone();
        
        let task = tokio::spawn(async move {
            // Simulate image loading (replace with actual HTTP client)
            let result = Self::download_image(&url).await;
            
            // Remove from loading tasks
            {
                let mut tasks = loading_tasks.write().await;
                tasks.remove(&url);
            }
            
            match result {
                Ok(data) => {
                    let data_len = data.len();
                    let image = CachedImage {
                        memory_size: data_len,
                        data: data.clone(),
                        format: ImageFormat::Jpeg, // Detect format in real implementation
                        width: 150,
                        height: 220,
                        access_time: Instant::now(),
                        load_time: start_time.elapsed(),
                    };
                    
                    let mut cache_guard = cache.write().await;
                    cache_guard.insert(url, image);
                    
                    // Update metrics
                    let mut metrics_guard = metrics.write().await;
                    metrics_guard.load_times.push(start_time.elapsed());
                    
                    Ok(data)
                }
                Err(e) => Err(e),
            }
        });
        
        // Store loading task
        {
            let mut tasks = self.loading_tasks.write().await;
            tasks.insert(cover_url.to_string(), task);
        }
        
        Ok(false)
    }
    
    /// Download image data (placeholder implementation)
    async fn download_image(url: &str) -> Result<Vec<u8>> {
        // In real implementation, use reqwest or similar
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(vec![0u8; 50000]) // Placeholder data
    }
    
    /// Preload covers for visible range
    pub async fn preload_covers(&self, start_index: usize, end_index: usize) -> Result<()> {
        let filtered_books = self.filtered_books.read().await;
        let mut preload_urls = Vec::new();
        
        for i in start_index..end_index.min(filtered_books.len()) {
            if let Some(book) = filtered_books.get(i) {
                if let Some(cover_url) = &book.cover_url {
                    preload_urls.push(cover_url.clone());
                }
            }
        }
        
        drop(filtered_books);
        
        // Queue for preloading
        {
            let mut cache = self.image_cache.write().await;
            cache.queue_preload(preload_urls);
        }
        
        // Start preloading task
        let cache = self.image_cache.clone();
        let loading_tasks = self.loading_tasks.clone();
        let metrics = self.performance_metrics.clone();
        
        tokio::spawn(async move {
            while let Some(url) = {
                let mut cache_guard = cache.write().await;
                cache_guard.next_preload()
            } {
                // Load if not already loading
                {
                    let loading_tasks_guard = loading_tasks.read().await;
                    if loading_tasks_guard.contains_key(&url) {
                        continue;
                    }
                }
                
                // Start loading
                let url_clone = url.clone();
                let cache_clone = cache.clone();
                let loading_tasks_clone = loading_tasks.clone();
                let metrics_clone = metrics.clone();
                
                let task = tokio::spawn(async move {
                    let start_time = Instant::now();
                    let result = Self::download_image(&url_clone).await;
                    
                    {
                        let mut tasks = loading_tasks_clone.write().await;
                        tasks.remove(&url_clone);
                    }
                    
                    if let Ok(data) = &result {
                        let image = CachedImage {
                            memory_size: data.len(),
                            data: data.clone(),
                            format: ImageFormat::Jpeg,
                            width: 150,
                            height: 220,
                            access_time: Instant::now(),
                            load_time: start_time.elapsed(),
                        };
                        
                        let mut cache_guard = cache_clone.write().await;
                        cache_guard.insert(url_clone, image);
                        
                        let mut metrics_guard = metrics_clone.write().await;
                        metrics_guard.load_times.push(start_time.elapsed());
                    }
                    
                    result
                });
                
                {
                    let mut tasks = loading_tasks.write().await;
                    tasks.insert(url, task);
                }
            }
        });
        
        Ok(())
    }
    
    /// Filter books based on search criteria
    pub async fn filter_books(&self, query: &str, status_filter: Option<ReadingStatus>) -> Result<()> {
        let books = self.books.read().await;
        let query_lower = query.to_lowercase();
        
        let filtered: Vec<Book> = books
            .iter()
            .filter(|book| {
                // Text search
                let matches_query = query.is_empty() || 
                    book.title.to_lowercase().contains(&query_lower) ||
                    book.author.to_lowercase().contains(&query_lower);
                
                // Status filter
                let matches_status = status_filter.is_none() || 
                    book.reading_status == *status_filter.as_ref().unwrap();
                
                matches_query && matches_status
            })
            .cloned()
            .collect();
        
        drop(books);
        
        // Update filtered books
        {
            let mut filtered_guard = self.filtered_books.write().await;
            *filtered_guard = filtered;
        }
        
        // Update virtual grid
        let mut grid = self.virtual_grid.write().await;
        grid.total_items = {
            let filtered_guard = self.filtered_books.read().await;
            filtered_guard.len()
        };
        let scroll_offset = grid.scroll_offset;
        grid.update_visible_range(scroll_offset);
        
        Ok(())
    }
    
    /// Get performance metrics
    pub async fn get_performance_metrics(&self) -> PerformanceMetrics {
        let metrics = self.performance_metrics.read().await;
        let cache = self.image_cache.read().await;
        let cache_stats = cache.stats();
        
        PerformanceMetrics {
            visible_items: metrics.visible_items,
            total_items: metrics.total_items,
            cache_hits: cache_stats.cache_hits,
            cache_misses: cache_stats.cache_misses,
            render_time_ms: metrics.render_time_ms,
            memory_usage_mb: cache_stats.memory_usage_mb,
            scroll_fps: metrics.scroll_fps,
            load_times: metrics.load_times.clone(),
        }
    }
    
    /// Update render time metric
    pub async fn update_render_time(&self, render_time: Duration) {
        let mut metrics = self.performance_metrics.write().await;
        metrics.render_time_ms = render_time.as_millis() as u64;
    }
    
    /// Update scroll FPS metric
    pub async fn update_scroll_fps(&self, fps: f32) {
        let mut metrics = self.performance_metrics.write().await;
        metrics.scroll_fps = fps;
    }
    
    /// Check if image is cached
    pub async fn is_image_cached(&self, url: &str) -> bool {
        let cache = self.image_cache.read().await;
        cache.contains(url)
    }
    
    /// Get total items count
    pub async fn get_total_items(&self) -> usize {
        let filtered_books = self.filtered_books.read().await;
        filtered_books.len()
    }
    
    /// Get grid configuration
    pub async fn get_grid_config(&self) -> (usize, f32, f32) {
        let grid = self.virtual_grid.read().await;
        (grid.items_per_row, grid.item_height, grid.total_height())
    }
    
    /// Update grid configuration
    pub async fn update_grid_config(&self, items_per_row: usize, item_height: f32) -> Result<()> {
        let mut grid = self.virtual_grid.write().await;
        grid.items_per_row = items_per_row;
        grid.item_height = item_height;
        let scroll_offset = grid.scroll_offset;
        grid.update_visible_range(scroll_offset);
        Ok(())
    }
    
    /// Clear all caches
    pub async fn clear_caches(&self) {
        let mut cache = self.image_cache.write().await;
        cache.clear();
        
        let mut tasks = self.loading_tasks.write().await;
        tasks.clear();
    }
    
    /// Get memory usage in MB
    pub async fn get_memory_usage(&self) -> usize {
        let cache = self.image_cache.read().await;
        cache.stats().memory_usage_mb
    }
}

impl Default for VirtualLibraryService {
    fn default() -> Self {
        Self::new(100) // 100MB default cache
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::book::Book;
    use crate::models::library::ReadingStatus;
    
    #[test]
    fn test_virtual_grid_calculations() {
        let mut grid = VirtualGrid::new(1000, 6, 260.0, 600.0);
        
        // Test total rows calculation
        assert_eq!(grid.total_rows(), 167); // ceil(1000/6) = 167
        
        // Test visible range update
        grid.update_visible_range(0.0);
        assert_eq!(grid.visible_range.start, 0);
        assert!(grid.visible_range.end > 0);
        
        // Test scroll update
        grid.update_visible_range(1000.0);
        assert!(grid.visible_range.start > 0);
    }
    
    #[test]
    fn test_image_cache() {
        let mut cache = ImageCache::new(1); // 1MB limit
        
        let image = CachedImage {
            data: vec![0u8; 500000], // 500KB
            format: ImageFormat::Jpeg,
            width: 150,
            height: 220,
            memory_size: 500000,
            access_time: Instant::now(),
            load_time: Duration::from_millis(100),
        };
        
        // Test insertion
        cache.insert("test1".to_string(), image);
        assert!(cache.contains("test1"));
        
        // Test hit
        assert!(cache.get("test1").is_some());
        
        // Test miss
        assert!(cache.get("test2").is_none());
        
        let stats = cache.stats();
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 1);
    }
    
    #[tokio::test]
    async fn test_virtual_library_service() {
        let service = VirtualLibraryService::new(50);
        
        // Create test books
        let mut books = Vec::new();
        for i in 0..100 {
            books.push(Book {
                id: format!("book_{}", i),
                title: format!("Test Book {}", i),
                author: format!("Author {}", i),
                cover_url: Some(format!("https://example.com/cover_{}.jpg", i)),
                reading_status: ReadingStatus::Unread,
                ..Default::default()
            });
        }
        
        // Initialize service
        service.initialize(books).await.unwrap();
        
        // Test viewport update
        service.update_viewport(0.0, 600.0).await.unwrap();
        
        // Test visible books
        let visible_books = service.get_visible_books().await.unwrap();
        assert!(!visible_books.is_empty());
        
        // Test filtering
        service.filter_books("Test", None).await.unwrap();
        let filtered_count = service.get_total_items().await;
        assert_eq!(filtered_count, 100); // All books match "Test"
        
        // Test metrics
        let metrics = service.get_performance_metrics().await;
        assert_eq!(metrics.total_items, 100);
    }
}