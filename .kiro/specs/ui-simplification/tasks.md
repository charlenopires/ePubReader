# Implementation Plan

- [ ] 1. Create reading configuration system
  - Create `ReadingConfiguration` struct with font and layout settings
  - Implement `ConfigurationManager` for saving/loading settings
  - Add font management utilities and validation
  - Create configuration file storage in user data directory
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [x] 2. Update UI theme system to dark-only
  - Remove theme selector buttons from main window
  - Update theme system to use dark theme exclusively
  - Remove light and sepia theme definitions
  - Ensure consistent dark theme across all components
  - _Requirements: 2.1, 2.2, 2.3, 2.4_

- [x] 3. Simplify main window header
  - Remove view mode toggle buttons (Grid/List/Large)
  - Remove theme selector buttons (Light/Dark/Sepia)
  - Reorganize header layout with simplified structure
  - Add settings button for accessing reading configuration
  - _Requirements: 1.2, 1.3, 2.2_

- [ ] 4. Enhance book grid component
  - Update BookGrid to be the only view mode
  - Add better error handling for missing books
  - Implement placeholder for books without covers
  - Add error indicators for corrupted or missing files
  - _Requirements: 1.1, 1.4, 4.3, 4.4, 4.5_

- [x] 5. Create settings panel UI
  - Design settings panel component with reading configuration options
  - Add font size slider with range validation
  - Create font family dropdown with available fonts
  - Implement line height and paragraph spacing controls
  - Add margin size adjustment controls
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [ ] 6. Update book loading and display logic
  - Enhance BookService to provide better book display information
  - Add error detection for missing or corrupted book files
  - Implement cover loading with fallback to placeholders
  - Add retry mechanisms for failed book loads
  - _Requirements: 4.1, 4.2, 4.3, 4.4_

- [ ] 7. Implement book display error handling
  - Create error types for different book loading failures
  - Add user-friendly error messages and recovery options
  - Implement retry functionality for recoverable errors
  - Add logging for book loading issues
  - _Requirements: 4.3, 4.4_

- [ ] 8. Update main application integration
  - Modify main.rs to use simplified UI and dark theme by default
  - Integrate reading configuration system
  - Update book loading to use enhanced display information
  - Remove view mode and theme change callbacks
  - _Requirements: 1.1, 2.1, 4.1_

- [ ] 9. Create comprehensive testing
  - Write unit tests for reading configuration system
  - Test book loading with various error scenarios
  - Create UI tests for simplified interface
  - Test settings panel functionality and persistence
  - _Requirements: 1.1, 2.1, 3.4, 4.1_

- [x] 10. Debug and fix book display issues
  - Investigate why loaded books are not showing in the grid
  - Fix any database query issues preventing book display
  - Ensure proper book model conversion for UI display
  - Test with actual book files to verify complete functionality
  - _Requirements: 4.1, 4.2_