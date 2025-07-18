use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tokio::time::timeout;
use anyhow::{Result, anyhow};
use reqwest::Client;
use bytes::Bytes;

/// Async image loader with concurrent loading and intelligent prioritization
pub struct AsyncImageLoader {
    client: Client,
    loading_semaphore: Arc<Semaphore>,
    active_downloads: Arc<RwLock<HashMap<String, tokio::task::JoinHandle<Result<LoadedImage>>>>>,
    priority_queue: Arc<RwLock<Vec<LoadRequest>>>,
    max_concurrent_downloads: usize,
    timeout_duration: Duration,
    retry_attempts: usize,
}

#[derive(Debug, Clone)]
pub struct LoadedImage {
    pub data: Bytes,
    pub format: ImageFormat,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub file_size: usize,
    pub load_time: Duration,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImageFormat {
    Jpeg,
    Png,
    Webp,
    Gif,
    Bmp,
    Tiff,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadPriority {
    Immediate,  // Currently visible
    High,       // About to be visible
    Normal,     // Preload for smooth scrolling
    Low,        // Background preload
}

#[derive(Debug, Clone)]
pub struct LoadRequest {
    pub url: String,
    pub priority: LoadPriority,
    pub requested_at: Instant,
    pub viewport_distance: f32, // Distance from current viewport
}

impl AsyncImageLoader {
    pub fn new(max_concurrent_downloads: usize, timeout_seconds: u64) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            client,
            loading_semaphore: Arc::new(Semaphore::new(max_concurrent_downloads)),
            active_downloads: Arc::new(RwLock::new(HashMap::new())),
            priority_queue: Arc::new(RwLock::new(Vec::new())),
            max_concurrent_downloads,
            timeout_duration: Duration::from_secs(timeout_seconds),
            retry_attempts: 3,
        }
    }
    
    /// Load image with priority and caching
    pub async fn load_image(&self, url: &str, priority: LoadPriority) -> Result<LoadedImage> {
        let start_time = Instant::now();
        
        // Check if already loading
        {
            let active_downloads = self.active_downloads.read().await;
            if active_downloads.contains_key(url) {
                // If already loading, return early (could implement waiting logic here)
                return Err(anyhow!("Already loading"));
            }
        }
        
        // Add to priority queue if not immediate
        if priority != LoadPriority::Immediate {
            self.add_to_queue(url.to_string(), priority).await;
            return Err(anyhow!("Added to queue"));
        }
        
        // Load immediately
        self.load_with_semaphore(url.to_string(), start_time).await
    }
    
    /// Load image with semaphore control
    async fn load_with_semaphore(&self, url: String, start_time: Instant) -> Result<LoadedImage> {
        let _permit = self.loading_semaphore.acquire().await
            .map_err(|_| anyhow!("Failed to acquire semaphore"))?;
        
        let result = self.download_image(&url).await;
        
        // Remove from active downloads
        {
            let mut active_downloads = self.active_downloads.write().await;
            active_downloads.remove(&url);
        }
        
        match result {
            Ok(mut image) => {
                image.load_time = start_time.elapsed();
                Ok(image)
            }
            Err(e) => Err(e),
        }
    }
    
    /// Download image with retries
    async fn download_image(&self, url: &str) -> Result<LoadedImage> {
        let mut last_error = None;
        
        for attempt in 0..self.retry_attempts {
            match self.download_image_once(url).await {
                Ok(image) => return Ok(image),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.retry_attempts - 1 {
                        let delay = Duration::from_millis(100 * (2_u64.pow(attempt as u32)));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| anyhow!("Failed to download image")))
    }
    
    /// Single download attempt
    async fn download_image_once(&self, url: &str) -> Result<LoadedImage> {
        let response = timeout(self.timeout_duration, self.client.get(url).send()).await??;
        
        if !response.status().is_success() {
            return Err(anyhow!("HTTP error: {}", response.status()));
        }
        
        let content_type = response.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();
        
        let data = response.bytes().await?;
        let format = Self::detect_format(&data, &content_type);
        let (width, height) = Self::extract_dimensions(&data, &format);
        
        Ok(LoadedImage {
            file_size: data.len(),
            data,
            format,
            width,
            height,
            load_time: Duration::from_millis(0), // Will be set by caller
            url: url.to_string(),
        })
    }
    
    /// Detect image format from data and content type
    fn detect_format(data: &[u8], content_type: &str) -> ImageFormat {
        if content_type.contains("jpeg") || content_type.contains("jpg") {
            return ImageFormat::Jpeg;
        }
        if content_type.contains("png") {
            return ImageFormat::Png;
        }
        if content_type.contains("webp") {
            return ImageFormat::Webp;
        }
        if content_type.contains("gif") {
            return ImageFormat::Gif;
        }
        
        // Check magic bytes
        if data.len() >= 2 {
            match &data[0..2] {
                [0xFF, 0xD8] => ImageFormat::Jpeg,
                [0x89, 0x50] if data.len() >= 8 && &data[1..8] == b"NG\r\n\x1A\n" => ImageFormat::Png,
                [0x52, 0x49] if data.len() >= 12 && &data[8..12] == b"WEBP" => ImageFormat::Webp,
                [0x47, 0x49] if data.len() >= 6 && &data[0..6] == b"GIF87a" => ImageFormat::Gif,
                [0x47, 0x49] if data.len() >= 6 && &data[0..6] == b"GIF89a" => ImageFormat::Gif,
                [0x42, 0x4D] => ImageFormat::Bmp,
                _ => ImageFormat::Unknown,
            }
        } else {
            ImageFormat::Unknown
        }
    }
    
    /// Extract image dimensions (simplified - would use image crate in real implementation)
    fn extract_dimensions(data: &[u8], format: &ImageFormat) -> (Option<u32>, Option<u32>) {
        match format {
            ImageFormat::Jpeg => Self::extract_jpeg_dimensions(data),
            ImageFormat::Png => Self::extract_png_dimensions(data),
            _ => (None, None),
        }
    }
    
    /// Extract JPEG dimensions from header
    fn extract_jpeg_dimensions(data: &[u8]) -> (Option<u32>, Option<u32>) {
        if data.len() < 10 {
            return (None, None);
        }
        
        // Simplified JPEG parsing - would use proper parser in real implementation
        let mut i = 2; // Skip SOI marker
        while i < data.len() - 8 {
            if data[i] == 0xFF {
                let marker = data[i + 1];
                if marker == 0xC0 || marker == 0xC2 { // SOF0 or SOF2
                    if i + 9 < data.len() {
                        let height = u16::from_be_bytes([data[i + 5], data[i + 6]]) as u32;
                        let width = u16::from_be_bytes([data[i + 7], data[i + 8]]) as u32;
                        return (Some(width), Some(height));
                    }
                }
                let length = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
                i += 2 + length;
            } else {
                i += 1;
            }
        }
        
        (None, None)
    }
    
    /// Extract PNG dimensions from header
    fn extract_png_dimensions(data: &[u8]) -> (Option<u32>, Option<u32>) {
        if data.len() < 24 {
            return (None, None);
        }
        
        // PNG IHDR chunk starts at byte 16
        if &data[12..16] == b"IHDR" {
            let width = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
            let height = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
            return (Some(width), Some(height));
        }
        
        (None, None)
    }
    
    /// Add request to priority queue
    async fn add_to_queue(&self, url: String, priority: LoadPriority) {
        let request = LoadRequest {
            url,
            priority,
            requested_at: Instant::now(),
            viewport_distance: 0.0,
        };
        
        let mut queue = self.priority_queue.write().await;
        queue.push(request);
        
        // Sort by priority and distance
        queue.sort_by(|a, b| {
            match a.priority.cmp(&b.priority) {
                std::cmp::Ordering::Equal => {
                    a.viewport_distance.partial_cmp(&b.viewport_distance)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }
                other => other,
            }
        });
    }
    
    /// Process priority queue
    pub async fn process_queue(&self) -> Result<()> {
        let mut queue = self.priority_queue.write().await;
        
        // Process highest priority items first
        while let Some(request) = queue.pop() {
            // Check if we have capacity
            if self.loading_semaphore.available_permits() == 0 {
                // Put back and wait
                queue.push(request);
                break;
            }
            
            // Start loading
            let url = request.url.clone();
            let loader = self.clone();
            
            let handle = tokio::spawn(async move {
                loader.load_with_semaphore(url, Instant::now()).await
            });
            
            // Track active download
            {
                let mut active_downloads = self.active_downloads.write().await;
                active_downloads.insert(request.url, handle);
            }
        }
        
        Ok(())
    }
    
    /// Cancel all downloads
    pub async fn cancel_all(&self) {
        let mut active_downloads = self.active_downloads.write().await;
        
        for (_, handle) in active_downloads.drain() {
            handle.abort();
        }
        
        let mut queue = self.priority_queue.write().await;
        queue.clear();
    }
    
    /// Cancel specific download
    pub async fn cancel_download(&self, url: &str) {
        let mut active_downloads = self.active_downloads.write().await;
        
        if let Some(handle) = active_downloads.remove(url) {
            handle.abort();
        }
        
        // Remove from queue
        let mut queue = self.priority_queue.write().await;
        queue.retain(|req| req.url != url);
    }
    
    /// Get loading statistics
    pub async fn get_stats(&self) -> LoadingStats {
        let active_downloads = self.active_downloads.read().await;
        let queue = self.priority_queue.read().await;
        
        LoadingStats {
            active_downloads: active_downloads.len(),
            queued_requests: queue.len(),
            available_permits: self.loading_semaphore.available_permits(),
            max_concurrent: self.max_concurrent_downloads,
        }
    }
    
    /// Update viewport distance for prioritization
    pub async fn update_viewport_distances(&self, url_distances: HashMap<String, f32>) {
        let mut queue = self.priority_queue.write().await;
        
        for request in queue.iter_mut() {
            if let Some(distance) = url_distances.get(&request.url) {
                request.viewport_distance = *distance;
            }
        }
        
        // Resort queue
        queue.sort_by(|a, b| {
            match a.priority.cmp(&b.priority) {
                std::cmp::Ordering::Equal => {
                    a.viewport_distance.partial_cmp(&b.viewport_distance)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }
                other => other,
            }
        });
    }
    
    /// Preload images for smooth scrolling
    pub async fn preload_for_scroll(&self, urls: Vec<String>, scroll_direction: ScrollDirection) {
        let priority = match scroll_direction {
            ScrollDirection::Down => LoadPriority::High,
            ScrollDirection::Up => LoadPriority::Normal,
            ScrollDirection::None => LoadPriority::Low,
        };
        
        for url in urls {
            self.add_to_queue(url, priority.clone()).await;
        }
        
        // Process queue
        let _ = self.process_queue().await;
    }
    
    /// Check if image is currently loading
    pub async fn is_loading(&self, url: &str) -> bool {
        let active_downloads = self.active_downloads.read().await;
        active_downloads.contains_key(url)
    }
    
    /// Get queue size
    pub async fn queue_size(&self) -> usize {
        let queue = self.priority_queue.read().await;
        queue.len()
    }
}

impl Clone for AsyncImageLoader {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            loading_semaphore: self.loading_semaphore.clone(),
            active_downloads: self.active_downloads.clone(),
            priority_queue: self.priority_queue.clone(),
            max_concurrent_downloads: self.max_concurrent_downloads,
            timeout_duration: self.timeout_duration,
            retry_attempts: self.retry_attempts,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoadingStats {
    pub active_downloads: usize,
    pub queued_requests: usize,
    pub available_permits: usize,
    pub max_concurrent: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScrollDirection {
    Up,
    Down,
    None,
}

impl PartialOrd for LoadPriority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LoadPriority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (LoadPriority::Immediate, LoadPriority::Immediate) => std::cmp::Ordering::Equal,
            (LoadPriority::Immediate, _) => std::cmp::Ordering::Less,
            (_, LoadPriority::Immediate) => std::cmp::Ordering::Greater,
            (LoadPriority::High, LoadPriority::High) => std::cmp::Ordering::Equal,
            (LoadPriority::High, _) => std::cmp::Ordering::Less,
            (_, LoadPriority::High) => std::cmp::Ordering::Greater,
            (LoadPriority::Normal, LoadPriority::Normal) => std::cmp::Ordering::Equal,
            (LoadPriority::Normal, LoadPriority::Low) => std::cmp::Ordering::Less,
            (LoadPriority::Low, LoadPriority::Normal) => std::cmp::Ordering::Greater,
            (LoadPriority::Low, LoadPriority::Low) => std::cmp::Ordering::Equal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_image_format_detection() {
        // JPEG
        let jpeg_data = [0xFF, 0xD8, 0xFF, 0xE0];
        assert_eq!(AsyncImageLoader::detect_format(&jpeg_data, "image/jpeg"), ImageFormat::Jpeg);
        
        // PNG
        let png_data = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert_eq!(AsyncImageLoader::detect_format(&png_data, "image/png"), ImageFormat::Png);
        
        // Content type detection
        assert_eq!(AsyncImageLoader::detect_format(&[], "image/webp"), ImageFormat::Webp);
    }
    
    #[test]
    fn test_priority_ordering() {
        assert!(LoadPriority::Immediate < LoadPriority::High);
        assert!(LoadPriority::High < LoadPriority::Normal);
        assert!(LoadPriority::Normal < LoadPriority::Low);
    }
    
    #[tokio::test]
    async fn test_async_image_loader() {
        let loader = AsyncImageLoader::new(5, 10);
        
        // Test stats
        let stats = loader.get_stats().await;
        assert_eq!(stats.max_concurrent, 5);
        assert_eq!(stats.available_permits, 5);
        
        // Test queue operations
        loader.add_to_queue("test1".to_string(), LoadPriority::High).await;
        loader.add_to_queue("test2".to_string(), LoadPriority::Low).await;
        
        assert_eq!(loader.queue_size().await, 2);
        
        // Test cancellation
        loader.cancel_all().await;
        assert_eq!(loader.queue_size().await, 0);
    }
    
    #[tokio::test]
    async fn test_viewport_distance_update() {
        let loader = AsyncImageLoader::new(5, 10);
        
        loader.add_to_queue("test1".to_string(), LoadPriority::Normal).await;
        loader.add_to_queue("test2".to_string(), LoadPriority::Normal).await;
        
        let mut distances = HashMap::new();
        distances.insert("test1".to_string(), 10.0);
        distances.insert("test2".to_string(), 5.0);
        
        loader.update_viewport_distances(distances).await;
        
        // Queue should be sorted by distance now
        let queue = loader.priority_queue.read().await;
        assert_eq!(queue[0].url, "test2"); // Closer to viewport
        assert_eq!(queue[1].url, "test1");
    }
}