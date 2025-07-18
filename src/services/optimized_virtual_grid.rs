use std::collections::HashMap;
use std::sync::Arc;
use std::ops::Range;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use anyhow::Result;
use lru::LruCache;

use crate::models::book::Book;
use crate::models::library::ReadingStatus;

/// Grid virtual otimizado para alta performance
#[derive(Debug, Clone)]
pub struct OptimizedVirtualGrid {
    pub visible_range: Range<usize>,
    pub item_height: f32,
    pub items_per_row: usize,
    pub total_items: usize,
    pub viewport_height: f32,
    pub scroll_offset: f32,
    pub buffer_size: usize,
    
    // Otimizações
    pub last_update: Instant,
    pub update_threshold: Duration,
    pub batch_size: usize,
    pub enable_preloading: bool,
}

impl OptimizedVirtualGrid {
    pub fn new(
        total_items: usize,
        items_per_row: usize,
        item_height: f32,
        viewport_height: f32,
    ) -> Self {
        Self {
            visible_range: 0..0,
            item_height,
            items_per_row,
            total_items,
            viewport_height,
            scroll_offset: 0.0,
            buffer_size: 2,
            last_update: Instant::now(),
            update_threshold: Duration::from_millis(16), // 60 FPS
            batch_size: 10,
            enable_preloading: true,
        }
    }
    
    /// Atualiza o range visível de forma otimizada
    pub fn update_visible_range(&mut self, scroll_offset: f32) -> bool {
        // Throttle de atualizações para evitar overhead
        if self.last_update.elapsed() < self.update_threshold {
            return false;
        }
        
        let old_range = self.visible_range.clone();
        self.scroll_offset = scroll_offset;
        
        // Calcula as linhas visíveis
        let rows_per_viewport = (self.viewport_height / self.item_height).ceil() as usize;
        let start_row = (scroll_offset / self.item_height).floor() as usize;
        let end_row = (start_row + rows_per_viewport + self.buffer_size * 2).min(self.total_rows());
        
        // Buffer para scroll suave
        let buffer_start = start_row.saturating_sub(self.buffer_size);
        let buffer_end = (end_row + self.buffer_size).min(self.total_rows());
        
        // Calcula índices dos itens
        let start_index = buffer_start * self.items_per_row;
        let end_index = (buffer_end * self.items_per_row).min(self.total_items);
        
        self.visible_range = start_index..end_index;
        self.last_update = Instant::now();
        
        // Retorna true se houve mudança significativa
        old_range != self.visible_range
    }
    
    /// Obtém o número total de linhas
    pub fn total_rows(&self) -> usize {
        if self.total_items == 0 {
            0
        } else {
            (self.total_items + self.items_per_row - 1) / self.items_per_row
        }
    }
    
    /// Obtém a altura total do conteúdo
    pub fn total_content_height(&self) -> f32 {
        self.total_rows() as f32 * self.item_height
    }
    
    /// Verifica se um item está visível
    pub fn is_item_visible(&self, index: usize) -> bool {
        self.visible_range.contains(&index)
    }
    
    /// Obtém o range de preload baseado na direção do scroll
    pub fn get_preload_range(&self, scroll_direction: ScrollDirection) -> Range<usize> {
        let preload_size = self.batch_size;
        
        match scroll_direction {
            ScrollDirection::Down => {
                let start = self.visible_range.end;
                let end = (start + preload_size).min(self.total_items);
                start..end
            }
            ScrollDirection::Up => {
                let end = self.visible_range.start;
                let start = end.saturating_sub(preload_size);
                start..end
            }
            ScrollDirection::None => {
                // Preload em ambas as direções
                let up_start = self.visible_range.start.saturating_sub(preload_size / 2);
                let down_end = (self.visible_range.end + preload_size / 2).min(self.total_items);
                up_start..down_end
            }
        }
    }
    
    /// Obtém métricas de performance
    pub fn get_performance_metrics(&self) -> GridPerformanceMetrics {
        GridPerformanceMetrics {
            visible_items: self.visible_range.len(),
            total_items: self.total_items,
            buffer_efficiency: self.calculate_buffer_efficiency(),
            update_frequency: self.calculate_update_frequency(),
            memory_efficiency: self.calculate_memory_efficiency(),
        }
    }
    
    /// Calcula eficiência do buffer
    fn calculate_buffer_efficiency(&self) -> f32 {
        let viewport_items = (self.viewport_height / self.item_height).ceil() as usize * self.items_per_row;
        if viewport_items > 0 {
            viewport_items as f32 / self.visible_range.len() as f32
        } else {
            0.0
        }
    }
    
    /// Calcula frequência de atualização
    fn calculate_update_frequency(&self) -> f32 {
        let elapsed = self.last_update.elapsed();
        if elapsed.as_secs_f32() > 0.0 {
            1.0 / elapsed.as_secs_f32()
        } else {
            0.0
        }
    }
    
    /// Calcula eficiência de memória
    fn calculate_memory_efficiency(&self) -> f32 {
        if self.total_items > 0 {
            self.visible_range.len() as f32 / self.total_items as f32
        } else {
            0.0
        }
    }
    
    /// Otimiza configurações baseado no uso
    pub fn optimize_settings(&mut self, metrics: &GridPerformanceMetrics) {
        // Ajusta buffer size baseado na performance
        if metrics.update_frequency > 120.0 { // Muito rápido
            self.buffer_size = self.buffer_size.saturating_sub(1).max(1);
        } else if metrics.update_frequency < 30.0 { // Muito lento
            self.buffer_size = (self.buffer_size + 1).min(5);
        }
        
        // Ajusta threshold de atualização
        if metrics.buffer_efficiency < 0.3 { // Buffer muito grande
            self.update_threshold = Duration::from_millis(32); // 30 FPS
        } else if metrics.buffer_efficiency > 0.8 { // Buffer muito pequeno
            self.update_threshold = Duration::from_millis(8); // 120 FPS
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScrollDirection {
    Up,
    Down,
    None,
}

#[derive(Debug, Clone)]
pub struct GridPerformanceMetrics {
    pub visible_items: usize,
    pub total_items: usize,
    pub buffer_efficiency: f32,
    pub update_frequency: f32,
    pub memory_efficiency: f32,
}

/// Cache de imagens otimizado com LRU
pub struct OptimizedImageCache {
    cache: LruCache<String, CachedImage>,
    max_memory_mb: usize,
    current_memory_mb: usize,
    
    // Métricas
    cache_hits: u64,
    cache_misses: u64,
    evictions: u64,
    
    // Otimizações
    preload_queue: Vec<String>,
    loading_queue: HashMap<String, Instant>,
    priority_cache: HashMap<String, Priority>,
    
    // Configurações
    max_cache_size: usize,
    eviction_batch_size: usize,
    preload_threshold: f32,
}

#[derive(Debug, Clone)]
pub struct CachedImage {
    pub data: Vec<u8>,
    pub format: ImageFormat,
    pub width: u32,
    pub height: u32,
    pub memory_size: usize,
    pub last_accessed: Instant,
    pub access_count: u32,
    pub load_time: Duration,
    pub priority: Priority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Priority {
    Critical,   // Atualmente visível
    High,       // Próximo a ficar visível
    Medium,     // Buffer de scroll
    Low,        // Preload especulativo
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImageFormat {
    Jpeg,
    Png,
    Webp,
    Avif,
    Unknown,
}

impl OptimizedImageCache {
    pub fn new(max_memory_mb: usize) -> Self {
        let capacity = (max_memory_mb * 1024 * 1024) / 100000; // ~100KB por imagem
        
        Self {
            cache: LruCache::new(capacity.try_into().unwrap_or(1000)),
            max_memory_mb,
            current_memory_mb: 0,
            cache_hits: 0,
            cache_misses: 0,
            evictions: 0,
            preload_queue: Vec::new(),
            loading_queue: HashMap::new(),
            priority_cache: HashMap::new(),
            max_cache_size: capacity,
            eviction_batch_size: 10,
            preload_threshold: 0.8,
        }
    }
    
    /// Obtém imagem do cache
    pub fn get(&mut self, key: &str) -> Option<&CachedImage> {
        if let Some(image) = self.cache.get_mut(key) {
            image.last_accessed = Instant::now();
            image.access_count += 1;
            self.cache_hits += 1;
            Some(image)
        } else {
            self.cache_misses += 1;
            None
        }
    }
    
    /// Insere imagem no cache com gerenciamento inteligente
    pub fn insert(&mut self, key: String, mut image: CachedImage) {
        // Define prioridade baseada no contexto
        if let Some(priority) = self.priority_cache.get(&key) {
            image.priority = priority.clone();
        }
        
        // Verifica se precisa fazer eviction
        let required_memory = image.memory_size / 1024 / 1024;
        while self.current_memory_mb + required_memory > self.max_memory_mb {
            if !self.evict_least_important() {
                break; // Não conseguiu liberar mais memória
            }
        }
        
        self.current_memory_mb += required_memory;
        self.cache.put(key, image);
    }
    
    /// Remove item menos importante do cache
    fn evict_least_important(&mut self) -> bool {
        // Coleta candidatos para eviction
        let mut candidates: Vec<(String, Priority, Instant, u32)> = Vec::new();
        
        for (key, image) in self.cache.iter() {
            candidates.push((
                key.clone(),
                image.priority.clone(),
                image.last_accessed,
                image.access_count,
            ));
        }
        
        // Ordena por prioridade (menor primeiro) e tempo de acesso
        candidates.sort_by(|a, b| {
            match a.1.cmp(&b.1) {
                std::cmp::Ordering::Equal => a.2.cmp(&b.2), // Menos recente primeiro
                other => other.reverse(), // Menor prioridade primeiro
            }
        });
        
        // Remove o menos importante
        if let Some((key, _, _, _)) = candidates.first() {
            if let Some(evicted) = self.cache.pop(key) {
                self.current_memory_mb -= evicted.memory_size / 1024 / 1024;
                self.evictions += 1;
                return true;
            }
        }
        
        false
    }
    
    /// Define prioridade de uma imagem
    pub fn set_priority(&mut self, key: &str, priority: Priority) {
        self.priority_cache.insert(key.to_string(), priority);
    }
    
    /// Adiciona à fila de preload
    pub fn queue_preload(&mut self, urls: Vec<String>, priority: Priority) {
        for url in urls {
            if !self.cache.contains(&url) && !self.loading_queue.contains_key(&url) {
                self.set_priority(&url, priority);
                self.preload_queue.push(url);
            }
        }
        
        // Ordena fila por prioridade
        self.preload_queue.sort_by(|a, b| {
            let priority_a = self.priority_cache.get(a).unwrap_or(&Priority::Low);
            let priority_b = self.priority_cache.get(b).unwrap_or(&Priority::Low);
            priority_a.cmp(priority_b).reverse()
        });
    }
    
    /// Obtém próximo item para preload
    pub fn next_preload(&mut self) -> Option<String> {
        self.preload_queue.pop()
    }
    
    /// Marca como em carregamento
    pub fn mark_loading(&mut self, key: &str) {
        self.loading_queue.insert(key.to_string(), Instant::now());
    }
    
    /// Marca como carregamento concluído
    pub fn mark_loaded(&mut self, key: &str) {
        self.loading_queue.remove(key);
    }
    
    /// Verifica se está carregando
    pub fn is_loading(&self, key: &str) -> bool {
        self.loading_queue.contains_key(key)
    }
    
    /// Limpa carregamentos antigos
    pub fn cleanup_stale_loading(&mut self, timeout: Duration) {
        let now = Instant::now();
        self.loading_queue.retain(|_, start_time| {
            now.duration_since(*start_time) < timeout
        });
    }
    
    /// Obtém estatísticas do cache
    pub fn get_stats(&self) -> CacheStats {
        CacheStats {
            cache_hits: self.cache_hits,
            cache_misses: self.cache_misses,
            evictions: self.evictions,
            memory_usage_mb: self.current_memory_mb,
            cache_size: self.cache.len(),
            hit_rate: if self.cache_hits + self.cache_misses > 0 {
                self.cache_hits as f64 / (self.cache_hits + self.cache_misses) as f64
            } else {
                0.0
            },
            preload_queue_size: self.preload_queue.len(),
            loading_queue_size: self.loading_queue.len(),
        }
    }
    
    /// Otimiza cache baseado no uso
    pub fn optimize(&mut self, stats: &CacheStats) {
        // Ajusta tamanho do batch de eviction
        if stats.hit_rate < 0.7 {
            self.eviction_batch_size = (self.eviction_batch_size + 1).min(20);
        } else if stats.hit_rate > 0.9 {
            self.eviction_batch_size = self.eviction_batch_size.saturating_sub(1).max(1);
        }
        
        // Ajusta threshold de preload
        if stats.preload_queue_size > 100 {
            self.preload_threshold = (self.preload_threshold + 0.1).min(1.0);
        } else if stats.preload_queue_size < 10 {
            self.preload_threshold = (self.preload_threshold - 0.1).max(0.1);
        }
    }
    
    /// Limpa cache
    pub fn clear(&mut self) {
        self.cache.clear();
        self.current_memory_mb = 0;
        self.preload_queue.clear();
        self.loading_queue.clear();
        self.priority_cache.clear();
    }
}

impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Priority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Priority::Critical, Priority::Critical) => std::cmp::Ordering::Equal,
            (Priority::Critical, _) => std::cmp::Ordering::Greater,
            (_, Priority::Critical) => std::cmp::Ordering::Less,
            (Priority::High, Priority::High) => std::cmp::Ordering::Equal,
            (Priority::High, _) => std::cmp::Ordering::Greater,
            (_, Priority::High) => std::cmp::Ordering::Less,
            (Priority::Medium, Priority::Medium) => std::cmp::Ordering::Equal,
            (Priority::Medium, Priority::Low) => std::cmp::Ordering::Greater,
            (Priority::Low, Priority::Medium) => std::cmp::Ordering::Less,
            (Priority::Low, Priority::Low) => std::cmp::Ordering::Equal,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub evictions: u64,
    pub memory_usage_mb: usize,
    pub cache_size: usize,
    pub hit_rate: f64,
    pub preload_queue_size: usize,
    pub loading_queue_size: usize,
}

/// Serviço de biblioteca virtual otimizado
pub struct OptimizedLibraryService {
    virtual_grid: Arc<RwLock<OptimizedVirtualGrid>>,
    image_cache: Arc<RwLock<OptimizedImageCache>>,
    books: Arc<RwLock<Vec<Book>>>,
    filtered_books: Arc<RwLock<Vec<Book>>>,
    
    // Métricas de performance
    performance_metrics: Arc<RwLock<LibraryPerformanceMetrics>>,
    
    // Configurações
    settings: LibrarySettings,
}

#[derive(Debug, Clone)]
pub struct LibraryPerformanceMetrics {
    pub total_books: usize,
    pub visible_books: usize,
    pub filtered_books: usize,
    pub render_time_ms: u64,
    pub filter_time_ms: u64,
    pub scroll_fps: f32,
    pub memory_usage_mb: f32,
    pub cache_hit_rate: f32,
    pub last_update: Instant,
}

#[derive(Debug, Clone)]
pub struct LibrarySettings {
    pub max_cache_memory_mb: usize,
    pub items_per_row: usize,
    pub item_height: f32,
    pub enable_preloading: bool,
    pub preload_distance: usize,
    pub scroll_buffer_size: usize,
    pub performance_monitoring: bool,
}

impl Default for LibrarySettings {
    fn default() -> Self {
        Self {
            max_cache_memory_mb: 200,
            items_per_row: 6,
            item_height: 260.0,
            enable_preloading: true,
            preload_distance: 20,
            scroll_buffer_size: 2,
            performance_monitoring: true,
        }
    }
}

impl OptimizedLibraryService {
    pub fn new(settings: LibrarySettings) -> Self {
        Self {
            virtual_grid: Arc::new(RwLock::new(OptimizedVirtualGrid::new(
                0,
                settings.items_per_row,
                settings.item_height,
                600.0,
            ))),
            image_cache: Arc::new(RwLock::new(OptimizedImageCache::new(
                settings.max_cache_memory_mb,
            ))),
            books: Arc::new(RwLock::new(Vec::new())),
            filtered_books: Arc::new(RwLock::new(Vec::new())),
            performance_metrics: Arc::new(RwLock::new(LibraryPerformanceMetrics {
                total_books: 0,
                visible_books: 0,
                filtered_books: 0,
                render_time_ms: 0,
                filter_time_ms: 0,
                scroll_fps: 0.0,
                memory_usage_mb: 0.0,
                cache_hit_rate: 0.0,
                last_update: Instant::now(),
            })),
            settings,
        }
    }
    
    /// Inicializa o serviço com otimizações
    pub async fn initialize(&self, books: Vec<Book>) -> Result<()> {
        let start_time = Instant::now();
        
        // Armazena livros
        {
            let mut books_guard = self.books.write().await;
            *books_guard = books.clone();
        }
        
        // Inicializa livros filtrados
        {
            let mut filtered_guard = self.filtered_books.write().await;
            *filtered_guard = books.clone();
        }
        
        // Atualiza grid virtual
        {
            let mut grid = self.virtual_grid.write().await;
            grid.total_items = books.len();
            grid.update_visible_range(0.0);
        }
        
        // Atualiza métricas
        {
            let mut metrics = self.performance_metrics.write().await;
            metrics.total_books = books.len();
            metrics.filtered_books = books.len();
            metrics.last_update = Instant::now();
        }
        
        // Inicia preload inicial
        if self.settings.enable_preloading {
            self.preload_initial_covers().await?;
        }
        
        println!("📚 Library initialized with {} books in {:?}", 
                books.len(), start_time.elapsed());
        
        Ok(())
    }
    
    /// Preload inicial das capas mais importantes
    async fn preload_initial_covers(&self) -> Result<()> {
        let filtered_books = self.filtered_books.read().await;
        let preload_count = self.settings.preload_distance.min(filtered_books.len());
        
        let mut preload_urls = Vec::new();
        for i in 0..preload_count {
            if let Some(book) = filtered_books.get(i) {
                if let Some(cover_url) = &book.cover_url {
                    preload_urls.push(cover_url.clone());
                }
            }
        }
        
        if !preload_urls.is_empty() {
            let mut cache = self.image_cache.write().await;
            cache.queue_preload(preload_urls, Priority::High);
        }
        
        Ok(())
    }
    
    /// Atualiza viewport com otimizações
    pub async fn update_viewport(&self, scroll_offset: f32, viewport_height: f32) -> Result<()> {
        let start_time = Instant::now();
        
        // Atualiza grid virtual
        let (should_update, visible_range) = {
            let mut grid = self.virtual_grid.write().await;
            grid.viewport_height = viewport_height;
            let should_update = grid.update_visible_range(scroll_offset);
            (should_update, grid.visible_range.clone())
        };
        
        // Só continua se houve mudança significativa
        if !should_update {
            return Ok(());
        }
        
        // Preload baseado no scroll
        if self.settings.enable_preloading {
            self.preload_for_visible_range(&visible_range).await?;
        }
        
        // Atualiza métricas
        {
            let mut metrics = self.performance_metrics.write().await;
            metrics.visible_books = visible_range.len();
            metrics.render_time_ms = start_time.elapsed().as_millis() as u64;
            metrics.last_update = Instant::now();
        }
        
        Ok(())
    }
    
    /// Preload para o range visível
    async fn preload_for_visible_range(&self, visible_range: &Range<usize>) -> Result<()> {
        let filtered_books = self.filtered_books.read().await;
        let mut preload_urls = Vec::new();
        
        // Coleta URLs das capas visíveis
        for i in visible_range.clone() {
            if let Some(book) = filtered_books.get(i) {
                if let Some(cover_url) = &book.cover_url {
                    preload_urls.push(cover_url.clone());
                }
            }
        }
        
        // Adiciona URLs do buffer de preload
        let preload_start = visible_range.end;
        let preload_end = (preload_start + self.settings.preload_distance).min(filtered_books.len());
        
        for i in preload_start..preload_end {
            if let Some(book) = filtered_books.get(i) {
                if let Some(cover_url) = &book.cover_url {
                    preload_urls.push(cover_url.clone());
                }
            }
        }
        
        if !preload_urls.is_empty() {
            let mut cache = self.image_cache.write().await;
            cache.queue_preload(preload_urls, Priority::Medium);
        }
        
        Ok(())
    }
    
    /// Obtém livros visíveis de forma otimizada
    pub async fn get_visible_books(&self) -> Result<Vec<Book>> {
        let grid = self.virtual_grid.read().await;
        let filtered_books = self.filtered_books.read().await;
        
        let visible_books = filtered_books
            .iter()
            .enumerate()
            .filter(|(i, _)| grid.visible_range.contains(i))
            .map(|(_, book)| book.clone())
            .collect();
        
        Ok(visible_books)
    }
    
    /// Filtra livros com otimizações
    pub async fn filter_books(&self, query: &str, status_filter: Option<ReadingStatus>) -> Result<()> {
        let start_time = Instant::now();
        
        let books = self.books.read().await;
        let query_lower = query.to_lowercase();
        
        // Filtro otimizado
        let filtered: Vec<Book> = books
            .par_iter() // Processamento paralelo
            .filter(|book| {
                // Filtro de texto
                let matches_query = query.is_empty() || 
                    book.title.to_lowercase().contains(&query_lower) ||
                    book.author.to_lowercase().contains(&query_lower);
                
                // Filtro de status
                let matches_status = status_filter.is_none() || 
                    book.status == status_filter.unwrap();
                
                matches_query && matches_status
            })
            .cloned()
            .collect();
        
        let filter_time = start_time.elapsed();
        
        // Atualiza livros filtrados
        {
            let mut filtered_guard = self.filtered_books.write().await;
            *filtered_guard = filtered;
        }
        
        // Atualiza grid virtual
        {
            let mut grid = self.virtual_grid.write().await;
            grid.total_items = {
                let filtered_guard = self.filtered_books.read().await;
                filtered_guard.len()
            };
            grid.update_visible_range(grid.scroll_offset);
        }
        
        // Atualiza métricas
        {
            let mut metrics = self.performance_metrics.write().await;
            metrics.filtered_books = {
                let filtered_guard = self.filtered_books.read().await;
                filtered_guard.len()
            };
            metrics.filter_time_ms = filter_time.as_millis() as u64;
            metrics.last_update = Instant::now();
        }
        
        // Preload após filtro
        if self.settings.enable_preloading {
            self.preload_initial_covers().await?;
        }
        
        Ok(())
    }
    
    /// Obtém métricas de performance
    pub async fn get_performance_metrics(&self) -> LibraryPerformanceMetrics {
        let metrics = self.performance_metrics.read().await;
        let cache_stats = {
            let cache = self.image_cache.read().await;
            cache.get_stats()
        };
        
        LibraryPerformanceMetrics {
            total_books: metrics.total_books,
            visible_books: metrics.visible_books,
            filtered_books: metrics.filtered_books,
            render_time_ms: metrics.render_time_ms,
            filter_time_ms: metrics.filter_time_ms,
            scroll_fps: metrics.scroll_fps,
            memory_usage_mb: cache_stats.memory_usage_mb as f32,
            cache_hit_rate: cache_stats.hit_rate as f32,
            last_update: metrics.last_update,
        }
    }
    
    /// Otimiza configurações baseado na performance
    pub async fn optimize_performance(&self) -> Result<()> {
        let metrics = self.get_performance_metrics().await;
        
        // Otimiza grid virtual
        {
            let mut grid = self.virtual_grid.write().await;
            let grid_metrics = grid.get_performance_metrics();
            grid.optimize_settings(&grid_metrics);
        }
        
        // Otimiza cache de imagens
        {
            let mut cache = self.image_cache.write().await;
            let cache_stats = cache.get_stats();
            cache.optimize(&cache_stats);
        }
        
        println!("🔧 Performance optimized - FPS: {:.1}, Memory: {:.1}MB, Cache Hit: {:.1}%",
                metrics.scroll_fps, metrics.memory_usage_mb, metrics.cache_hit_rate * 100.0);
        
        Ok(())
    }
    
    /// Limpa recursos
    pub async fn cleanup(&self) {
        let mut cache = self.image_cache.write().await;
        cache.clear();
        
        let mut metrics = self.performance_metrics.write().await;
        metrics.last_update = Instant::now();
    }
}

// Adiciona rayon para processamento paralelo
use rayon::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_optimized_virtual_grid() {
        let mut grid = OptimizedVirtualGrid::new(1000, 6, 260.0, 600.0);
        
        // Testa cálculos básicos
        assert_eq!(grid.total_rows(), 167); // ceil(1000/6)
        assert_eq!(grid.total_content_height(), 167.0 * 260.0);
        
        // Testa atualização do range visível
        let updated = grid.update_visible_range(0.0);
        assert!(updated);
        assert_eq!(grid.visible_range.start, 0);
        assert!(grid.visible_range.end > 0);
        
        // Testa throttling
        let updated_again = grid.update_visible_range(10.0);
        assert!(!updated_again); // Deve ser throttled
    }
    
    #[test]
    fn test_optimized_image_cache() {
        let mut cache = OptimizedImageCache::new(1); // 1MB
        
        let image = CachedImage {
            data: vec![0u8; 500000], // 500KB
            format: ImageFormat::Jpeg,
            width: 150,
            height: 220,
            memory_size: 500000,
            last_accessed: Instant::now(),
            access_count: 0,
            load_time: Duration::from_millis(100),
            priority: Priority::High,
        };
        
        cache.insert("test1".to_string(), image);
        assert!(cache.get("test1").is_some());
        
        let stats = cache.get_stats();
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_size, 1);
    }
    
    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Critical > Priority::High);
        assert!(Priority::High > Priority::Medium);
        assert!(Priority::Medium > Priority::Low);
    }
    
    #[tokio::test]
    async fn test_optimized_library_service() {
        let settings = LibrarySettings::default();
        let service = OptimizedLibraryService::new(settings);
        
        // Cria livros de teste
        let mut books = Vec::new();
        for i in 0..100 {
            books.push(Book {
                id: format!("book_{}", i),
                title: format!("Test Book {}", i),
                author: format!("Author {}", i),
                cover_url: Some(format!("https://example.com/cover_{}.jpg", i)),
                status: ReadingStatus::Unread,
                ..Default::default()
            });
        }
        
        // Testa inicialização
        service.initialize(books).await.unwrap();
        
        // Testa atualização de viewport
        service.update_viewport(0.0, 600.0).await.unwrap();
        
        // Testa obtenção de livros visíveis
        let visible_books = service.get_visible_books().await.unwrap();
        assert!(!visible_books.is_empty());
        
        // Testa filtro
        service.filter_books("Test", None).await.unwrap();
        
        // Testa métricas
        let metrics = service.get_performance_metrics().await;
        assert_eq!(metrics.total_books, 100);
        
        // Testa otimização
        service.optimize_performance().await.unwrap();
    }
}