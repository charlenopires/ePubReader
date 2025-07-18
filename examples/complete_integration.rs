/*!
 * Exemplo completo de integração do Virtual Grid Otimizado
 * 
 * Este exemplo demonstra como usar todos os componentes de performance
 * em uma aplicação real de ePub Reader.
 */

use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use tokio::sync::RwLock;
use slint::{ModelRc, VecModel, SharedString};

// Imports dos serviços
use crate::services::{
    OptimizedLibraryService, 
    LibrarySettings,
    OptimizedImageCache,
    AsyncImageLoader,
    PerformanceMonitor,
    LoadPriority,
    Priority,
    ScrollDirection,
};

use crate::models::book::Book;
use crate::models::library::BookStatus;

// Estruturas para interface Slint
#[derive(Clone)]
pub struct SlintBookItem {
    pub id: SharedString,
    pub title: SharedString,
    pub author: SharedString,
    pub cover_url: SharedString,
    pub is_cover_loaded: bool,
    pub reading_progress: f32,
    pub status: SharedString,
    pub last_read: SharedString,
    pub rating: i32,
    pub book_size_mb: f32,
    pub page_count: i32,
}

#[derive(Clone)]
pub struct SlintPerformanceMetrics {
    pub visible_items: i32,
    pub total_items: i32,
    pub render_time_ms: i32,
    pub scroll_fps: f32,
    pub memory_usage_mb: f32,
    pub cache_hit_rate: f32,
}

/// Controller principal da aplicação
pub struct EpubReaderController {
    // Serviços principais
    library_service: Arc<OptimizedLibraryService>,
    image_loader: Arc<AsyncImageLoader>,
    performance_monitor: Arc<PerformanceMonitor>,
    
    // Estado da aplicação
    app_state: Arc<RwLock<AppState>>,
    
    // Interface Slint
    ui_handle: slint::Weak<MainWindow>,
    
    // Configurações
    settings: LibrarySettings,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub current_scroll_offset: f32,
    pub visible_range: std::ops::Range<usize>,
    pub selected_book_id: Option<String>,
    pub filter_query: String,
    pub filter_status: Option<BookStatus>,
    pub is_scrolling: bool,
    pub last_scroll_time: Instant,
    pub performance_mode: PerformanceMode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PerformanceMode {
    HighPerformance,  // Máxima performance
    Balanced,         // Balanceado
    PowerSaving,      // Economia de energia
}

impl EpubReaderController {
    /// Cria novo controller
    pub fn new(ui_handle: slint::Weak<MainWindow>) -> Self {
        let settings = LibrarySettings::default();
        
        Self {
            library_service: Arc::new(OptimizedLibraryService::new(settings.clone())),
            image_loader: Arc::new(AsyncImageLoader::new(8, 30)),
            performance_monitor: Arc::new(PerformanceMonitor::new(None)),
            app_state: Arc::new(RwLock::new(AppState {
                current_scroll_offset: 0.0,
                visible_range: 0..0,
                selected_book_id: None,
                filter_query: String::new(),
                filter_status: None,
                is_scrolling: false,
                last_scroll_time: Instant::now(),
                performance_mode: PerformanceMode::Balanced,
            })),
            ui_handle,
            settings,
        }
    }
    
    /// Inicializa o controller
    pub async fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Inicializando ePub Reader Controller...");
        
        // Inicia monitoramento de performance
        self.performance_monitor.start_monitoring().await;
        
        // Carrega livros
        let books = self.load_books_from_storage().await?;
        self.library_service.initialize(books).await?;
        
        // Configura callbacks da UI
        self.setup_ui_callbacks().await?;
        
        // Inicia tarefas em background
        self.start_background_tasks().await;
        
        // Atualiza UI inicial
        self.update_ui().await?;
        
        println!("✅ Controller inicializado com sucesso!");
        Ok(())
    }
    
    /// Configura callbacks da UI
    async fn setup_ui_callbacks(&self) -> Result<(), Box<dyn std::error::Error>> {
        let ui = self.ui_handle.upgrade().ok_or("UI handle lost")?;
        
        // Callback de scroll
        let controller = self.clone();
        ui.on_viewport_changed(move |offset, start, end| {
            let controller = controller.clone();
            tokio::spawn(async move {
                if let Err(e) = controller.handle_viewport_changed(offset, start, end).await {
                    eprintln!("Erro no viewport: {}", e);
                }
            });
        });
        
        // Callback de seleção de livro
        let controller = self.clone();
        ui.on_book_selected(move |book_id| {
            let controller = controller.clone();
            tokio::spawn(async move {
                if let Err(e) = controller.handle_book_selected(book_id.to_string()).await {
                    eprintln!("Erro na seleção: {}", e);
                }
            });
        });
        
        // Callback de abertura de livro
        let controller = self.clone();
        ui.on_book_opened(move |book_id| {
            let controller = controller.clone();
            tokio::spawn(async move {
                if let Err(e) = controller.handle_book_opened(book_id.to_string()).await {
                    eprintln!("Erro na abertura: {}", e);
                }
            });
        });
        
        // Callback de preload
        let controller = self.clone();
        ui.on_preload_request(move |start, end| {
            let controller = controller.clone();
            tokio::spawn(async move {
                if let Err(e) = controller.handle_preload_request(start, end).await {
                    eprintln!("Erro no preload: {}", e);
                }
            });
        });
        
        // Callback de filtro
        let controller = self.clone();
        ui.on_filter_changed(move |query, status| {
            let controller = controller.clone();
            tokio::spawn(async move {
                if let Err(e) = controller.handle_filter_changed(query.to_string(), status.to_string()).await {
                    eprintln!("Erro no filtro: {}", e);
                }
            });
        });
        
        // Callback de configuração
        let controller = self.clone();
        ui.on_settings_changed(move |items_per_row, performance_mode| {
            let controller = controller.clone();
            tokio::spawn(async move {
                if let Err(e) = controller.handle_settings_changed(items_per_row, performance_mode.to_string()).await {
                    eprintln!("Erro nas configurações: {}", e);
                }
            });
        });
        
        Ok(())
    }
    
    /// Inicia tarefas em background
    async fn start_background_tasks(&self) {
        // Task de processamento de imagens
        let image_loader = self.image_loader.clone();
        let library_service = self.library_service.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            
            loop {
                interval.tick().await;
                
                // Processa fila de imagens
                if let Err(e) = image_loader.process_queue().await {
                    eprintln!("Erro processando imagens: {}", e);
                }
                
                // Limpa downloads antigos
                image_loader.cleanup_stale_downloads(Duration::from_secs(30)).await;
            }
        });
        
        // Task de monitoramento de performance
        let performance_monitor = self.performance_monitor.clone();
        let ui_handle = self.ui_handle.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            
            loop {
                interval.tick().await;
                
                // Coleta métricas
                let metrics = performance_monitor.get_metrics().await;
                
                // Verifica alertas
                let alerts = performance_monitor.check_targets().await;
                for alert in alerts {
                    match alert.severity {
                        crate::services::AlertSeverity::Critical => {
                            eprintln!("🚨 Crítico: {}", alert.message);
                        }
                        crate::services::AlertSeverity::Warning => {
                            eprintln!("⚠️  Aviso: {}", alert.message);
                        }
                        crate::services::AlertSeverity::Info => {
                            println!("ℹ️  Info: {}", alert.message);
                        }
                    }
                }
                
                // Atualiza UI com métricas
                if let Some(ui) = ui_handle.upgrade() {
                    ui.set_performance_metrics(SlintPerformanceMetrics {
                        visible_items: metrics.visible_items as i32,
                        total_items: metrics.total_items as i32,
                        render_time_ms: metrics.render_time_ms as i32,
                        scroll_fps: metrics.scroll_fps,
                        memory_usage_mb: metrics.memory_usage_mb,
                        cache_hit_rate: metrics.cache_hit_rate,
                    });
                }
            }
        });
        
        // Task de otimização automática
        let library_service = self.library_service.clone();
        let app_state = self.app_state.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                // Verifica se precisa otimizar
                let state = app_state.read().await;
                if state.performance_mode == PerformanceMode::HighPerformance {
                    drop(state);
                    
                    if let Err(e) = library_service.optimize_performance().await {
                        eprintln!("Erro na otimização: {}", e);
                    }
                }
            }
        });
        
        // Task de detecção de fim de scroll
        let app_state = self.app_state.clone();
        let controller = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(200));
            
            loop {
                interval.tick().await;
                
                let mut state = app_state.write().await;
                if state.is_scrolling && state.last_scroll_time.elapsed() > Duration::from_millis(300) {
                    state.is_scrolling = false;
                    drop(state);
                    
                    // Inicia preload após parar de scrollar
                    if let Err(e) = controller.handle_scroll_stopped().await {
                        eprintln!("Erro no pós-scroll: {}", e);
                    }
                }
            }
        });
    }
    
    /// Carrega livros do armazenamento
    async fn load_books_from_storage(&self) -> Result<Vec<Book>, Box<dyn std::error::Error>> {
        // Simula carregamento do banco de dados
        println!("📚 Carregando livros...");
        
        let mut books = Vec::new();
        
        // Carrega livros reais ou cria exemplos
        if let Ok(loaded_books) = self.load_from_database().await {
            books = loaded_books;
        } else {
            // Cria livros de exemplo
            books = self.create_sample_books(1000).await;
        }
        
        println!("✅ {} livros carregados", books.len());
        Ok(books)
    }
    
    /// Carrega livros do banco de dados
    async fn load_from_database(&self) -> Result<Vec<Book>, Box<dyn std::error::Error>> {
        // Implementação real carregaria do SQLite/PostgreSQL
        // Por enquanto, retorna erro para usar exemplos
        Err("Database not implemented yet".into())
    }
    
    /// Cria livros de exemplo
    async fn create_sample_books(&self, count: usize) -> Vec<Book> {
        let mut books = Vec::new();
        
        for i in 0..count {
            books.push(Book {
                id: format!("book_{}", i),
                title: format!("Sample Book {}", i + 1),
                author: format!("Author {}", (i % 50) + 1),
                cover_url: Some(format!("https://picsum.photos/150/220?random={}", i)),
                status: match i % 4 {
                    0 => BookStatus::Unread,
                    1 => BookStatus::CurrentlyReading,
                    2 => BookStatus::Finished,
                    _ => BookStatus::OnHold,
                },
                file_path: format!("/books/book_{}.epub", i),
                file_size: 1024 * 1024 * (1 + i % 10), // 1-10MB
                page_count: 100 + (i % 400), // 100-500 pages
                reading_progress: if i % 4 == 1 { (i % 100) as f32 / 100.0 } else { 0.0 },
                rating: if i % 5 == 0 { Some((i % 5 + 1) as u8) } else { None },
                tags: vec![format!("genre_{}", i % 10), format!("tag_{}", i % 20)],
                is_favorite: i % 10 == 0,
                added_date: chrono::Utc::now() - chrono::Duration::days((i % 365) as i64),
                last_read_date: if i % 3 == 0 { 
                    Some(chrono::Utc::now() - chrono::Duration::days((i % 30) as i64))
                } else { 
                    None 
                },
                ..Default::default()
            });
        }
        
        books
    }
    
    /// Atualiza UI com dados atuais
    async fn update_ui(&self) -> Result<(), Box<dyn std::error::Error>> {
        let ui = self.ui_handle.upgrade().ok_or("UI handle lost")?;
        
        // Obtém livros visíveis
        let visible_books = self.library_service.get_visible_books().await?;
        
        // Converte para formato Slint
        let slint_books: Vec<SlintBookItem> = visible_books
            .iter()
            .map(|book| self.convert_book_to_slint(book))
            .collect();
        
        // Atualiza modelo
        let model = ModelRc::new(VecModel::from(slint_books));
        ui.set_books(model);
        
        // Atualiza métricas
        let metrics = self.library_service.get_performance_metrics().await;
        ui.set_performance_metrics(SlintPerformanceMetrics {
            visible_items: metrics.visible_books as i32,
            total_items: metrics.total_books as i32,
            render_time_ms: metrics.render_time_ms as i32,
            scroll_fps: metrics.scroll_fps,
            memory_usage_mb: metrics.memory_usage_mb,
            cache_hit_rate: metrics.cache_hit_rate,
        });
        
        Ok(())
    }
    
    /// Converte Book para SlintBookItem
    fn convert_book_to_slint(&self, book: &Book) -> SlintBookItem {
        SlintBookItem {
            id: book.id.clone().into(),
            title: book.title.clone().into(),
            author: book.author.clone().into(),
            cover_url: book.cover_url.clone().unwrap_or_default().into(),
            is_cover_loaded: false, // Será atualizado pelo loader
            reading_progress: book.reading_progress,
            status: format!("{:?}", book.status).into(),
            last_read: book.last_read_date
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_default()
                .into(),
            rating: book.rating.unwrap_or(0) as i32,
            book_size_mb: book.file_size as f32 / (1024.0 * 1024.0),
            page_count: book.page_count as i32,
        }
    }
    
    /// Handlers de eventos da UI
    
    pub async fn handle_viewport_changed(&self, offset: f32, start: i32, end: i32) -> Result<(), Box<dyn std::error::Error>> {
        let frame_start = Instant::now();
        
        // Atualiza estado
        {
            let mut state = self.app_state.write().await;
            state.current_scroll_offset = offset;
            state.visible_range = start as usize..end as usize;
            state.is_scrolling = true;
            state.last_scroll_time = Instant::now();
        }
        
        // Atualiza serviço
        self.library_service.update_viewport(offset, 600.0).await?;
        
        // Registra tempo de frame
        let frame_time = frame_start.elapsed();
        self.performance_monitor.record_frame_time(frame_time).await;
        
        // Atualiza UI
        self.update_ui().await?;
        
        Ok(())
    }
    
    pub async fn handle_book_selected(&self, book_id: String) -> Result<(), Box<dyn std::error::Error>> {
        println!("📖 Livro selecionado: {}", book_id);
        
        // Atualiza estado
        {
            let mut state = self.app_state.write().await;
            state.selected_book_id = Some(book_id.clone());
        }
        
        // Carrega capa com prioridade alta
        if let Some(book) = self.find_book_by_id(&book_id).await {
            if let Some(cover_url) = &book.cover_url {
                self.image_loader.load_image(cover_url, LoadPriority::High).await?;
            }
        }
        
        Ok(())
    }
    
    pub async fn handle_book_opened(&self, book_id: String) -> Result<(), Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        
        println!("📚 Abrindo livro: {}", book_id);
        
        // Simula abertura do livro
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        // Registra tempo de abertura
        let open_time = start_time.elapsed();
        self.performance_monitor.record_book_open_time(open_time).await;
        
        // Atualiza progresso de leitura
        // (implementação real atualizaria banco de dados)
        
        println!("✅ Livro aberto em {:?}", open_time);
        Ok(())
    }
    
    pub async fn handle_preload_request(&self, start: i32, end: i32) -> Result<(), Box<dyn std::error::Error>> {
        let visible_books = self.library_service.get_visible_books().await?;
        
        let mut cover_urls = Vec::new();
        for i in start as usize..end as usize {
            if let Some(book) = visible_books.get(i) {
                if let Some(cover_url) = &book.cover_url {
                    cover_urls.push(cover_url.clone());
                }
            }
        }
        
        if !cover_urls.is_empty() {
            self.image_loader.preload_for_scroll(cover_urls, ScrollDirection::Down).await;
        }
        
        Ok(())
    }
    
    pub async fn handle_filter_changed(&self, query: String, status: String) -> Result<(), Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        
        println!("🔍 Aplicando filtro: '{}', status: '{}'", query, status);
        
        let status_filter = match status.as_str() {
            "reading" => Some(BookStatus::CurrentlyReading),
            "finished" => Some(BookStatus::Finished),
            "unread" => Some(BookStatus::Unread),
            _ => None,
        };
        
        // Atualiza estado
        {
            let mut state = self.app_state.write().await;
            state.filter_query = query.clone();
            state.filter_status = status_filter;
        }
        
        // Aplica filtro
        self.library_service.filter_books(&query, status_filter).await?;
        
        // Registra tempo do filtro
        let filter_time = start_time.elapsed();
        println!("✅ Filtro aplicado em {:?}", filter_time);
        
        // Atualiza UI
        self.update_ui().await?;
        
        Ok(())
    }
    
    pub async fn handle_settings_changed(&self, items_per_row: i32, performance_mode: String) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔧 Configurações alteradas: {} itens/linha, modo: {}", items_per_row, performance_mode);
        
        // Atualiza modo de performance
        let mode = match performance_mode.as_str() {
            "high_performance" => PerformanceMode::HighPerformance,
            "power_saving" => PerformanceMode::PowerSaving,
            _ => PerformanceMode::Balanced,
        };
        
        {
            let mut state = self.app_state.write().await;
            state.performance_mode = mode;
        }
        
        // Atualiza configuração do grid
        self.library_service.update_grid_config(items_per_row as usize, 260.0).await?;
        
        // Atualiza UI
        self.update_ui().await?;
        
        Ok(())
    }
    
    pub async fn handle_scroll_stopped(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Inicia preload agressivo após parar de scrollar
        let visible_books = self.library_service.get_visible_books().await?;
        let mut preload_urls = Vec::new();
        
        for book in visible_books {
            if let Some(cover_url) = &book.cover_url {
                preload_urls.push(cover_url.clone());
            }
        }
        
        if !preload_urls.is_empty() {
            self.image_loader.preload_for_scroll(preload_urls, ScrollDirection::None).await;
        }
        
        Ok(())
    }
    
    /// Métodos auxiliares
    
    async fn find_book_by_id(&self, book_id: &str) -> Option<Book> {
        let visible_books = self.library_service.get_visible_books().await.ok()?;
        visible_books.into_iter().find(|book| book.id == book_id)
    }
    
    pub async fn get_performance_report(&self) -> crate::services::PerformanceReport {
        self.performance_monitor.generate_report().await
    }
    
    pub async fn cleanup(&self) {
        println!("🧹 Limpando recursos...");
        
        // Para monitoramento
        self.performance_monitor.stop_monitoring().await;
        
        // Cancela downloads
        self.image_loader.cancel_all().await;
        
        // Limpa caches
        self.library_service.cleanup().await;
        
        println!("✅ Recursos limpos");
    }
}

// Implementa Clone para usar em tasks
impl Clone for EpubReaderController {
    fn clone(&self) -> Self {
        Self {
            library_service: self.library_service.clone(),
            image_loader: self.image_loader.clone(),
            performance_monitor: self.performance_monitor.clone(),
            app_state: self.app_state.clone(),
            ui_handle: self.ui_handle.clone(),
            settings: self.settings.clone(),
        }
    }
}

// Estrutura da janela principal Slint
slint::slint! {
    import { OptimizedVirtualGrid } from "./ui/components/virtual_grid.slint";
    
    export struct SlintBookItem {
        id: string,
        title: string,
        author: string,
        cover_url: string,
        is_cover_loaded: bool,
        reading_progress: float,
        status: string,
        last_read: string,
        rating: int,
        book_size_mb: float,
        page_count: int,
    }
    
    export struct SlintPerformanceMetrics {
        visible_items: int,
        total_items: int,
        render_time_ms: int,
        scroll_fps: float,
        memory_usage_mb: float,
        cache_hit_rate: float,
    }
    
    export component MainWindow inherits Window {
        title: "ePub Reader - High Performance";
        width: 1200px;
        height: 800px;
        
        // Propriedades
        in-out property <[SlintBookItem]> books: [];
        in-out property <SlintPerformanceMetrics> performance_metrics;
        in-out property <bool> show_performance_overlay: false;
        in-out property <string> filter_query: "";
        in-out property <string> filter_status: "all";
        in-out property <int> items_per_row: 6;
        in-out property <string> performance_mode: "balanced";
        
        // Callbacks
        callback viewport_changed(float /* offset */, int /* start */, int /* end */);
        callback book_selected(string /* book_id */);
        callback book_opened(string /* book_id */);
        callback preload_request(int /* start */, int /* end */);
        callback filter_changed(string /* query */, string /* status */);
        callback settings_changed(int /* items_per_row */, string /* performance_mode */);
        
        // Layout principal
        VerticalLayout {
            // Barra de ferramentas
            toolbar := Rectangle {
                height: 60px;
                background: #f0f0f0;
                border-width: 0px 0px 1px 0px;
                border-color: #e0e0e0;
                
                HorizontalLayout {
                    padding: 10px;
                    spacing: 10px;
                    alignment: center;
                    
                    Text {
                        text: "📚 ePub Reader";
                        font-size: 18px;
                        font-weight: 600;
                    }
                    
                    Rectangle { horizontal-stretch: 1; }
                    
                    // Filtro
                    LineEdit {
                        width: 200px;
                        placeholder-text: "Search books...";
                        text: filter_query;
                        
                        edited(text) => {
                            root.filter_query = text;
                            root.filter_changed(text, root.filter_status);
                        }
                    }
                    
                    // Botão de performance
                    Button {
                        text: show_performance_overlay ? "Hide Stats" : "Show Stats";
                        
                        clicked => {
                            root.show_performance_overlay = !root.show_performance_overlay;
                        }
                    }
                }
            }
            
            // Grid principal
            OptimizedVirtualGrid {
                items: root.books;
                items-per-row: root.items_per_row;
                show-debug-info: root.show_performance_overlay;
                metrics: root.performance_metrics;
                
                // Callbacks
                viewport-changed(offset, start, end) => {
                    root.viewport_changed(offset, start, end);
                }
                
                book-selected(book_id) => {
                    root.book_selected(book_id);
                }
                
                book-opened(book_id) => {
                    root.book_opened(book_id);
                }
                
                preload-request(start, end) => {
                    root.preload_request(start, end);
                }
            }
            
            // Barra de status
            status_bar := Rectangle {
                height: 30px;
                background: #f8f8f8;
                border-width: 1px 0px 0px 0px;
                border-color: #e0e0e0;
                
                HorizontalLayout {
                    padding: 5px 10px;
                    alignment: center;
                    
                    Text {
                        text: root.books.length + " books";
                        font-size: 12px;
                    }
                    
                    Rectangle { horizontal-stretch: 1; }
                    
                    if root.show_performance_overlay: Text {
                        text: "FPS: " + round(root.performance_metrics.scroll_fps) + 
                              " | Memory: " + round(root.performance_metrics.memory_usage_mb) + "MB" +
                              " | Cache: " + round(root.performance_metrics.cache_hit_rate * 100) + "%";
                        font-size: 12px;
                        color: root.performance_metrics.scroll_fps > 50 ? #4CAF50 : #FF9800;
                    }
                }
            }
        }
    }
}

/// Função principal do exemplo
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Iniciando ePub Reader com Performance Otimizada");
    
    // Cria UI
    let ui = MainWindow::new()?;
    
    // Cria controller
    let controller = EpubReaderController::new(ui.as_weak());
    
    // Inicializa controller
    controller.initialize().await?;
    
    // Configura handlers de fechamento
    let cleanup_controller = controller.clone();
    ui.on_window_close_requested(move || {
        let controller = cleanup_controller.clone();
        tokio::spawn(async move {
            controller.cleanup().await;
        });
        slint::CloseRequestResponse::HideWindow
    });
    
    // Gera relatório inicial
    tokio::spawn({
        let controller = controller.clone();
        async move {
            tokio::time::sleep(Duration::from_secs(2)).await;
            
            let report = controller.get_performance_report().await;
            println!("\n📊 Relatório de Performance Inicial:");
            println!("Score Geral: {:.1}/100", report.overall_score);
            
            for recommendation in &report.recommendations {
                println!("💡 {}", recommendation);
            }
        }
    });
    
    // Executa UI
    ui.run()?;
    
    // Cleanup final
    controller.cleanup().await;
    
    println!("✅ Aplicação finalizada");
    Ok(())
}

/// Testes de integração
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_complete_integration() {
        // Cria UI mock
        let ui = MainWindow::new().unwrap();
        
        // Cria controller
        let controller = EpubReaderController::new(ui.as_weak());
        
        // Testa inicialização
        controller.initialize().await.unwrap();
        
        // Testa operações básicas
        controller.handle_viewport_changed(100.0, 0, 20).await.unwrap();
        controller.handle_book_selected("book_0".to_string()).await.unwrap();
        controller.handle_filter_changed("Test".to_string(), "all".to_string()).await.unwrap();
        
        // Verifica performance
        let report = controller.get_performance_report().await;
        assert!(report.overall_score > 70.0);
        
        // Cleanup
        controller.cleanup().await;
    }
    
    #[tokio::test]
    async fn test_performance_under_load() {
        let ui = MainWindow::new().unwrap();
        let controller = EpubReaderController::new(ui.as_weak());
        
        controller.initialize().await.unwrap();
        
        // Simula carga pesada
        for i in 0..100 {
            controller.handle_viewport_changed(i as f32 * 50.0, i * 2, i * 2 + 20).await.unwrap();
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        
        // Verifica que ainda funciona
        let report = controller.get_performance_report().await;
        assert!(report.overall_score > 50.0); // Ainda deve ter performance aceitável
        
        controller.cleanup().await;
    }
}