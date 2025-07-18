use std::fs;
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};
use tracing_appender::rolling::{RollingFileAppender, Rotation};

pub fn init_logging() -> Result<(), Box<dyn std::error::Error>> {
    // Create logs directory
    let log_dir = get_log_directory()?;
    fs::create_dir_all(&log_dir)?;
    
    // Create file appender with daily rotation
    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        &log_dir,
        "epubreader.log"
    );
    
    // Create console layer for development
    let console_layer = fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true);
    
    // Create file layer for persistent logging
    let file_layer = fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(false)
        .with_writer(file_appender);
    
    // Set log level filter
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,epubreader=debug"));
    
    // Initialize subscriber
    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .init();
    
    info!("Logging initialized. Log directory: {}", log_dir.display());
    
    Ok(())
}

pub fn get_log_directory() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let app_dir = crate::file_manager::get_app_directory()?;
    Ok(app_dir.join("logs"))
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        tracing::error!($($arg)*);
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        tracing::warn!($($arg)*);
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        tracing::info!($($arg)*);
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        tracing::debug!($($arg)*);
    };
}