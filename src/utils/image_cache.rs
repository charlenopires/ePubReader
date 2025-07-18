use std::path::{Path, PathBuf};
use std::fs;
use anyhow::Result;
use tokio::fs as async_fs;
use image::{ImageFormat, DynamicImage};

/// Image cache service for managing book covers and thumbnails
pub struct ImageCache {
    cache_dir: PathBuf,
    covers_dir: PathBuf,
    thumbnails_dir: PathBuf,
}

impl ImageCache {
    /// Create a new image cache instance
    pub fn new(cache_dir: PathBuf) -> Result<Self> {
        let covers_dir = cache_dir.join("covers");
        let thumbnails_dir = cache_dir.join("thumbnails");
        
        // Create directories if they don't exist
        fs::create_dir_all(&covers_dir)?;
        fs::create_dir_all(&thumbnails_dir)?;
        
        Ok(Self {
            cache_dir,
            covers_dir,
            thumbnails_dir,
        })
    }

    /// Save a book cover image
    pub async fn save_cover(&self, book_id: &str, image_data: &[u8]) -> Result<PathBuf> {
        let cover_path = self.covers_dir.join(format!("{}.jpg", book_id));
        
        // Load and convert image to JPEG
        let image = image::load_from_memory(image_data)?;
        let mut output = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut output);
        image.write_to(&mut cursor, ImageFormat::Jpeg)?;
        
        // Save the converted image
        async_fs::write(&cover_path, output).await?;
        
        // Generate thumbnail
        self.generate_thumbnail(book_id, &image).await?;
        
        Ok(cover_path)
    }

    /// Generate thumbnail for a book cover
    async fn generate_thumbnail(&self, book_id: &str, image: &DynamicImage) -> Result<()> {
        let thumbnail_path = self.get_thumbnail_path(book_id);
        
        // Create thumbnail (200x300 pixels)
        let thumbnail = image.resize_to_fill(200, 300, image::imageops::FilterType::Lanczos3);
        
        // Save thumbnail
        let mut output = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut output);
        thumbnail.write_to(&mut cursor, ImageFormat::Jpeg)?;
        
        async_fs::write(&thumbnail_path, output).await?;
        
        Ok(())
    }

    /// Get the path for a book's thumbnail
    pub fn get_thumbnail_path(&self, book_id: &str) -> PathBuf {
        self.thumbnails_dir.join(format!("{}_thumb.jpg", book_id))
    }

    /// Get the path for a book's cover
    pub fn get_cover_path(&self, book_id: &str) -> PathBuf {
        self.covers_dir.join(format!("{}.jpg", book_id))
    }

    /// Check if a cover exists
    pub fn cover_exists(&self, book_id: &str) -> bool {
        self.get_cover_path(book_id).exists()
    }

    /// Check if a thumbnail exists
    pub fn thumbnail_exists(&self, book_id: &str) -> bool {
        self.get_thumbnail_path(book_id).exists()
    }

    /// Remove a book's cover and thumbnail
    pub async fn remove_cover(&self, cover_path: &Path) -> Result<()> {
        if cover_path.exists() {
            async_fs::remove_file(cover_path).await?;
        }
        
        // Also remove thumbnail if it exists
        if let Some(file_stem) = cover_path.file_stem() {
            let thumbnail_path = self.thumbnails_dir.join(format!("{}_thumb.jpg", file_stem.to_string_lossy()));
            if thumbnail_path.exists() {
                async_fs::remove_file(thumbnail_path).await?;
            }
        }
        
        Ok(())
    }

    /// Clear all cached images
    pub async fn clear_cache(&self) -> Result<()> {
        // Remove all files in covers directory
        let mut covers_dir = async_fs::read_dir(&self.covers_dir).await?;
        while let Some(entry) = covers_dir.next_entry().await? {
            async_fs::remove_file(entry.path()).await?;
        }
        
        // Remove all files in thumbnails directory
        let mut thumbnails_dir = async_fs::read_dir(&self.thumbnails_dir).await?;
        while let Some(entry) = thumbnails_dir.next_entry().await? {
            async_fs::remove_file(entry.path()).await?;
        }
        
        Ok(())
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> Result<CacheStats> {
        let covers_count = self.count_files_in_dir(&self.covers_dir).await?;
        let thumbnails_count = self.count_files_in_dir(&self.thumbnails_dir).await?;
        let total_size = self.calculate_directory_size(&self.cache_dir).await?;
        
        Ok(CacheStats {
            covers_count,
            thumbnails_count,
            total_size_bytes: total_size,
        })
    }

    /// Count files in a directory
    async fn count_files_in_dir(&self, dir: &Path) -> Result<usize> {
        let mut count = 0;
        let mut entries = async_fs::read_dir(dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_file() {
                count += 1;
            }
        }
        
        Ok(count)
    }

    /// Calculate total size of a directory
    fn calculate_directory_size<'a>(&'a self, dir: &'a Path) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<u64>> + Send + 'a>> {
        Box::pin(async move {
            let mut total_size = 0;
            let mut entries = async_fs::read_dir(dir).await?;
            
            while let Some(entry) = entries.next_entry().await? {
                let metadata = entry.metadata().await?;
                if metadata.is_file() {
                    total_size += metadata.len();
                } else if metadata.is_dir() {
                    total_size += self.calculate_directory_size(&entry.path()).await?;
                }
            }
            
            Ok(total_size)
        })
    }

    /// Cleanup old unused cache files
    pub async fn cleanup_unused(&self, active_book_ids: &[String]) -> Result<()> {
        // Get all cached files
        let mut covers_dir = async_fs::read_dir(&self.covers_dir).await?;
        let mut thumbnails_dir = async_fs::read_dir(&self.thumbnails_dir).await?;
        
        // Remove unused covers
        while let Some(entry) = covers_dir.next_entry().await? {
            if let Some(file_stem) = entry.path().file_stem() {
                let book_id = file_stem.to_string_lossy().to_string();
                if !active_book_ids.contains(&book_id) {
                    async_fs::remove_file(entry.path()).await?;
                }
            }
        }
        
        // Remove unused thumbnails
        while let Some(entry) = thumbnails_dir.next_entry().await? {
            if let Some(file_stem) = entry.path().file_stem() {
                let filename = file_stem.to_string_lossy();
                if let Some(book_id) = filename.strip_suffix("_thumb") {
                    if !active_book_ids.contains(&book_id.to_string()) {
                        async_fs::remove_file(entry.path()).await?;
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Create a placeholder image for books without covers
    pub async fn create_placeholder(&self, book_id: &str, title: &str, author: &str) -> Result<PathBuf> {
        let placeholder_path = self.get_cover_path(book_id);
        
        // Create a simple placeholder image (200x300)
        let image = self.generate_placeholder_image(title, author)?;
        
        // Save placeholder
        let mut output = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut output);
        image.write_to(&mut cursor, ImageFormat::Jpeg)?;
        
        async_fs::write(&placeholder_path, output).await?;
        
        // Generate thumbnail
        self.generate_thumbnail(book_id, &image).await?;
        
        Ok(placeholder_path)
    }

    /// Generate a placeholder image with book title and author
    fn generate_placeholder_image(&self, title: &str, author: &str) -> Result<DynamicImage> {
        use image::{Rgb, RgbImage};
        
        // Create a simple colored rectangle as placeholder
        let width = 200;
        let height = 300;
        let mut image = RgbImage::new(width, height);
        
        // Fill with a nice gradient color based on title hash
        let hash = title.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32));
        let hue = (hash % 360) as f32;
        let rgb = hsl_to_rgb(hue, 0.7, 0.6);
        
        for pixel in image.pixels_mut() {
            *pixel = Rgb([rgb.0, rgb.1, rgb.2]);
        }
        
        // Note: For text rendering, we would need a font library like rusttype
        // For now, we'll just use a colored rectangle
        
        Ok(DynamicImage::ImageRgb8(image))
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub covers_count: usize,
    pub thumbnails_count: usize,
    pub total_size_bytes: u64,
}

impl CacheStats {
    /// Get total size in MB
    pub fn total_size_mb(&self) -> f64 {
        self.total_size_bytes as f64 / (1024.0 * 1024.0)
    }
}

/// Convert HSL color to RGB
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let h = h / 360.0;
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    
    let (r, g, b) = if h < 1.0 / 6.0 {
        (c, x, 0.0)
    } else if h < 2.0 / 6.0 {
        (x, c, 0.0)
    } else if h < 3.0 / 6.0 {
        (0.0, c, x)
    } else if h < 4.0 / 6.0 {
        (0.0, x, c)
    } else if h < 5.0 / 6.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    
    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}