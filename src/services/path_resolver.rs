use std::path::{Path, PathBuf};
use anyhow::{Result, anyhow};
use tracing::{info, warn, error};

/// Platform-specific path resolver for database and application files
pub struct PathResolver;

impl PathResolver {
    /// Get the primary database path for the current platform
    pub fn get_database_path() -> Result<PathBuf> {
        let app_dir = Self::get_app_data_directory()?;
        Ok(app_dir.join("library.db"))
    }
    
    /// Get the backup directory path
    pub fn get_backup_directory() -> Result<PathBuf> {
        let app_dir = Self::get_app_data_directory()?;
        Ok(app_dir.join("backups"))
    }
    
    /// Get fallback database path if primary path fails
    pub fn get_fallback_path() -> Result<PathBuf> {
        // Try user's home directory first
        if let Some(home_dir) = dirs::home_dir() {
            let fallback_dir = home_dir.join(".ebook-reader");
            if Self::ensure_directory_exists(&fallback_dir).is_ok() {
                return Ok(fallback_dir.join("library.db"));
            }
        }
        
        // Try current directory as last resort
        let current_dir = std::env::current_dir()
            .map_err(|e| anyhow!("Cannot determine current directory: {}", e))?;
        let fallback_dir = current_dir.join("ebook-reader-data");
        Self::ensure_directory_exists(&fallback_dir)?;
        
        Ok(fallback_dir.join("library.db"))
    }
    
    /// Get platform-specific application data directory
    pub fn get_app_data_directory() -> Result<PathBuf> {
        let app_dir = if cfg!(target_os = "windows") {
            // Windows: %APPDATA%\ebook-reader
            dirs::config_dir()
                .or_else(|| dirs::data_dir())
                .ok_or_else(|| anyhow!("Cannot determine Windows app data directory"))?
                .join("ebook-reader")
        } else if cfg!(target_os = "macos") {
            // macOS: ~/Library/Application Support/ebook-reader
            dirs::data_dir()
                .ok_or_else(|| anyhow!("Cannot determine macOS app data directory"))?
                .join("ebook-reader")
        } else {
            // Linux and other Unix-like: ~/.local/share/ebook-reader
            dirs::data_dir()
                .or_else(|| {
                    dirs::home_dir().map(|home| home.join(".local").join("share"))
                })
                .ok_or_else(|| anyhow!("Cannot determine Linux app data directory"))?
                .join("ebook-reader")
        };
        
        info!("Using app data directory: {}", app_dir.display());
        Ok(app_dir)
    }
    
    /// Get cache directory for temporary files
    pub fn get_cache_directory() -> Result<PathBuf> {
        let cache_dir = if cfg!(target_os = "windows") {
            // Windows: %LOCALAPPDATA%\ebook-reader\cache
            dirs::cache_dir()
                .ok_or_else(|| anyhow!("Cannot determine Windows cache directory"))?
                .join("ebook-reader")
        } else if cfg!(target_os = "macos") {
            // macOS: ~/Library/Caches/ebook-reader
            dirs::cache_dir()
                .ok_or_else(|| anyhow!("Cannot determine macOS cache directory"))?
                .join("ebook-reader")
        } else {
            // Linux: ~/.cache/ebook-reader
            dirs::cache_dir()
                .or_else(|| {
                    dirs::home_dir().map(|home| home.join(".cache"))
                })
                .ok_or_else(|| anyhow!("Cannot determine Linux cache directory"))?
                .join("ebook-reader")
        };
        
        Ok(cache_dir)
    }
    
    /// Get logs directory
    pub fn get_logs_directory() -> Result<PathBuf> {
        let app_dir = Self::get_app_data_directory()?;
        Ok(app_dir.join("logs"))
    }
    
    /// Ensure directory exists with proper permissions
    pub fn ensure_directory_exists(path: &Path) -> Result<()> {
        if path.exists() {
            if !path.is_dir() {
                return Err(anyhow!("Path exists but is not a directory: {}", path.display()));
            }
            return Ok(());
        }
        
        info!("Creating directory: {}", path.display());
        std::fs::create_dir_all(path)
            .map_err(|e| anyhow!("Failed to create directory {}: {}", path.display(), e))?;
        
        // Set appropriate permissions on Unix-like systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(path) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o755); // rwxr-xr-x
                if let Err(e) = std::fs::set_permissions(path, perms) {
                    warn!("Failed to set directory permissions for {}: {}", path.display(), e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Check if path has read/write permissions
    pub fn check_permissions(path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(anyhow!("Path does not exist: {}", path.display()));
        }
        
        // Check read permission
        match std::fs::metadata(path) {
            Ok(metadata) => {
                if metadata.permissions().readonly() {
                    return Err(anyhow!("Path is read-only: {}", path.display()));
                }
            }
            Err(e) => {
                return Err(anyhow!("Cannot read path metadata {}: {}", path.display(), e));
            }
        }
        
        // Test write permission by creating a temporary file
        if path.is_dir() {
            let test_file = path.join(".write_test");
            match std::fs::write(&test_file, b"test") {
                Ok(_) => {
                    let _ = std::fs::remove_file(&test_file);
                }
                Err(e) => {
                    return Err(anyhow!("No write permission for directory {}: {}", path.display(), e));
                }
            }
        }
        
        Ok(())
    }
    
    /// Get temporary directory for the application
    pub fn get_temp_directory() -> Result<PathBuf> {
        let temp_dir = std::env::temp_dir().join("ebook-reader");
        Self::ensure_directory_exists(&temp_dir)?;
        Ok(temp_dir)
    }
    
    /// Resolve database path with fallback logic
    pub fn resolve_database_path_with_fallback() -> Result<PathBuf> {
        // Try primary path first
        match Self::get_database_path() {
            Ok(primary_path) => {
                if let Some(parent) = primary_path.parent() {
                    match Self::ensure_directory_exists(parent) {
                        Ok(_) => {
                            if Self::check_permissions(parent).is_ok() {
                                info!("Using primary database path: {}", primary_path.display());
                                return Ok(primary_path);
                            } else {
                                warn!("No permissions for primary path, trying fallback");
                            }
                        }
                        Err(e) => {
                            warn!("Cannot create primary directory: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Cannot determine primary database path: {}", e);
            }
        }
        
        // Try fallback path
        match Self::get_fallback_path() {
            Ok(fallback_path) => {
                info!("Using fallback database path: {}", fallback_path.display());
                Ok(fallback_path)
            }
            Err(e) => {
                error!("Cannot determine fallback database path: {}", e);
                Err(e)
            }
        }
    }
    
    /// Get configuration file path
    pub fn get_config_path() -> Result<PathBuf> {
        let app_dir = Self::get_app_data_directory()?;
        Ok(app_dir.join("config.toml"))
    }
    
    /// Validate path is safe to use (prevent directory traversal)
    pub fn validate_path_safety(path: &Path) -> Result<()> {
        let path_str = path.to_string_lossy();
        
        // Check for directory traversal attempts
        if path_str.contains("..") {
            return Err(anyhow!("Path contains directory traversal: {}", path_str));
        }
        
        // Check for absolute paths that might be dangerous
        if path.is_absolute() {
            // Allow absolute paths only in expected locations
            let app_dir = Self::get_app_data_directory()?;
            let cache_dir = Self::get_cache_directory()?;
            let temp_dir = Self::get_temp_directory()?;
            
            if !path.starts_with(&app_dir) && 
               !path.starts_with(&cache_dir) && 
               !path.starts_with(&temp_dir) {
                warn!("Path outside expected directories: {}", path_str);
            }
        }
        
        Ok(())
    }
    
    /// Get platform-specific file extension for executables
    pub fn get_executable_extension() -> &'static str {
        if cfg!(target_os = "windows") {
            ".exe"
        } else {
            ""
        }
    }
    
    /// Check if running in a sandboxed environment
    pub fn is_sandboxed() -> bool {
        #[cfg(target_os = "macos")]
        {
            // Check for macOS sandbox
            std::env::var("APP_SANDBOX_CONTAINER_ID").is_ok()
        }
        
        #[cfg(target_os = "linux")]
        {
            // Check for common Linux sandboxes
            std::env::var("FLATPAK_ID").is_ok() || 
            std::env::var("SNAP_NAME").is_ok()
        }
        
        #[cfg(target_os = "windows")]
        {
            // Windows sandboxing detection would be more complex
            false
        }
        
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_get_app_data_directory() {
        let result = PathResolver::get_app_data_directory();
        assert!(result.is_ok());
        
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("ebook-reader"));
    }
    
    #[test]
    fn test_ensure_directory_exists() {
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path().join("test_subdir");
        
        assert!(!test_path.exists());
        
        let result = PathResolver::ensure_directory_exists(&test_path);
        assert!(result.is_ok());
        assert!(test_path.exists());
        assert!(test_path.is_dir());
    }
    
    #[test]
    fn test_check_permissions() {
        let temp_dir = TempDir::new().unwrap();
        let result = PathResolver::check_permissions(temp_dir.path());
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_validate_path_safety() {
        // Safe paths
        assert!(PathResolver::validate_path_safety(Path::new("safe/path")).is_ok());
        assert!(PathResolver::validate_path_safety(Path::new("./safe/path")).is_ok());
        
        // Unsafe paths
        assert!(PathResolver::validate_path_safety(Path::new("../unsafe/path")).is_err());
        assert!(PathResolver::validate_path_safety(Path::new("safe/../unsafe")).is_err());
    }
    
    #[test]
    fn test_get_database_path() {
        let result = PathResolver::get_database_path();
        assert!(result.is_ok());
        
        let path = result.unwrap();
        assert!(path.to_string_lossy().ends_with("library.db"));
    }
    
    #[test]
    fn test_get_fallback_path() {
        let result = PathResolver::get_fallback_path();
        assert!(result.is_ok());
        
        let path = result.unwrap();
        assert!(path.to_string_lossy().ends_with("library.db"));
    }
    
    #[test]
    fn test_resolve_database_path_with_fallback() {
        let result = PathResolver::resolve_database_path_with_fallback();
        assert!(result.is_ok());
        
        let path = result.unwrap();
        assert!(path.to_string_lossy().ends_with("library.db"));
    }
}