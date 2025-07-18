# Design Document

## Overview

This design addresses the database initialization error by implementing a robust database setup system that handles directory creation, database file initialization, schema migration, and error recovery. The solution ensures the application can start successfully on any supported platform with proper error handling and user feedback.

## Architecture

### Components

1. **DatabaseInitializer** - Handles database setup and initialization
2. **SchemaManager** - Manages database schema versions and migrations
3. **BackupManager** - Handles database backups and recovery
4. **PathResolver** - Resolves platform-specific database paths
5. **ErrorHandler** - Provides user-friendly error messages and recovery options

### Data Flow

```
Application Start
    ↓
PathResolver.get_database_path()
    ↓
DatabaseInitializer.ensure_database_exists()
    ↓
SchemaManager.check_and_migrate()
    ↓
DatabaseService.new() with validated path
    ↓
Application Ready
```

## Components and Interfaces

### DatabaseInitializer

```rust
pub struct DatabaseInitializer {
    path_resolver: PathResolver,
    backup_manager: BackupManager,
}

impl DatabaseInitializer {
    pub async fn ensure_database_ready(database_path: &Path) -> Result<PathBuf>;
    pub async fn create_database_directory(path: &Path) -> Result<()>;
    pub async fn validate_database_file(path: &Path) -> Result<DatabaseStatus>;
    pub async fn handle_corrupted_database(path: &Path) -> Result<()>;
}
```

### PathResolver

```rust
pub struct PathResolver;

impl PathResolver {
    pub fn get_database_path() -> Result<PathBuf>;
    pub fn get_backup_directory() -> Result<PathBuf>;
    pub fn get_fallback_path() -> Result<PathBuf>;
    pub fn ensure_directory_exists(path: &Path) -> Result<()>;
    pub fn check_permissions(path: &Path) -> Result<()>;
}
```

### SchemaManager

```rust
pub struct SchemaManager {
    pool: SqlitePool,
}

impl SchemaManager {
    pub async fn get_current_version(&self) -> Result<u32>;
    pub async fn apply_migrations(&self, target_version: u32) -> Result<()>;
    pub async fn create_schema_version_table(&self) -> Result<()>;
    pub async fn validate_schema(&self) -> Result<bool>;
}
```

### BackupManager

```rust
pub struct BackupManager {
    backup_dir: PathBuf,
}

impl BackupManager {
    pub async fn create_backup(&self, database_path: &Path) -> Result<PathBuf>;
    pub async fn restore_from_backup(&self, database_path: &Path) -> Result<()>;
    pub async fn cleanup_old_backups(&self) -> Result<()>;
    pub fn list_available_backups(&self) -> Result<Vec<BackupInfo>>;
}
```

## Data Models

### DatabaseStatus

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum DatabaseStatus {
    Ready,
    Missing,
    Corrupted,
    PermissionDenied,
    Locked,
    InvalidSchema,
}
```

### BackupInfo

```rust
#[derive(Debug, Clone)]
pub struct BackupInfo {
    pub path: PathBuf,
    pub created_at: DateTime<Utc>,
    pub size: u64,
    pub schema_version: u32,
}
```

### DatabaseConfig

```rust
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub path: PathBuf,
    pub backup_enabled: bool,
    pub max_backups: usize,
    pub connection_timeout: Duration,
    pub retry_attempts: u32,
    pub retry_delay: Duration,
}
```

## Error Handling

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
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
}
```

### Recovery Strategies

1. **Missing Database**: Create new database with latest schema
2. **Corrupted Database**: Restore from backup or create fresh database
3. **Permission Issues**: Try alternative locations or show user guidance
4. **Locked Database**: Retry with exponential backoff
5. **Schema Issues**: Apply migrations or recreate schema

## Testing Strategy

### Unit Tests

1. **PathResolver Tests**
   - Test platform-specific path resolution
   - Test directory creation with various permissions
   - Test fallback path scenarios

2. **DatabaseInitializer Tests**
   - Test database creation from scratch
   - Test corruption detection and recovery
   - Test permission handling

3. **SchemaManager Tests**
   - Test schema version detection
   - Test migration application
   - Test rollback scenarios

4. **BackupManager Tests**
   - Test backup creation and restoration
   - Test backup cleanup
   - Test backup validation

### Integration Tests

1. **End-to-End Database Setup**
   - Test complete initialization flow
   - Test error scenarios and recovery
   - Test cross-platform compatibility

2. **Migration Testing**
   - Test migrations from various schema versions
   - Test migration failure recovery
   - Test data preservation during migrations

### Performance Tests

1. **Startup Time**
   - Measure database initialization time
   - Test with various database sizes
   - Test backup/restore performance

## Platform Considerations

### Windows
- Use `%APPDATA%\ebook-reader\` for database storage
- Handle Windows file locking behavior
- Support UNC paths and long file names

### macOS
- Use `~/Library/Application Support/ebook-reader/` for database storage
- Handle macOS sandboxing if applicable
- Support case-sensitive/insensitive filesystems

### Linux
- Use `~/.local/share/ebook-reader/` for database storage
- Handle various filesystem permissions
- Support different Linux distributions

## Security Considerations

1. **File Permissions**: Set appropriate permissions on database files
2. **Backup Security**: Encrypt backups if they contain sensitive data
3. **Path Validation**: Validate all file paths to prevent directory traversal
4. **Error Messages**: Don't expose sensitive path information in error messages

## Migration Strategy

### Schema Versioning

```sql
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL,
    description TEXT
);
```

### Migration Files

Each migration will be a separate function with:
- Version number
- Description
- Up migration (apply changes)
- Down migration (rollback changes)
- Validation (verify migration success)

## Monitoring and Logging

1. **Initialization Metrics**: Track database setup time and success rate
2. **Error Logging**: Log all database initialization errors with context
3. **Performance Monitoring**: Monitor database operation performance
4. **User Feedback**: Provide progress indicators during long operations