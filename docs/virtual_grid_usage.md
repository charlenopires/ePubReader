# Guia de Uso: Virtual Grid Otimizado

## Visão Geral

O Virtual Grid Otimizado é um sistema de alta performance para renderização de bibliotecas grandes (1000+ livros) com scroll suave e uso eficiente de memória.

## Instalação

```toml
[dependencies]
# Adicione ao seu Cargo.toml
lru = "0.12"
rayon = "1.8"
tokio = { version = "1.0", features = ["full"] }
```

## Uso Básico

### 1. Configuração Inicial

```rust
use crate::services::{OptimizedLibraryService, LibrarySettings};

// Configuração customizada
let settings = LibrarySettings {
    max_cache_memory_mb: 200,     // 200MB cache
    items_per_row: 6,             // 6 livros por linha
    item_height: 260.0,           // Altura de cada item
    enable_preloading: true,      // Preload inteligente
    preload_distance: 20,         // Preload 20 itens à frente
    scroll_buffer_size: 2,        // 2 linhas de buffer
    performance_monitoring: true, // Monitoramento ativo
};

// Criar serviço
let service = OptimizedLibraryService::new(settings);
```

### 2. Inicialização com Livros

```rust
// Carrega livros do banco de dados
let books = load_books_from_database().await?;

// Inicializa o serviço
service.initialize(books).await?;

// Verifica métricas iniciais
let metrics = service.get_performance_metrics().await;
println!("Biblioteca inicializada: {} livros, {:.1}MB memória", 
         metrics.total_books, metrics.memory_usage_mb);
```

### 3. Integração com UI (Slint)

```slint
// main.slint
import { OptimizedVirtualGrid } from "./ui/components/virtual_grid.slint";

export component MainWindow inherits Window {
    property <[VirtualBookItem]> books: [];
    property <GridPerformanceMetrics> metrics;
    
    VerticalLayout {
        // Grid otimizado
        grid := OptimizedVirtualGrid {
            items: root.books;
            items-per-row: 6;
            show-debug-info: true;
            metrics: root.metrics;
            
            // Callbacks
            book-selected(book_id) => {
                // Lógica de seleção
            }
            
            viewport-changed(offset, start, end) => {
                // Atualiza backend
                LibraryController.update_viewport(offset, start, end);
            }
            
            preload-request(start, end) => {
                // Solicita preload
                LibraryController.preload_covers(start, end);
            }
        }
    }
}
```

### 4. Controller de Integração

```rust
pub struct LibraryController {
    service: Arc<OptimizedLibraryService>,
    ui_handle: slint::Weak<MainWindow>,
}

impl LibraryController {
    pub async fn update_viewport(&self, offset: i32, start: i32, end: i32) {
        // Atualiza serviço
        self.service.update_viewport(offset as f32, 600.0).await.unwrap();
        
        // Atualiza UI
        let ui = self.ui_handle.upgrade().unwrap();
        let visible_books = self.service.get_visible_books().await.unwrap();
        ui.set_books(convert_to_slint_items(visible_books));
        
        // Atualiza métricas
        let metrics = self.service.get_performance_metrics().await;
        ui.set_metrics(convert_to_slint_metrics(metrics));
    }
    
    pub async fn preload_covers(&self, start: i32, end: i32) {
        // Preload é feito automaticamente pelo serviço
        // Mas pode ser customizado aqui
    }
}
```

## Configurações Avançadas

### 1. Otimização para Dispositivos Diferentes

```rust
// Configuração para dispositivos lentos
let low_end_settings = LibrarySettings {
    max_cache_memory_mb: 50,      // Menos memória
    items_per_row: 4,             // Menos itens por linha
    enable_preloading: false,     // Sem preload
    scroll_buffer_size: 1,        // Buffer menor
    performance_monitoring: false,
    ..Default::default()
};

// Configuração para dispositivos potentes
let high_end_settings = LibrarySettings {
    max_cache_memory_mb: 500,     // Mais memória
    items_per_row: 8,             // Mais itens por linha
    enable_preloading: true,      // Preload agressivo
    preload_distance: 50,         // Preload maior
    scroll_buffer_size: 4,        // Buffer maior
    performance_monitoring: true,
    ..Default::default()
};
```

### 2. Monitoramento de Performance

```rust
// Monitora performance em tempo real
tokio::spawn({
    let service = service.clone();
    async move {
        loop {
            sleep(Duration::from_secs(1)).await;
            
            let metrics = service.get_performance_metrics().await;
            
            // Alerta se performance degradar
            if metrics.memory_usage_mb > 300.0 {
                println!("⚠️  Alto uso de memória: {:.1}MB", metrics.memory_usage_mb);
            }
            
            if metrics.cache_hit_rate < 0.7 {
                println!("⚠️  Cache hit rate baixo: {:.1}%", metrics.cache_hit_rate * 100.0);
            }
            
            // Otimiza automaticamente
            service.optimize_performance().await.unwrap();
        }
    }
});
```

### 3. Filtros e Busca

```rust
// Filtro por status
service.filter_books("", Some(BookStatus::CurrentlyReading)).await?;

// Busca textual
service.filter_books("Harry Potter", None).await?;

// Filtro combinado
service.filter_books("Fantasy", Some(BookStatus::Unread)).await?;
```

## Otimizações de Performance

### 1. Scroll Suave

```rust
// Detecta direção do scroll para otimizar preload
let mut last_scroll_offset = 0.0;

// No handler de scroll
let current_offset = scroll_view.viewport_y();
let scroll_direction = if current_offset > last_scroll_offset {
    ScrollDirection::Down
} else if current_offset < last_scroll_offset {
    ScrollDirection::Up
} else {
    ScrollDirection::None
};

// Atualiza com direção
service.update_viewport_with_direction(current_offset, scroll_direction).await?;
last_scroll_offset = current_offset;
```

### 2. Carregamento Inteligente

```rust
// Prioriza capas por proximidade
let visible_books = service.get_visible_books().await?;
for (index, book) in visible_books.iter().enumerate() {
    let priority = match index {
        0..=5 => Priority::Critical,    // Primeiros 6 = crítico
        6..=17 => Priority::High,       // Próximos 12 = alto
        _ => Priority::Medium,          // Resto = médio
    };
    
    if let Some(cover_url) = &book.cover_url {
        image_loader.load_with_priority(cover_url, priority).await?;
    }
}
```

### 3. Cleanup Automático

```rust
// Limpa recursos periodicamente
tokio::spawn({
    let service = service.clone();
    async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            
            // Limpa cache antigo
            service.cleanup_old_cache().await;
            
            // Otimiza configurações
            service.optimize_performance().await.unwrap();
            
            // Coleta lixo se necessário
            if service.get_memory_usage().await > 400.0 {
                service.force_gc().await;
            }
        }
    }
});
```

## Debugging e Profiling

### 1. Debug Visual

```slint
// Ativa overlay de debug
grid := OptimizedVirtualGrid {
    show-debug-info: true;  // Mostra métricas na tela
    metrics: root.metrics;
}
```

### 2. Logging Detalhado

```rust
// Ativa logs detalhados
env_logger::init();

// Ou use tracing
tracing_subscriber::fmt::init();

// Logs automáticos do serviço
service.enable_detailed_logging(true).await;
```

### 3. Profiling com Cargo

```bash
# Instala flamegraph
cargo install flamegraph

# Executa profiling
cargo flamegraph --bin epub-reader

# Analisa performance
cargo bench
```

## Troubleshooting

### Problemas Comuns

1. **Scroll Lento**
   ```rust
   // Reduz buffer size
   settings.scroll_buffer_size = 1;
   
   // Desativa preload
   settings.enable_preloading = false;
   ```

2. **Alto Uso de Memória**
   ```rust
   // Reduz cache
   settings.max_cache_memory_mb = 50;
   
   // Força cleanup
   service.force_cleanup().await;
   ```

3. **Capas Não Carregam**
   ```rust
   // Verifica conectividade
   if !is_online().await {
       service.set_offline_mode(true).await;
   }
   
   // Verifica cache
   let stats = service.get_cache_stats().await;
   println!("Cache: {:.1}% hit rate", stats.hit_rate * 100.0);
   ```

### Logs Úteis

```rust
// Métricas detalhadas
let metrics = service.get_detailed_metrics().await;
println!("📊 Detalhes:");
println!("  - Livros visíveis: {}/{}", metrics.visible_books, metrics.total_books);
println!("  - Render time: {}ms", metrics.render_time_ms);
println!("  - Memória: {:.1}MB", metrics.memory_usage_mb);
println!("  - Cache hit: {:.1}%", metrics.cache_hit_rate * 100.0);
println!("  - FPS: {:.1}", metrics.scroll_fps);
```

## Exemplo Completo

```rust
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Configuração
    let settings = LibrarySettings {
        max_cache_memory_mb: 200,
        items_per_row: 6,
        item_height: 260.0,
        enable_preloading: true,
        preload_distance: 20,
        scroll_buffer_size: 2,
        performance_monitoring: true,
    };
    
    // 2. Inicialização
    let service = Arc::new(OptimizedLibraryService::new(settings));
    let books = load_books_from_database().await?;
    service.initialize(books).await?;
    
    // 3. Monitoramento
    let monitor_service = service.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            
            let metrics = monitor_service.get_performance_metrics().await;
            println!("📊 FPS: {:.1}, Memory: {:.1}MB, Cache: {:.1}%",
                    metrics.scroll_fps, metrics.memory_usage_mb, 
                    metrics.cache_hit_rate * 100.0);
            
            monitor_service.optimize_performance().await.unwrap();
        }
    });
    
    // 4. UI (integração com Slint)
    let ui = MainWindow::new()?;
    
    // 5. Controller
    let controller = LibraryController::new(service, ui.as_weak());
    
    // 6. Executar
    ui.run()?;
    
    Ok(())
}
```

Este sistema garante performance excelente mesmo com bibliotecas de 5000+ livros, mantendo 60 FPS no scroll e uso controlado de memória.