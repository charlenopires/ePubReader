use std::path::{Path, PathBuf};
use anyhow::{Result, anyhow};
use sqlx::SqlitePool;
use thiserror::Error;
use tracing::{info, warn, error};

/// Custom error types for database initialization
#[derive(Debug, Error)]
pub enum DatabaseInitError {
    #[error("Failed to create database directory: {0}")]
    DirectoryCreation(#[from] std::io::Error),
    
    #[error("Database file is corrupted")]
    CorruptedDatabase,
    
    #[error("Permission denied accessing database path: {path}")]
    PermissionDenied { path: PathBuf },
    
    #[error("Database is locked by another process")]
    DatabaseLocked,
    
    #[error("Schema migration failed: {0}")]
    MigrationFailed(String),
    
    #[error("Backup operation failed: {0}")]
    BackupFailed(String),
    
    #[error("Database connection failed: {0}")]
    ConnectionFailed(String),
}

/// Database status enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum DatabaseStatus {
    Ready,
    Missing,
    Corrupted,
    PermissionDenied,
    Locked,
    InvalidSchema,
}

/// Database initializer for handling setup and validation
pub struct DatabaseInitializer {
    database_path: PathBuf,
    backup_enabled: bool,
    max_retry_attempts: u32,
}

impl DatabaseInitializer {
    /// Create a new database initializer
    pub fn new(database_path: PathBuf) -> Self {
        Self {
            database_path,
            backup_enabled: true,
            max_retry_attempts: 3,
        }
    }
    
    /// Ensure database is ready for use
    pub async fn ensure_database_ready(&self) -> Result<PathBuf, DatabaseInitError> {
        info!("Initializing database at: {}", self.database_path.display());
        
        // Create database directory if it doesn't exist
        self.create_database_directory().await?;
        
        // Check database status
        let status = self.validate_database_file().await?;
        
        match status {
            DatabaseStatus::Ready => {
                info!("Database is ready");
                Ok(self.database_path.clone())
            }
            DatabaseStatus::Missing => {
                info!("Database file missing, creating new database");
                self.create_new_database().await?;
                Ok(self.database_path.clone())
            }
            DatabaseStatus::Corrupted => {
                warn!("Database is corrupted, attempting recovery");
                self.handle_corrupted_database().await?;
                Ok(self.database_path.clone())
            }
            DatabaseStatus::PermissionDenied => {
                error!("Permission denied accessing database");
                Err(DatabaseInitError::PermissionDenied {
                    path: self.database_path.clone(),
                })
            }
            DatabaseStatus::Locked => {
                warn!("Database is locked, retrying...");
                self.handle_locked_database().await?;
                Ok(self.database_path.clone())
            }
            DatabaseStatus::InvalidSchema => {
                warn!("Invalid schema detected, attempting migration");
                self.handle_invalid_schema().await?;
                Ok(self.database_path.clone())
            }
        }
    }
    
    /// Create database directory with proper permissions
    async fn create_database_directory(&self) -> Result<(), DatabaseInitError> {
        if let Some(parent) = self.database_path.parent() {
            if !parent.exists() {
                info!("Creating database directory: {}", parent.display());
                std::fs::create_dir_all(parent)?;
                
                // Set appropriate permissions (Unix-like systems)
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = std::fs::metadata(parent)?.permissions();
                    perms.set_mode(0o755); // rwxr-xr-x
                    std::fs::set_permissions(parent, perms)?;
                }
            }
            
            // Check if we can write to the directory
            if !self.check_directory_writable(parent).await? {
                return Err(DatabaseInitError::PermissionDenied {
                    path: parent.to_path_buf(),
                });
            }
        }
        
        Ok(())
    }
    
    /// Check if directory is writable
    async fn check_directory_writable(&self, dir: &Path) -> Result<bool, DatabaseInitError> {
        let test_file = dir.join(".write_test");
        
        match std::fs::write(&test_file, b"test") {
            Ok(_) => {
                let _ = std::fs::remove_file(&test_file);
                Ok(true)
            }
            Err(_) => Ok(false),
        }
    }
    
    /// Validate database file status
    async fn validate_database_file(&self) -> Result<DatabaseStatus, DatabaseInitError> {
        if !self.database_path.exists() {
            return Ok(DatabaseStatus::Missing);
        }
        
        // Check if file is readable
        match std::fs::metadata(&self.database_path) {
            Ok(metadata) => {
                if metadata.len() == 0 {
                    return Ok(DatabaseStatus::Missing);
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    return Ok(DatabaseStatus::PermissionDenied);
                }
                return Ok(DatabaseStatus::Corrupted);
            }
        }
        
        // Try to connect to database to check if it's valid
        match self.test_database_connection().await {
            Ok(true) => Ok(DatabaseStatus::Ready),
            Ok(false) => Ok(DatabaseStatus::InvalidSchema),
            Err(e) => {
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("locked") {
                    Ok(DatabaseStatus::Locked)
                } else if error_msg.contains("corrupt") || error_msg.contains("malformed") {
                    Ok(DatabaseStatus::Corrupted)
                } else {
                    Ok(DatabaseStatus::Corrupted)
                }
            }
        }
    }
    
    /// Test database connection and basic functionality
    async fn test_database_connection(&self) -> Result<bool> {
        let database_url = format!("sqlite://{}?mode=rwc", self.database_path.display());
        
        match SqlitePool::connect(&database_url).await {
            Ok(pool) => {
                // Test basic query
                match sqlx::query("SELECT 1").fetch_one(&pool).await {
                    Ok(_) => {
                        pool.close().await;
                        Ok(true)
                    }
                    Err(_) => {
                        pool.close().await;
                        Ok(false)
                    }
                }
            }
            Err(e) => Err(anyhow!("Connection failed: {}", e)),
        }
    }
    
    /// Create a new database file
    async fn create_new_database(&self) -> Result<(), DatabaseInitError> {
        info!("Creating new database file");
        
        let database_url = format!("sqlite://{}?mode=rwc", self.database_path.display());
        
        match SqlitePool::connect(&database_url).await {
            Ok(pool) => {
                // Create a simple test table to ensure database is working
                sqlx::query("CREATE TABLE IF NOT EXISTS _init_test (id INTEGER PRIMARY KEY)")
                    .execute(&pool)
                    .await
                    .map_err(|e| DatabaseInitError::ConnectionFailed(e.to_string()))?;
                
                // Drop the test table
                sqlx::query("DROP TABLE IF EXISTS _init_test")
                    .execute(&pool)
                    .await
                    .map_err(|e| DatabaseInitError::ConnectionFailed(e.to_string()))?;
                
                pool.close().await;
                info!("New database created successfully");
                Ok(())
            }
            Err(e) => {
                error!("Failed to create database: {}", e);
                Err(DatabaseInitError::ConnectionFailed(e.to_string()))
            }
        }
    }
    
    /// Handle corrupted database
    async fn handle_corrupted_database(&self) -> Result<(), DatabaseInitError> {
        warn!("Handling corrupted database");
        
        // Create backup of corrupted file
        if self.backup_enabled {
            let backup_path = self.create_corruption_backup().await?;
            info!("Corrupted database backed up to: {}", backup_path.display());
        }
        
        // Remove corrupted file
        if let Err(e) = std::fs::remove_file(&self.database_path) {
            warn!("Failed to remove corrupted database: {}", e);
        }
        
        // Create new database
        self.create_new_database().await?;
        
        Ok(())
    }
    
    /// Handle locked database with retry logic
    async fn handle_locked_database(&self) -> Result<(), DatabaseInitError> {
        use tokio::time::{sleep, Duration};
        
        for attempt in 1..=self.max_retry_attempts {
            info!("Attempting to connect to locked database (attempt {}/{})", attempt, self.max_retry_attempts);
            
            match self.test_database_connection().await {
                Ok(_) => {
                    info!("Database is now accessible");
                    return Ok(());
                }
                Err(_) => {
                    if attempt < self.max_retry_attempts {
                        let delay = Duration::from_millis(1000 * attempt as u64); // Exponential backoff
                        sleep(delay).await;
                    }
                }
            }
        }
        
        Err(DatabaseInitError::DatabaseLocked)
    }
    
    /// Handle invalid schema
    async fn handle_invalid_schema(&self) -> Result<(), DatabaseInitError> {
        // This will be implemented when we add the SchemaManager
        warn!("Invalid schema detected - schema migration not yet implemented");
        Ok(())
    }
    
    /// Create backup of corrupted database
    async fn create_corruption_backup(&self) -> Result<PathBuf, DatabaseInitError> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("corrupted_backup_{}.db", timestamp);
        let backup_path = self.database_path.parent()
            .unwrap_or_else(|| Path::new("."))
            .join(backup_name);
        
        std::fs::copy(&self.database_path, &backup_path)
            .map_err(|e| DatabaseInitError::BackupFailed(e.to_string()))?;
        
        Ok(backup_path)
    }
    
    /// Get database path
    pub fn database_path(&self) -> &Path {
        &self.database_path
    }
    
    /// Enable or disable backup creation
    pub fn set_backup_enabled(&mut self, enabled: bool) {
        self.backup_enabled = enabled;
    }
    
    /// Set maximum retry attempts for locked database
    pub fn set_max_retry_attempts(&mut self, attempts: u32) {
        self.max_retry_attempts = attempts;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_create_new_database() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let initializer = DatabaseInitializer::new(db_path.clone());
        let result = initializer.ensure_database_ready().await;
        
        assert!(result.is_ok());
        assert!(db_path.exists());
    }
    
    #[tokio::test]
    async fn test_missing_directory_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("subdir").join("test.db");
        
        let initializer = DatabaseInitializer::new(db_path.clone());
        let result = initializer.ensure_database_ready().await;
        
        assert!(result.is_ok());
        assert!(db_path.exists());
        assert!(db_path.parent().unwrap().exists());
    }
    
    #[tokio::test]
    async fn test_database_status_detection() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let initializer = DatabaseInitializer::new(db_path.clone());
        
        // Test missing database
        let status = initializer.validate_database_file().await.unwrap();
        assert_eq!(status, DatabaseStatus::Missing);
        
        // Create empty file
        std::fs::write(&db_path, b"").unwrap();
        let status = initializer.validate_database_file().await.unwrap();
        assert_eq!(status, DatabaseStatus::Missing);
        
        // Create invalid file
        std::fs::write(&db_path, b"invalid content").unwrap();
        let status = initializer.validate_database_file().await.unwrap();
        assert_eq!(status, DatabaseStatus::Corrupted);
    }
}