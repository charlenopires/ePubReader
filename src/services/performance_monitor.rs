use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;
use tokio::time::interval;
use serde::{Deserialize, Serialize};

/// Performance monitoring service for tracking app performance
pub struct PerformanceMonitor {
    metrics: Arc<RwLock<PerformanceMetrics>>,
    frame_times: Arc<RwLock<VecDeque<Duration>>>,
    memory_samples: Arc<RwLock<VecDeque<MemorySample>>>,
    is_monitoring: Arc<RwLock<bool>>,
    targets: PerformanceTargets,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    // Rendering metrics
    pub avg_frame_time_ms: f64,
    pub current_fps: f64,
    pub target_fps: f64,
    pub frame_drops: u64,
    pub render_time_ms: u64,
    
    // Loading metrics
    pub startup_time_ms: u64,
    pub book_open_time_ms: u64,
    pub search_time_ms: u64,
    pub image_load_time_ms: u64,
    
    // Memory metrics
    pub memory_usage_mb: f64,
    pub peak_memory_mb: f64,
    pub cache_hit_rate: f64,
    pub cache_memory_mb: f64,
    
    // UI responsiveness
    pub input_lag_ms: u64,
    pub scroll_performance: f64,
    pub animation_smoothness: f64,
    
    // Network metrics
    pub network_requests: u64,
    pub network_errors: u64,
    pub avg_download_speed_mbps: f64,
    
    // Library metrics
    pub total_books: usize,
    pub visible_books: usize,
    pub books_per_second: f64,
    
    // Timestamps
    pub measurement_start: SystemTime,
    pub last_update: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTargets {
    pub startup_time_ms: u64,
    pub book_open_time_ms: u64,
    pub search_time_ms: u64,
    pub target_fps: f64,
    pub max_memory_mb: f64,
    pub min_cache_hit_rate: f64,
}

impl Default for PerformanceTargets {
    fn default() -> Self {
        Self {
            startup_time_ms: 2000,    // < 2 seconds
            book_open_time_ms: 1000,  // < 1 second
            search_time_ms: 500,      // < 500ms for 500 pages
            target_fps: 60.0,         // 60 FPS
            max_memory_mb: 500.0,     // 500MB max
            min_cache_hit_rate: 0.8,  // 80% cache hit rate
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySample {
    pub timestamp: SystemTime,
    pub heap_mb: f64,
    pub cache_mb: f64,
    pub total_mb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    pub alert_type: AlertType,
    pub message: String,
    pub severity: AlertSeverity,
    pub timestamp: SystemTime,
    pub value: f64,
    pub threshold: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AlertType {
    HighMemoryUsage,
    LowFrameRate,
    SlowStartup,
    SlowBookOpen,
    SlowSearch,
    LowCacheHitRate,
    NetworkTimeout,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

impl PerformanceMonitor {
    pub fn new(targets: Option<PerformanceTargets>) -> Self {
        let now = SystemTime::now();
        
        Self {
            metrics: Arc::new(RwLock::new(PerformanceMetrics {
                avg_frame_time_ms: 0.0,
                current_fps: 0.0,
                target_fps: 60.0,
                frame_drops: 0,
                render_time_ms: 0,
                startup_time_ms: 0,
                book_open_time_ms: 0,
                search_time_ms: 0,
                image_load_time_ms: 0,
                memory_usage_mb: 0.0,
                peak_memory_mb: 0.0,
                cache_hit_rate: 0.0,
                cache_memory_mb: 0.0,
                input_lag_ms: 0,
                scroll_performance: 0.0,
                animation_smoothness: 0.0,
                network_requests: 0,
                network_errors: 0,
                avg_download_speed_mbps: 0.0,
                total_books: 0,
                visible_books: 0,
                books_per_second: 0.0,
                measurement_start: now,
                last_update: now,
            })),
            frame_times: Arc::new(RwLock::new(VecDeque::with_capacity(120))), // 2 seconds at 60fps
            memory_samples: Arc::new(RwLock::new(VecDeque::with_capacity(300))), // 5 minutes at 1 sample/second
            is_monitoring: Arc::new(RwLock::new(false)),
            targets: targets.unwrap_or_default(),
        }
    }
    
    /// Start performance monitoring
    pub async fn start_monitoring(&self) {
        let mut is_monitoring = self.is_monitoring.write().await;
        if *is_monitoring {
            return;
        }
        *is_monitoring = true;
        drop(is_monitoring);
        
        // Start background monitoring task
        let metrics = self.metrics.clone();
        let memory_samples = self.memory_samples.clone();
        let is_monitoring = self.is_monitoring.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(1000)); // 1 second intervals
            
            while *is_monitoring.read().await {
                interval.tick().await;
                
                // Sample memory usage
                let memory_usage = Self::get_memory_usage().await;
                let sample = MemorySample {
                    timestamp: SystemTime::now(),
                    heap_mb: memory_usage.heap_mb,
                    cache_mb: memory_usage.cache_mb,
                    total_mb: memory_usage.total_mb,
                };
                
                {
                    let mut samples = memory_samples.write().await;
                    samples.push_back(sample);
                    if samples.len() > 300 {
                        samples.pop_front();
                    }
                }
                
                // Update metrics
                {
                    let mut metrics_guard = metrics.write().await;
                    metrics_guard.memory_usage_mb = memory_usage.total_mb;
                    metrics_guard.cache_memory_mb = memory_usage.cache_mb;
                    if memory_usage.total_mb > metrics_guard.peak_memory_mb {
                        metrics_guard.peak_memory_mb = memory_usage.total_mb;
                    }
                    metrics_guard.last_update = SystemTime::now();
                }
            }
        });
    }
    
    /// Stop performance monitoring
    pub async fn stop_monitoring(&self) {
        let mut is_monitoring = self.is_monitoring.write().await;
        *is_monitoring = false;
    }
    
    /// Record frame rendering time
    pub async fn record_frame_time(&self, frame_time: Duration) {
        let frame_time_ms = frame_time.as_millis() as f64;
        
        // Update frame times buffer
        {
            let mut frame_times = self.frame_times.write().await;
            frame_times.push_back(frame_time);
            if frame_times.len() > 120 {
                frame_times.pop_front();
            }
        }
        
        // Calculate FPS and averages
        let frame_times = self.frame_times.read().await;
        let avg_frame_time = if frame_times.is_empty() {
            0.0
        } else {
            frame_times.iter().map(|t| t.as_millis() as f64).sum::<f64>() / frame_times.len() as f64
        };
        
        let current_fps = if avg_frame_time > 0.0 {
            1000.0 / avg_frame_time
        } else {
            0.0
        };
        
        drop(frame_times);
        
        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.avg_frame_time_ms = avg_frame_time;
            metrics.current_fps = current_fps;
            metrics.render_time_ms = frame_time_ms as u64;
            
            // Count frame drops (frames slower than 16.67ms for 60fps)
            if frame_time_ms > 16.67 {
                metrics.frame_drops += 1;
            }
            
            metrics.last_update = SystemTime::now();
        }
    }
    
    /// Record startup time
    pub async fn record_startup_time(&self, startup_time: Duration) {
        let mut metrics = self.metrics.write().await;
        metrics.startup_time_ms = startup_time.as_millis() as u64;
        metrics.last_update = SystemTime::now();
    }
    
    /// Record book open time
    pub async fn record_book_open_time(&self, open_time: Duration) {
        let mut metrics = self.metrics.write().await;
        metrics.book_open_time_ms = open_time.as_millis() as u64;
        metrics.last_update = SystemTime::now();
    }
    
    /// Record search time
    pub async fn record_search_time(&self, search_time: Duration) {
        let mut metrics = self.metrics.write().await;
        metrics.search_time_ms = search_time.as_millis() as u64;
        metrics.last_update = SystemTime::now();
    }
    
    /// Record image load time
    pub async fn record_image_load_time(&self, load_time: Duration) {
        let mut metrics = self.metrics.write().await;
        metrics.image_load_time_ms = load_time.as_millis() as u64;
        metrics.last_update = SystemTime::now();
    }
    
    /// Update cache statistics
    pub async fn update_cache_stats(&self, hit_rate: f64, cache_memory_mb: f64) {
        let mut metrics = self.metrics.write().await;
        metrics.cache_hit_rate = hit_rate;
        metrics.cache_memory_mb = cache_memory_mb;
        metrics.last_update = SystemTime::now();
    }
    
    /// Update network statistics
    pub async fn update_network_stats(&self, requests: u64, errors: u64, avg_speed_mbps: f64) {
        let mut metrics = self.metrics.write().await;
        metrics.network_requests = requests;
        metrics.network_errors = errors;
        metrics.avg_download_speed_mbps = avg_speed_mbps;
        metrics.last_update = SystemTime::now();
    }
    
    /// Update library statistics
    pub async fn update_library_stats(&self, total_books: usize, visible_books: usize) {
        let mut metrics = self.metrics.write().await;
        metrics.total_books = total_books;
        metrics.visible_books = visible_books;
        
        // Calculate books per second (rendering rate)
        if let Ok(elapsed) = metrics.last_update.elapsed() {
            if elapsed.as_secs_f64() > 0.0 {
                metrics.books_per_second = visible_books as f64 / elapsed.as_secs_f64();
            }
        }
        
        metrics.last_update = SystemTime::now();
    }
    
    /// Update input lag
    pub async fn update_input_lag(&self, lag: Duration) {
        let mut metrics = self.metrics.write().await;
        metrics.input_lag_ms = lag.as_millis() as u64;
        metrics.last_update = SystemTime::now();
    }
    
    /// Update scroll performance
    pub async fn update_scroll_performance(&self, performance: f64) {
        let mut metrics = self.metrics.write().await;
        metrics.scroll_performance = performance;
        metrics.last_update = SystemTime::now();
    }
    
    /// Update animation smoothness
    pub async fn update_animation_smoothness(&self, smoothness: f64) {
        let mut metrics = self.metrics.write().await;
        metrics.animation_smoothness = smoothness;
        metrics.last_update = SystemTime::now();
    }
    
    /// Get current metrics
    pub async fn get_metrics(&self) -> PerformanceMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }
    
    /// Check if performance targets are met
    pub async fn check_targets(&self) -> Vec<PerformanceAlert> {
        let metrics = self.metrics.read().await;
        let mut alerts = Vec::new();
        
        // Check startup time
        if metrics.startup_time_ms > self.targets.startup_time_ms {
            alerts.push(PerformanceAlert {
                alert_type: AlertType::SlowStartup,
                message: format!("Startup time {}ms exceeds target {}ms", 
                               metrics.startup_time_ms, self.targets.startup_time_ms),
                severity: AlertSeverity::Warning,
                timestamp: SystemTime::now(),
                value: metrics.startup_time_ms as f64,
                threshold: self.targets.startup_time_ms as f64,
            });
        }
        
        // Check book open time
        if metrics.book_open_time_ms > self.targets.book_open_time_ms {
            alerts.push(PerformanceAlert {
                alert_type: AlertType::SlowBookOpen,
                message: format!("Book open time {}ms exceeds target {}ms", 
                               metrics.book_open_time_ms, self.targets.book_open_time_ms),
                severity: AlertSeverity::Warning,
                timestamp: SystemTime::now(),
                value: metrics.book_open_time_ms as f64,
                threshold: self.targets.book_open_time_ms as f64,
            });
        }
        
        // Check search time
        if metrics.search_time_ms > self.targets.search_time_ms {
            alerts.push(PerformanceAlert {
                alert_type: AlertType::SlowSearch,
                message: format!("Search time {}ms exceeds target {}ms", 
                               metrics.search_time_ms, self.targets.search_time_ms),
                severity: AlertSeverity::Warning,
                timestamp: SystemTime::now(),
                value: metrics.search_time_ms as f64,
                threshold: self.targets.search_time_ms as f64,
            });
        }
        
        // Check frame rate
        if metrics.current_fps < self.targets.target_fps * 0.8 {
            let severity = if metrics.current_fps < self.targets.target_fps * 0.5 {
                AlertSeverity::Critical
            } else {
                AlertSeverity::Warning
            };
            
            alerts.push(PerformanceAlert {
                alert_type: AlertType::LowFrameRate,
                message: format!("Frame rate {:.1} FPS below target {:.1} FPS", 
                               metrics.current_fps, self.targets.target_fps),
                severity,
                timestamp: SystemTime::now(),
                value: metrics.current_fps,
                threshold: self.targets.target_fps,
            });
        }
        
        // Check memory usage
        if metrics.memory_usage_mb > self.targets.max_memory_mb {
            let severity = if metrics.memory_usage_mb > self.targets.max_memory_mb * 1.5 {
                AlertSeverity::Critical
            } else {
                AlertSeverity::Warning
            };
            
            alerts.push(PerformanceAlert {
                alert_type: AlertType::HighMemoryUsage,
                message: format!("Memory usage {:.1}MB exceeds target {:.1}MB", 
                               metrics.memory_usage_mb, self.targets.max_memory_mb),
                severity,
                timestamp: SystemTime::now(),
                value: metrics.memory_usage_mb,
                threshold: self.targets.max_memory_mb,
            });
        }
        
        // Check cache hit rate
        if metrics.cache_hit_rate < self.targets.min_cache_hit_rate {
            alerts.push(PerformanceAlert {
                alert_type: AlertType::LowCacheHitRate,
                message: format!("Cache hit rate {:.1}% below target {:.1}%", 
                               metrics.cache_hit_rate * 100.0, self.targets.min_cache_hit_rate * 100.0),
                severity: AlertSeverity::Info,
                timestamp: SystemTime::now(),
                value: metrics.cache_hit_rate,
                threshold: self.targets.min_cache_hit_rate,
            });
        }
        
        alerts
    }
    
    /// Generate performance report
    pub async fn generate_report(&self) -> PerformanceReport {
        let metrics = self.metrics.read().await;
        let alerts = self.check_targets().await;
        
        let overall_score = self.calculate_performance_score(&metrics).await;
        let recommendations = self.generate_recommendations(&metrics, &alerts).await;
        
        PerformanceReport {
            metrics: metrics.clone(),
            alerts,
            overall_score,
            recommendations,
            generated_at: SystemTime::now(),
        }
    }
    
    /// Calculate overall performance score (0-100)
    async fn calculate_performance_score(&self, metrics: &PerformanceMetrics) -> f64 {
        let mut score: f64 = 100.0;
        
        // Deduct points for performance issues
        if metrics.startup_time_ms > self.targets.startup_time_ms {
            score -= 10.0;
        }
        
        if metrics.book_open_time_ms > self.targets.book_open_time_ms {
            score -= 10.0;
        }
        
        if metrics.search_time_ms > self.targets.search_time_ms {
            score -= 10.0;
        }
        
        if metrics.current_fps < self.targets.target_fps * 0.8 {
            score -= 20.0;
        }
        
        if metrics.memory_usage_mb > self.targets.max_memory_mb {
            score -= 15.0;
        }
        
        if metrics.cache_hit_rate < self.targets.min_cache_hit_rate {
            score -= 10.0;
        }
        
        // Bonus points for excellent performance
        if metrics.current_fps > self.targets.target_fps * 1.1 {
            score += 5.0;
        }
        
        if metrics.cache_hit_rate > 0.95 {
            score += 5.0;
        }
        
        score.max(0.0).min(100.0)
    }
    
    /// Generate performance recommendations
    async fn generate_recommendations(&self, metrics: &PerformanceMetrics, alerts: &[PerformanceAlert]) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        for alert in alerts {
            match alert.alert_type {
                AlertType::SlowStartup => {
                    recommendations.push("Consider lazy loading components and caching initialization data".to_string());
                }
                AlertType::SlowBookOpen => {
                    recommendations.push("Optimize book parsing and implement progressive loading".to_string());
                }
                AlertType::SlowSearch => {
                    recommendations.push("Implement search indexing and result caching".to_string());
                }
                AlertType::LowFrameRate => {
                    recommendations.push("Reduce rendering complexity and implement virtual scrolling".to_string());
                }
                AlertType::HighMemoryUsage => {
                    recommendations.push("Implement LRU cache eviction and optimize image loading".to_string());
                }
                AlertType::LowCacheHitRate => {
                    recommendations.push("Improve cache preloading strategies and increase cache size".to_string());
                }
                _ => {}
            }
        }
        
        // General recommendations
        if metrics.frame_drops > 10 {
            recommendations.push("Consider reducing animation complexity during scrolling".to_string());
        }
        
        if metrics.total_books > 1000 && metrics.visible_books > 50 {
            recommendations.push("Implement virtual scrolling for large libraries".to_string());
        }
        
        recommendations
    }
    
    /// Get memory usage (simplified - would use actual memory profiling)
    async fn get_memory_usage() -> MemoryUsage {
        // In a real implementation, this would use system APIs or profiling tools
        MemoryUsage {
            heap_mb: 50.0,
            cache_mb: 25.0,
            total_mb: 75.0,
        }
    }
    
    /// Export metrics to JSON
    pub async fn export_metrics(&self) -> Result<String, serde_json::Error> {
        let metrics = self.metrics.read().await;
        serde_json::to_string_pretty(&*metrics)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub metrics: PerformanceMetrics,
    pub alerts: Vec<PerformanceAlert>,
    pub overall_score: f64,
    pub recommendations: Vec<String>,
    pub generated_at: SystemTime,
}

#[derive(Debug, Clone)]
struct MemoryUsage {
    pub heap_mb: f64,
    pub cache_mb: f64,
    pub total_mb: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_performance_monitor() {
        let monitor = PerformanceMonitor::new(None);
        
        // Start monitoring
        monitor.start_monitoring().await;
        
        // Record some metrics
        monitor.record_frame_time(Duration::from_millis(16)).await;
        monitor.record_startup_time(Duration::from_millis(1500)).await;
        monitor.record_book_open_time(Duration::from_millis(800)).await;
        
        // Get metrics
        let metrics = monitor.get_metrics().await;
        assert_eq!(metrics.startup_time_ms, 1500);
        assert_eq!(metrics.book_open_time_ms, 800);
        assert!(metrics.current_fps > 0.0);
        
        // Check targets
        let alerts = monitor.check_targets().await;
        assert!(!alerts.is_empty() || metrics.startup_time_ms <= 2000);
        
        // Generate report
        let report = monitor.generate_report().await;
        assert!(report.overall_score >= 0.0);
        assert!(report.overall_score <= 100.0);
        
        // Stop monitoring
        monitor.stop_monitoring().await;
    }
    
    #[tokio::test]
    async fn test_performance_targets() {
        let custom_targets = PerformanceTargets {
            startup_time_ms: 1000,
            book_open_time_ms: 500,
            search_time_ms: 250,
            target_fps: 60.0,
            max_memory_mb: 300.0,
            min_cache_hit_rate: 0.9,
        };
        
        let monitor = PerformanceMonitor::new(Some(custom_targets));
        
        // Record metrics that exceed targets
        monitor.record_startup_time(Duration::from_millis(1500)).await;
        monitor.record_book_open_time(Duration::from_millis(800)).await;
        
        let alerts = monitor.check_targets().await;
        assert!(!alerts.is_empty());
        
        // Check alert types
        let startup_alerts: Vec<_> = alerts.iter().filter(|a| a.alert_type == AlertType::SlowStartup).collect();
        assert!(!startup_alerts.is_empty());
    }
    
    #[test]
    fn test_alert_severity() {
        let alert = PerformanceAlert {
            alert_type: AlertType::HighMemoryUsage,
            message: "High memory usage".to_string(),
            severity: AlertSeverity::Critical,
            timestamp: SystemTime::now(),
            value: 800.0,
            threshold: 500.0,
        };
        
        assert_eq!(alert.severity, AlertSeverity::Critical);
        assert_eq!(alert.alert_type, AlertType::HighMemoryUsage);
    }
}