# Design Document

## Overview

This design outlines the comprehensive internationalization strategy for translating the ePub Reader Library project from Portuguese to English. The approach involves systematic translation of all project components while maintaining functionality, preserving Git history, and creating a well-organized documentation structure.

## Architecture

### Translation Strategy

The internationalization will follow a layered approach:

1. **Documentation Layer**: Translate all markdown files and create organized guides
2. **Code Layer**: Translate comments, strings, and variable names where appropriate
3. **User Interface Layer**: Translate all user-facing text and labels
4. **File System Layer**: Rename files and update references
5. **Repository Layer**: Commit and push changes with proper Git practices

### File Organization Structure

```
epub-reader-library/
├── README.md (translated)
├── guides/
│   ├── README.md (guide index)
│   ├── user-guide.md (from GUIA_DE_USO.md)
│   ├── visual-guide.md (from GUIA_VISUAL.md)
│   ├── quick-start.md (from INICIO_RAPIDO.md)
│   └── configuration.md (from CONFIG.md if needed)
├── src/ (comments translated)
├── assets/ (unchanged)
├── [other files with translated content]
```

## Components and Interfaces

### Documentation Translation Component

**Purpose**: Handle translation of all markdown documentation files

**Responsibilities**:
- Translate README.md maintaining structure and links
- Create guides directory with organized documentation
- Translate technical content while preserving accuracy
- Update internal links and references

**Interface**:
- Input: Portuguese markdown files
- Output: English markdown files with preserved formatting
- Dependencies: File system operations, link validation

### Code Translation Component

**Purpose**: Translate code comments and user-facing strings

**Responsibilities**:
- Translate Rust comments in src/ directory
- Translate JavaScript comments in app.js and related files
- Translate HTML content and attributes
- Update string literals in user interface

**Interface**:
- Input: Source code files with Portuguese content
- Output: Source code files with English content
- Dependencies: Language-specific parsers, syntax preservation

### File Rename Component

**Purpose**: Rename Portuguese files to English equivalents

**Responsibilities**:
- Rename GUIA_DE_USO.md → guides/user-guide.md
- Rename GUIA_VISUAL.md → guides/visual-guide.md  
- Rename INICIO_RAPIDO.md → guides/quick-start.md
- Update all internal references to renamed files

**Interface**:
- Input: File paths and new naming scheme
- Output: Renamed files with updated references
- Dependencies: Git operations, reference tracking

### User Interface Translation Component

**Purpose**: Translate all user-facing text in the application

**Responsibilities**:
- Translate HTML text content
- Translate JavaScript string literals
- Translate error messages and status text
- Translate modal dialog content

**Interface**:
- Input: UI files with Portuguese text
- Output: UI files with English text
- Dependencies: DOM structure preservation, functionality testing

## Data Models

### Translation Mapping

```typescript
interface TranslationMapping {
  sourceFile: string;
  targetFile: string;
  translationType: 'documentation' | 'code' | 'ui' | 'rename';
  preserveFormatting: boolean;
  updateReferences: string[];
}
```

### File Rename Mapping

```typescript
interface FileRenameMapping {
  oldPath: string;
  newPath: string;
  referencingFiles: string[];
  gitOperation: 'move' | 'copy-delete';
}
```

### Translation Context

```typescript
interface TranslationContext {
  technicalTerms: Map<string, string>;
  preservedTerms: string[];
  linkMappings: Map<string, string>;
  codePatterns: RegExp[];
}
```

## Error Handling

### Translation Errors

**File Access Errors**:
- Graceful handling of missing files
- Backup creation before modifications
- Rollback capability for failed operations

**Translation Quality Errors**:
- Validation of technical term consistency
- Preservation of code syntax and functionality
- Link validation after translation

**Git Operation Errors**:
- Proper error messages for commit failures
- Conflict resolution strategies
- Push failure recovery

### Recovery Strategies

1. **Backup and Restore**: Create backups before major operations
2. **Incremental Processing**: Process files in batches with checkpoints
3. **Validation Gates**: Verify each component before proceeding
4. **Rollback Mechanism**: Ability to revert changes if issues occur

## Testing Strategy

### Translation Accuracy Testing

**Documentation Testing**:
- Verify all technical terms are correctly translated
- Ensure code examples remain functional
- Validate all links work correctly
- Check formatting preservation

**Code Testing**:
- Compile and run application after translation
- Verify all functionality remains intact
- Test user interface interactions
- Validate error handling still works

**File System Testing**:
- Verify all file renames completed successfully
- Check all references updated correctly
- Ensure Git history preserved
- Validate directory structure

### Integration Testing

**End-to-End Workflow**:
1. Build and run application
2. Test all major features (add book, translate, read)
3. Verify UI text displays correctly
4. Test error scenarios and messages
5. Validate documentation accessibility

**Repository Testing**:
1. Verify commit includes all changes
2. Test push operation succeeds
3. Validate Git history preservation
4. Check branch integrity

### Quality Assurance

**Translation Quality Metrics**:
- Technical accuracy of translated terms
- Consistency across all files
- Professional tone and clarity
- Preservation of original meaning

**Technical Quality Metrics**:
- Application functionality unchanged
- No broken links or references
- Proper file organization
- Clean Git history

## Implementation Phases

### Phase 1: Preparation and Backup
- Create backup of current state
- Analyze all files requiring translation
- Prepare translation mappings and context
- Set up validation frameworks

### Phase 2: Documentation Translation
- Translate README.md
- Create guides directory structure
- Translate and organize guide files
- Update internal documentation links

### Phase 3: Code Translation
- Translate Rust source comments
- Translate JavaScript comments
- Update HTML content
- Translate configuration files

### Phase 4: User Interface Translation
- Translate UI text and labels
- Update modal dialog content
- Translate error messages
- Update status indicators

### Phase 5: File Organization
- Rename Portuguese files to English
- Update all file references
- Organize guides directory
- Clean up obsolete files

### Phase 6: Validation and Commit
- Run comprehensive tests
- Validate all translations
- Commit changes with proper messages
- Push to remote repository

## Dependencies and Constraints

### External Dependencies
- Git for version control operations
- File system access for file operations
- Text processing capabilities
- Markdown rendering for validation

### Technical Constraints
- Must preserve all existing functionality
- Must maintain Git history where possible
- Must not break any existing integrations
- Must follow English technical writing standards

### Resource Constraints
- Translation must be completed in single session
- All changes must be atomic where possible
- Must minimize disruption to development workflow
- Must provide clear rollback path if needed