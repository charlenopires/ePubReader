#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::{Duration, Instant};
    use tokio::time::sleep;
    
    use crate::services::{OptimizedLibraryService, LibrarySettings};
    use crate::models::book::Book;
    use crate::models::library::BookStatus;
    
    /// Testa performance com bibliotecas grandes
    #[tokio::test]
    async fn test_large_library_performance() {
        let settings = LibrarySettings {
            max_cache_memory_mb: 100,
            items_per_row: 6,
            item_height: 260.0,
            enable_preloading: true,
            preload_distance: 20,
            scroll_buffer_size: 2,
            performance_monitoring: true,
        };
        
        let service = OptimizedLibraryService::new(settings);
        
        // Testa com diferentes tamanhos de biblioteca
        for size in [100, 500, 1000, 2000, 5000] {
            println!("\n🧪 Testing with {} books", size);
            
            let books = create_test_books(size);
            let start_time = Instant::now();
            
            // Testa inicialização
            service.initialize(books).await.unwrap();
            let init_time = start_time.elapsed();
            
            // Verifica targets de performance
            assert!(init_time < Duration::from_secs(if size <= 1000 { 2 } else { 5 }),
                    "Initialization took too long: {:?} for {} books", init_time, size);
            
            // Testa scroll performance
            let scroll_start = Instant::now();
            for i in 0..50 {
                service.update_viewport(i as f32 * 50.0, 600.0).await.unwrap();
            }
            let scroll_time = scroll_start.elapsed();
            
            // Verifica scroll performance (deve manter 30+ FPS)
            let avg_frame_time = scroll_time.as_millis() as f32 / 50.0;
            assert!(avg_frame_time < 33.0, // 30 FPS = 33ms per frame
                    "Scroll too slow: {:.1}ms per frame for {} books", avg_frame_time, size);
            
            // Testa filtro performance
            let filter_start = Instant::now();
            service.filter_books("Test", None).await.unwrap();
            let filter_time = filter_start.elapsed();
            
            assert!(filter_time < Duration::from_millis(500),
                    "Filter took too long: {:?} for {} books", filter_time, size);
            
            // Verifica métricas
            let metrics = service.get_performance_metrics().await;
            assert_eq!(metrics.total_books, size);
            assert!(metrics.memory_usage_mb < 150.0, 
                    "Memory usage too high: {:.1}MB", metrics.memory_usage_mb);
            
            println!("✅ Size: {} - Init: {:?}, Scroll: {:.1}ms/frame, Filter: {:?}, Memory: {:.1}MB",
                    size, init_time, avg_frame_time, filter_time, metrics.memory_usage_mb);
            
            service.cleanup().await;
        }
    }
    
    /// Testa performance do cache de imagens
    #[tokio::test]
    async fn test_image_cache_performance() {
        use crate::services::{OptimizedImageCache, CachedImage, ImageFormat, Priority};
        
        let mut cache = OptimizedImageCache::new(50); // 50MB
        
        // Testa inserção em lote
        let start_time = Instant::now();
        for i in 0..500 {
            let image = CachedImage {
                data: vec![0u8; 100000], // 100KB
                format: ImageFormat::Jpeg,
                width: 150,
                height: 220,
                memory_size: 100000,
                last_accessed: Instant::now(),
                access_count: 0,
                load_time: Duration::from_millis(50),
                priority: if i < 50 { Priority::Critical } else { Priority::Low },
            };
            
            cache.insert(format!("image_{}", i), image);
        }
        let insert_time = start_time.elapsed();
        
        // Deve ser rápido (< 100ms para 500 imagens)
        assert!(insert_time < Duration::from_millis(100),
                "Cache insertion too slow: {:?}", insert_time);
        
        // Testa recuperação
        let fetch_start = Instant::now();
        let mut hits = 0;
        for i in 0..500 {
            if cache.get(&format!("image_{}", i)).is_some() {
                hits += 1;
            }
        }
        let fetch_time = fetch_start.elapsed();
        
        // Deve ser muito rápido (< 10ms)
        assert!(fetch_time < Duration::from_millis(10),
                "Cache fetch too slow: {:?}", fetch_time);
        
        let stats = cache.get_stats();
        println!("📊 Cache stats: {} hits, {:.1}% hit rate, {}MB memory",
                stats.cache_hits, stats.hit_rate * 100.0, stats.memory_usage_mb);
        
        // Verifica hit rate
        assert!(stats.hit_rate > 0.0, "No cache hits");
        
        // Verifica que respeitou limite de memória
        assert!(stats.memory_usage_mb <= 50,
                "Cache exceeded memory limit: {}MB", stats.memory_usage_mb);
    }
    
    /// Testa performance do virtual scrolling
    #[tokio::test]
    async fn test_virtual_scrolling_performance() {
        use crate::services::OptimizedVirtualGrid;
        
        let mut grid = OptimizedVirtualGrid::new(10000, 6, 260.0, 600.0);
        
        // Testa atualizações rápidas de scroll
        let start_time = Instant::now();
        for i in 0..1000 {
            grid.update_visible_range(i as f32 * 10.0);
        }
        let update_time = start_time.elapsed();
        
        // Deve ser muito rápido (< 50ms para 1000 atualizações)
        assert!(update_time < Duration::from_millis(50),
                "Virtual grid updates too slow: {:?}", update_time);
        
        // Testa que apenas uma pequena fração dos itens está visível
        let metrics = grid.get_performance_metrics();
        let visibility_ratio = metrics.visible_items as f32 / metrics.total_items as f32;
        
        assert!(visibility_ratio < 0.1, // Menos de 10% dos itens devem estar visíveis
                "Too many items visible: {:.1}%", visibility_ratio * 100.0);
        
        println!("📊 Virtual grid: {}/{} items visible ({:.1}%)",
                metrics.visible_items, metrics.total_items, visibility_ratio * 100.0);
    }
    
    /// Testa performance sob carga
    #[tokio::test]
    async fn test_performance_under_load() {
        let settings = LibrarySettings::default();
        let service = OptimizedLibraryService::new(settings);
        
        // Inicializa com biblioteca grande
        let books = create_test_books(2000);
        service.initialize(books).await.unwrap();
        
        // Simula uso intensivo
        let load_start = Instant::now();
        
        // Múltiplas operações simultâneas
        let tasks = vec![
            // Scroll contínuo
            tokio::spawn({
                let service = service.clone();
                async move {
                    for i in 0..200 {
                        service.update_viewport(i as f32 * 25.0, 600.0).await.unwrap();
                        sleep(Duration::from_millis(5)).await;
                    }
                }
            }),
            
            // Filtros frequentes
            tokio::spawn({
                let service = service.clone();
                async move {
                    for query in ["Test", "Book", "Author", "Sample", ""] {
                        service.filter_books(query, None).await.unwrap();
                        sleep(Duration::from_millis(100)).await;
                    }
                }
            }),
            
            // Monitoramento de métricas
            tokio::spawn({
                let service = service.clone();
                async move {
                    for _ in 0..20 {
                        service.get_performance_metrics().await;
                        sleep(Duration::from_millis(50)).await;
                    }
                }
            }),
        ];
        
        // Aguarda todas as tasks
        for task in tasks {
            task.await.unwrap();
        }
        
        let load_time = load_start.elapsed();
        
        // Verifica que sobreviveu à carga
        assert!(load_time < Duration::from_secs(5),
                "Performance degraded under load: {:?}", load_time);
        
        // Verifica métricas finais
        let final_metrics = service.get_performance_metrics().await;
        assert!(final_metrics.memory_usage_mb < 200.0,
                "Memory usage too high after load: {:.1}MB", final_metrics.memory_usage_mb);
        
        println!("✅ Load test completed in {:?}, final memory: {:.1}MB",
                load_time, final_metrics.memory_usage_mb);
    }
    
    /// Testa targets específicos de performance
    #[tokio::test]
    async fn test_performance_targets() {
        let settings = LibrarySettings::default();
        let service = OptimizedLibraryService::new(settings);
        
        // Target: Inicialização < 2 segundos para 1000 livros
        let books = create_test_books(1000);
        let init_start = Instant::now();
        service.initialize(books).await.unwrap();
        let init_time = init_start.elapsed();
        
        assert!(init_time < Duration::from_secs(2),
                "❌ Initialization target missed: {:?} (target: 2s)", init_time);
        println!("✅ Initialization target met: {:?}", init_time);
        
        // Target: Scroll a 60 FPS (16.67ms por frame)
        let scroll_start = Instant::now();
        for i in 0..60 {
            service.update_viewport(i as f32 * 50.0, 600.0).await.unwrap();
        }
        let scroll_time = scroll_start.elapsed();
        let avg_frame_time = scroll_time.as_millis() as f32 / 60.0;
        
        assert!(avg_frame_time < 16.67,
                "❌ Scroll FPS target missed: {:.1}ms/frame (target: 16.67ms)", avg_frame_time);
        println!("✅ Scroll FPS target met: {:.1}ms/frame", avg_frame_time);
        
        // Target: Filtro < 500ms
        let filter_start = Instant::now();
        service.filter_books("Test Book", None).await.unwrap();
        let filter_time = filter_start.elapsed();
        
        assert!(filter_time < Duration::from_millis(500),
                "❌ Filter target missed: {:?} (target: 500ms)", filter_time);
        println!("✅ Filter target met: {:?}", filter_time);
        
        // Target: Uso de memória controlado
        let metrics = service.get_performance_metrics().await;
        assert!(metrics.memory_usage_mb < 200.0,
                "❌ Memory target missed: {:.1}MB (target: 200MB)", metrics.memory_usage_mb);
        println!("✅ Memory target met: {:.1}MB", metrics.memory_usage_mb);
    }
    
    /// Testa degradação graceful com bibliotecas enormes
    #[tokio::test]
    async fn test_graceful_degradation() {
        let settings = LibrarySettings::default();
        let service = OptimizedLibraryService::new(settings);
        
        // Testa com biblioteca muito grande
        let books = create_test_books(10000);
        let init_start = Instant::now();
        service.initialize(books).await.unwrap();
        let init_time = init_start.elapsed();
        
        // Deve ainda funcionar, mesmo que mais lento
        assert!(init_time < Duration::from_secs(15),
                "Failed to handle large library: {:?}", init_time);
        
        // Verifica que ainda mantém performance básica
        let scroll_start = Instant::now();
        for i in 0..10 {
            service.update_viewport(i as f32 * 100.0, 600.0).await.unwrap();
        }
        let scroll_time = scroll_start.elapsed();
        let avg_frame_time = scroll_time.as_millis() as f32 / 10.0;
        
        // Deve manter pelo menos 20 FPS
        assert!(avg_frame_time < 50.0,
                "Scroll too slow with large library: {:.1}ms/frame", avg_frame_time);
        
        println!("✅ Graceful degradation test passed: init {:?}, scroll {:.1}ms/frame",
                init_time, avg_frame_time);
    }
    
    /// Testa otimização automática
    #[tokio::test]
    async fn test_auto_optimization() {
        let settings = LibrarySettings::default();
        let service = OptimizedLibraryService::new(settings);
        
        let books = create_test_books(1000);
        service.initialize(books).await.unwrap();
        
        // Obtém métricas iniciais
        let initial_metrics = service.get_performance_metrics().await;
        
        // Simula carga que pode degradar performance
        for i in 0..100 {
            service.update_viewport(i as f32 * 10.0, 600.0).await.unwrap();
            service.filter_books(&format!("Query {}", i % 10), None).await.unwrap();
        }
        
        // Executa otimização
        service.optimize_performance().await.unwrap();
        
        // Verifica que otimização foi aplicada
        let optimized_metrics = service.get_performance_metrics().await;
        
        // Deve manter ou melhorar performance
        assert!(optimized_metrics.memory_usage_mb <= initial_metrics.memory_usage_mb * 1.1,
                "Memory usage increased after optimization: {:.1}MB -> {:.1}MB",
                initial_metrics.memory_usage_mb, optimized_metrics.memory_usage_mb);
        
        println!("✅ Auto-optimization test passed: {:.1}MB -> {:.1}MB",
                initial_metrics.memory_usage_mb, optimized_metrics.memory_usage_mb);
    }
    
    /// Função auxiliar para criar livros de teste
    fn create_test_books(count: usize) -> Vec<Book> {
        let mut books = Vec::with_capacity(count);
        
        for i in 0..count {
            books.push(Book {
                id: format!("book_{}", i),
                title: format!("Test Book {}", i + 1),
                author: format!("Author {}", (i % 100) + 1),
                cover_url: Some(format!("https://picsum.photos/150/220?random={}", i)),
                status: match i % 4 {
                    0 => BookStatus::Unread,
                    1 => BookStatus::CurrentlyReading,
                    2 => BookStatus::Finished,
                    _ => BookStatus::Unread,
                },
                file_path: format!("/path/to/book_{}.epub", i),
                file_size: 1024 * 1024 * (1 + i % 10), // 1-10MB
                page_count: 100 + (i % 400), // 100-500 pages
                word_count: 10000 + (i % 90000), // 10k-100k words
                language: "en".to_string(),
                isbn: format!("978-{:010}", i),
                publisher: format!("Publisher {}", (i % 20) + 1),
                publication_date: chrono::Utc::now() - chrono::Duration::days((i % 365) as i64),
                added_date: chrono::Utc::now() - chrono::Duration::days((i % 30) as i64),
                last_read_date: if i % 3 == 0 { 
                    Some(chrono::Utc::now() - chrono::Duration::days((i % 7) as i64))
                } else { 
                    None 
                },
                reading_progress: if i % 4 == 1 { (i % 100) as f32 / 100.0 } else { 0.0 },
                tags: vec![format!("tag_{}", i % 10), format!("genre_{}", i % 5)],
                rating: if i % 5 == 0 { Some((i % 5 + 1) as u8) } else { None },
                notes: if i % 7 == 0 { Some(format!("Notes for book {}", i)) } else { None },
                is_favorite: i % 10 == 0,
                series: if i % 3 == 0 { Some(format!("Series {}", i % 20)) } else { None },
                series_index: if i % 3 == 0 { Some(((i % 10) + 1) as f32) } else { None },
                description: Some(format!("Description for test book {}. This is a sample book created for testing purposes.", i)),
            });
        }
        
        books
    }
    
    /// Benchmark de performance
    #[tokio::test]
    async fn benchmark_performance() {
        println!("\n🏁 Performance Benchmark");
        println!("========================");
        
        let settings = LibrarySettings::default();
        let service = OptimizedLibraryService::new(settings);
        
        // Benchmark com diferentes tamanhos
        for size in [500, 1000, 2000, 5000] {
            let books = create_test_books(size);
            
            // Benchmark inicialização
            let start = Instant::now();
            service.initialize(books).await.unwrap();
            let init_time = start.elapsed();
            
            // Benchmark scroll
            let start = Instant::now();
            for i in 0..60 {
                service.update_viewport(i as f32 * 50.0, 600.0).await.unwrap();
            }
            let scroll_time = start.elapsed();
            let fps = 60.0 / scroll_time.as_secs_f32();
            
            // Benchmark filtro
            let start = Instant::now();
            service.filter_books("Test", None).await.unwrap();
            let filter_time = start.elapsed();
            
            // Métricas finais
            let metrics = service.get_performance_metrics().await;
            
            println!("{:5} books: init {:6.1}ms, scroll {:4.1} FPS, filter {:5.1}ms, memory {:5.1}MB",
                    size, 
                    init_time.as_millis(),
                    fps,
                    filter_time.as_millis(),
                    metrics.memory_usage_mb);
            
            service.cleanup().await;
        }
        
        println!("========================");
    }
}