# 📋 ePub Reader Library - Configuration Guide

## 🎯 Overview
This document describes all configuration files and options available in the ePub Reader Library. The application uses a hierarchical configuration system with multiple files for different aspects of the system.

## 📁 Configuration Files Structure

```
epub-reader-library/
├── app.config.toml          # Master configuration file
├── config.toml              # Main application configuration
├── ollama.config.toml       # Ollama service configuration
├── reader.config.json       # HTML reader configuration
├── logging.config.toml      # Logging configuration
├── .env.development         # Development environment
├── .env.production          # Production environment
├── .env.example             # Environment template
└── user.config.toml         # User preferences (auto-generated)
```

## 🔧 Configuration Files

### 1. `app.config.toml` - Master Configuration
The main configuration file that references all other configuration files and controls the configuration loading system.

**Key Sections:**
- `[config_files]` - References to other config files
- `[environment]` - Environment detection and switching
- `[config_loading]` - Loading order and behavior
- `[validation]` - Configuration validation settings
- `[hot_reload]` - Hot reload configuration changes

### 2. `config.toml` - Main Application Configuration
Contains all core application settings including database, storage, UI, and performance configurations.

**Key Sections:**
- `[app]` - Application metadata and window settings
- `[database]` - Database connection and migration settings
- `[storage]` - File storage and cleanup settings
- `[epub]` - EPUB processing configuration
- `[translation]` - Translation service settings
- `[ui]` - User interface configuration
- `[logging]` - Basic logging settings
- `[performance]` - Performance optimization settings
- `[security]` - Security and validation settings
- `[backup]` - Backup configuration
- `[experimental]` - Experimental features

### 3. `ollama.config.toml` - Ollama Service Configuration
Dedicated configuration for Ollama AI service integration.

**Key Sections:**
- `[connection]` - Connection settings and timeouts
- `[models]` - Model configurations and parameters
- `[translation]` - Translation-specific settings
- `[translation.prompts]` - System prompts for different scenarios
- `[health_check]` - Health monitoring configuration
- `[performance]` - Performance optimization
- `[cache]` - Response caching settings
- `[languages]` - Language-specific configurations

### 4. `reader.config.json` - HTML Reader Configuration
Configuration for the HTML-based ebook reader interface.

**Key Sections:**
- `ui.themes` - Theme definitions (light, dark, sepia)
- `ui.fonts` - Font family configurations
- `navigation` - Keyboard shortcuts and navigation
- `reading` - Reading experience settings
- `accessibility` - Accessibility features
- `performance` - Reader performance settings
- `content` - Content processing and display
- `customization` - Custom CSS/JS support

### 5. `logging.config.toml` - Logging Configuration
Comprehensive logging configuration for all application components.

**Key Sections:**
- `[global]` - Global logging settings
- `[console]` - Console output configuration
- `[file]` - File logging and rotation
- `[modules]` - Module-specific log levels
- `[performance]` - Performance logging
- `[security]` - Security-related logging
- `[audit]` - Audit trail logging

### 6. Environment Files
Environment-specific configurations that override default settings.

**Files:**
- `.env.development` - Development environment settings
- `.env.production` - Production environment settings
- `.env.example` - Template for environment variables

## 🚀 Quick Start

### 1. Basic Setup
```bash
# Copy environment template
cp .env.example .env.development

# Edit configuration files as needed
nano config.toml
nano ollama.config.toml
```

### 2. Environment Selection
The application automatically detects the environment based on:
1. `ENVIRONMENT` environment variable
2. `RUST_ENV` environment variable
3. `NODE_ENV` environment variable
4. Build type (debug/release)

### 3. Configuration Loading Order
Configurations are loaded in this order (later overrides earlier):
1. `config.toml` (base configuration)
2. `ollama.config.toml` (service-specific)
3. `logging.config.toml` (logging-specific)
4. `.env.{environment}` (environment-specific)
5. `user.config.toml` (user preferences)

## ⚙️ Configuration Options

### Application Settings
```toml
[app]
name = "ePub Reader Library"
version = "0.1.0"

[app.window]
width = 1200
height = 800
min_width = 800
min_height = 600
```

### Database Configuration
```toml
[database]
name = "library.db"
max_connections = 10
connection_timeout = 30
auto_migrate = true
```

### Ollama Integration
```toml
[translation.ollama]
host = "localhost"
port = 11434
default_model = "llama3.1:8b"
timeout = 120
max_concurrent_requests = 2
```

### UI Customization
```toml
[ui]
theme = "dark"
language = "en"
books_per_row = 4
enable_animations = true
```

### Performance Tuning
```toml
[performance]
worker_threads = 4
max_memory_usage_mb = 1024
enable_metadata_cache = true
enable_image_cache = true
```

## 🌍 Environment-Specific Configuration

### Development Environment
```bash
# .env.development
ENVIRONMENT=development
DEBUG=true
RUST_LOG=debug
OLLAMA_HOST=localhost
OLLAMA_PORT=11434
```

### Production Environment
```bash
# .env.production
ENVIRONMENT=production
DEBUG=false
RUST_LOG=warn
AUTO_BACKUP=true
CHECK_UPDATES=true
```

## 🔧 Advanced Configuration

### Custom Ollama Models
```toml
[models.configs."custom-model:7b"]
temperature = 0.2
top_p = 0.8
max_tokens = 3000
context_length = 4096
```

### Custom Reader Themes
```json
{
  "ui": {
    "themes": {
      "custom": {
        "name": "Custom Theme",
        "background": "#1e1e1e",
        "text": "#ffffff",
        "accent": "#00ff00"
      }
    }
  }
}
```

### Module-Specific Logging
```toml
[modules.epub_processor]
level = "debug"
log_metadata_extraction = true
log_image_processing = true

[modules.translation]
level = "info"
log_requests = true
log_performance = true
```

## 🔄 Configuration Management

### Hot Reload
Configuration changes are automatically detected and applied without restarting the application (for supported settings).

### Validation
All configurations are validated on startup and reload. Invalid configurations will prevent the application from starting.

### Backup
Configuration files are automatically backed up before changes are applied.

### Migration
Configuration schema migrations are handled automatically when upgrading the application.

## 🛠️ Troubleshooting

### Configuration Not Loading
1. Check file permissions (should be readable)
2. Verify TOML/JSON syntax
3. Check configuration validation errors in logs
4. Ensure required files exist

### Environment Detection Issues
```bash
# Force specific environment
export ENVIRONMENT=development
cargo tauri dev

# Check current environment
echo $ENVIRONMENT
```

### Ollama Configuration Problems
```bash
# Test Ollama connection
curl http://localhost:11434/api/tags

# Check Ollama configuration
cat ollama.config.toml | grep -A 5 "\[connection\]"
```

### Reader Configuration Issues
1. Validate JSON syntax in `reader.config.json`
2. Check theme definitions
3. Verify font family names
4. Test keyboard shortcuts

## 📊 Configuration Examples

### Minimal Configuration
```toml
# config.minimal.toml
[app]
name = "ePub Reader Library"

[database]
name = "library.db"

[translation.ollama]
host = "localhost"
port = 11434
```

### Performance Optimized
```toml
# config.performance.toml
[performance]
worker_threads = 8
max_memory_usage_mb = 2048
enable_metadata_cache = true
enable_image_cache = true

[epub.images]
resize_large_images = true
max_image_width = 1200
convert_to_webp = true

[translation.ollama]
max_concurrent_requests = 4
```

### Security Hardened
```toml
# config.security.toml
[security]
validate_epub_files = true
sanitize_html = true
allow_unsafe_operations = false

[logging]
level = "warn"
log_to_console = false

[backup]
enable_auto_backup = true
compress_backups = true
```

## 🔍 Configuration Reference

### Data Types
- `string` - Text values
- `integer` - Whole numbers
- `float` - Decimal numbers
- `boolean` - true/false values
- `array` - Lists of values
- `table` - Nested configurations

### Environment Variables
All configuration values can be overridden using environment variables with the format:
```
EPUB_READER_SECTION_KEY=value
```

Example:
```bash
export EPUB_READER_DATABASE_MAX_CONNECTIONS=20
export EPUB_READER_UI_THEME=light
```

### Configuration Validation
The application validates configurations against a schema. Common validation errors:
- Missing required fields
- Invalid data types
- Values outside allowed ranges
- Invalid enum values
- Malformed file paths

## 📚 Best Practices

### 1. Environment Separation
- Use different configurations for development, testing, and production
- Keep sensitive data in environment variables
- Use version control for configuration templates

### 2. Performance Tuning
- Adjust worker threads based on CPU cores
- Set appropriate memory limits
- Enable caching for better performance
- Optimize Ollama model parameters

### 3. Security
- Validate all input configurations
- Use secure file permissions
- Enable audit logging in production
- Regularly backup configurations

### 4. Maintenance
- Document custom configurations
- Test configuration changes in development
- Monitor configuration performance impact
- Keep configurations up to date

## 🆘 Support

For configuration-related issues:
1. Check the logs for validation errors
2. Verify file syntax and permissions
3. Test with minimal configuration
4. Consult the troubleshooting section
5. Create an issue with configuration details

---

**Note**: This configuration system is designed to be flexible and extensible. You can add custom configurations by following the established patterns and updating the master configuration file.