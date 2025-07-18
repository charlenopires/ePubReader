use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Reading theme model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingTheme {
    pub name: String,
    pub display_name: String,
    pub background_color: String,
    pub text_color: String,
    pub accent_color: String,
    pub link_color: String,
    pub selection_color: String,
    pub highlight_color: String,
    pub note_color: String,
    pub border_color: String,
    pub header_color: String,
    pub properties: ThemeProperties,
}

/// Theme properties for typography and layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeProperties {
    pub font_weight: u16,
    pub letter_spacing: f32,
    pub paragraph_spacing: f32,
    pub shadow_intensity: f32,
    pub brightness: f32,
    pub contrast: f32,
    pub default_font_size: u16,
    pub default_line_height: f32,
}

/// Reading theme preferences model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingThemePreferences {
    pub theme_name: String,
    pub font_family: String,
    pub font_size: u16,
    pub line_height: f32,
    pub brightness: f32,
    pub contrast: f32,
    pub margin_horizontal: u16,
    pub margin_vertical: u16,
    pub reading_width: u16,
    pub two_column_mode: bool,
    pub page_transition_enabled: bool,
    pub page_transition_duration: u16,
    pub auto_scroll_enabled: bool,
    pub auto_scroll_speed: f32,
}

impl Default for ReadingThemePreferences {
    fn default() -> Self {
        Self {
            theme_name: "default".to_string(),
            font_family: "Default".to_string(),
            font_size: 16,
            line_height: 1.5,
            brightness: 1.0,
            contrast: 1.0,
            margin_horizontal: 50,
            margin_vertical: 30,
            reading_width: 800,
            two_column_mode: false,
            page_transition_enabled: true,
            page_transition_duration: 300,
            auto_scroll_enabled: false,
            auto_scroll_speed: 1.0,
        }
    }
}

/// Font family options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FontFamily {
    Default,
    Serif,
    SansSerif,
    Monospace,
    Custom(String),
}

impl FontFamily {
    pub fn to_string(&self) -> String {
        match self {
            FontFamily::Default => "Default".to_string(),
            FontFamily::Serif => "Serif".to_string(),
            FontFamily::SansSerif => "Sans-Serif".to_string(),
            FontFamily::Monospace => "Monospace".to_string(),
            FontFamily::Custom(name) => name.clone(),
        }
    }
    
    pub fn from_string(s: &str) -> Self {
        match s {
            "Default" => FontFamily::Default,
            "Serif" => FontFamily::Serif,
            "Sans-Serif" => FontFamily::SansSerif,
            "Monospace" => FontFamily::Monospace,
            _ => FontFamily::Custom(s.to_string()),
        }
    }
    
    pub fn display_name(&self) -> &str {
        match self {
            FontFamily::Default => "System Default",
            FontFamily::Serif => "Serif",
            FontFamily::SansSerif => "Sans-Serif",
            FontFamily::Monospace => "Monospace",
            FontFamily::Custom(name) => name,
        }
    }
}

/// Theme manager for handling reading themes
pub struct ThemeManager {
    themes: HashMap<String, ReadingTheme>,
    current_preferences: ReadingThemePreferences,
}

impl ThemeManager {
    /// Create a new theme manager with default themes
    pub fn new() -> Self {
        let mut themes = HashMap::new();
        
        // Add predefined themes
        themes.insert("original".to_string(), Self::create_original_theme());
        themes.insert("quiet".to_string(), Self::create_quiet_theme());
        themes.insert("paper".to_string(), Self::create_paper_theme());
        themes.insert("bold".to_string(), Self::create_bold_theme());
        themes.insert("calm".to_string(), Self::create_calm_theme());
        themes.insert("focus".to_string(), Self::create_focus_theme());
        
        Self {
            themes,
            current_preferences: ReadingThemePreferences::default(),
        }
    }
    
    /// Get a theme by name
    pub fn get_theme(&self, name: &str) -> Option<&ReadingTheme> {
        self.themes.get(name)
    }
    
    /// Get all available themes
    pub fn get_all_themes(&self) -> Vec<&ReadingTheme> {
        self.themes.values().collect()
    }
    
    /// Get current reading preferences
    pub fn get_preferences(&self) -> &ReadingThemePreferences {
        &self.current_preferences
    }
    
    /// Update reading preferences
    pub fn update_preferences(&mut self, preferences: ReadingThemePreferences) {
        self.current_preferences = preferences;
    }
    
    /// Switch to a different theme
    pub fn switch_theme(&mut self, theme_name: &str) {
        if self.themes.contains_key(theme_name) {
            self.current_preferences.theme_name = theme_name.to_string();
        }
    }
    
    /// Adjust font size
    pub fn adjust_font_size(&mut self, size: u16) {
        self.current_preferences.font_size = size.clamp(12, 32);
    }
    
    /// Adjust line height
    pub fn adjust_line_height(&mut self, height: f32) {
        self.current_preferences.line_height = height.clamp(1.0, 3.0);
    }
    
    /// Adjust brightness
    pub fn adjust_brightness(&mut self, brightness: f32) {
        self.current_preferences.brightness = brightness.clamp(0.3, 1.5);
    }
    
    /// Adjust contrast
    pub fn adjust_contrast(&mut self, contrast: f32) {
        self.current_preferences.contrast = contrast.clamp(0.5, 2.0);
    }
    
    /// Change font family
    pub fn change_font_family(&mut self, family: &str) {
        self.current_preferences.font_family = family.to_string();
    }
    
    /// Toggle two-column mode
    pub fn toggle_two_column_mode(&mut self) {
        self.current_preferences.two_column_mode = !self.current_preferences.two_column_mode;
    }
    
    /// Adjust margins
    pub fn adjust_margins(&mut self, horizontal: u16, vertical: u16) {
        self.current_preferences.margin_horizontal = horizontal.clamp(20, 120);
        self.current_preferences.margin_vertical = vertical.clamp(20, 80);
    }
    
    /// Adjust reading width
    pub fn adjust_reading_width(&mut self, width: u16) {
        self.current_preferences.reading_width = width.clamp(400, 1200);
    }
    
    /// Get current theme
    pub fn get_current_theme(&self) -> Option<&ReadingTheme> {
        self.get_theme(&self.current_preferences.theme_name)
    }
    
    /// Create Original theme
    fn create_original_theme() -> ReadingTheme {
        ReadingTheme {
            name: "original".to_string(),
            display_name: "Original".to_string(),
            background_color: "#FFFFFF".to_string(),
            text_color: "#000000".to_string(),
            accent_color: "#007AFF".to_string(),
            link_color: "#007AFF".to_string(),
            selection_color: "#B3D9FF".to_string(),
            highlight_color: "#FFD700".to_string(),
            note_color: "#87CEEB".to_string(),
            border_color: "#E0E0E0".to_string(),
            header_color: "#F5F5F5".to_string(),
            properties: ThemeProperties {
                font_weight: 400,
                letter_spacing: 0.0,
                paragraph_spacing: 1.2,
                shadow_intensity: 0.0,
                brightness: 1.0,
                contrast: 1.0,
                default_font_size: 16,
                default_line_height: 1.5,
            },
        }
    }
    
    /// Create Quiet theme
    fn create_quiet_theme() -> ReadingTheme {
        ReadingTheme {
            name: "quiet".to_string(),
            display_name: "Quiet".to_string(),
            background_color: "#F8F8F8".to_string(),
            text_color: "#444444".to_string(),
            accent_color: "#666666".to_string(),
            link_color: "#555555".to_string(),
            selection_color: "#DDDDDD".to_string(),
            highlight_color: "#E8E8E8".to_string(),
            note_color: "#D0D0D0".to_string(),
            border_color: "#CCCCCC".to_string(),
            header_color: "#EEEEEE".to_string(),
            properties: ThemeProperties {
                font_weight: 300,
                letter_spacing: 0.2,
                paragraph_spacing: 1.4,
                shadow_intensity: 0.1,
                brightness: 0.9,
                contrast: 0.8,
                default_font_size: 16,
                default_line_height: 1.6,
            },
        }
    }
    
    /// Create Paper theme
    fn create_paper_theme() -> ReadingTheme {
        ReadingTheme {
            name: "paper".to_string(),
            display_name: "Paper".to_string(),
            background_color: "#F7F3E9".to_string(),
            text_color: "#2F2920".to_string(),
            accent_color: "#8B4513".to_string(),
            link_color: "#A0522D".to_string(),
            selection_color: "#E6D7C3".to_string(),
            highlight_color: "#F0E68C".to_string(),
            note_color: "#DEB887".to_string(),
            border_color: "#D2B48C".to_string(),
            header_color: "#F0EBD8".to_string(),
            properties: ThemeProperties {
                font_weight: 400,
                letter_spacing: 0.3,
                paragraph_spacing: 1.3,
                shadow_intensity: 0.2,
                brightness: 0.95,
                contrast: 0.9,
                default_font_size: 16,
                default_line_height: 1.5,
            },
        }
    }
    
    /// Create Bold theme
    fn create_bold_theme() -> ReadingTheme {
        ReadingTheme {
            name: "bold".to_string(),
            display_name: "Bold".to_string(),
            background_color: "#FFFFFF".to_string(),
            text_color: "#000000".to_string(),
            accent_color: "#FF0000".to_string(),
            link_color: "#0000FF".to_string(),
            selection_color: "#FFFF00".to_string(),
            highlight_color: "#FF6347".to_string(),
            note_color: "#00CED1".to_string(),
            border_color: "#000000".to_string(),
            header_color: "#F0F0F0".to_string(),
            properties: ThemeProperties {
                font_weight: 700,
                letter_spacing: 0.5,
                paragraph_spacing: 1.6,
                shadow_intensity: 0.3,
                brightness: 1.1,
                contrast: 1.3,
                default_font_size: 18,
                default_line_height: 1.8,
            },
        }
    }
    
    /// Create Calm theme
    fn create_calm_theme() -> ReadingTheme {
        ReadingTheme {
            name: "calm".to_string(),
            display_name: "Calm".to_string(),
            background_color: "#F0F8FF".to_string(),
            text_color: "#1E3A8A".to_string(),
            accent_color: "#3B82F6".to_string(),
            link_color: "#2563EB".to_string(),
            selection_color: "#DBEAFE".to_string(),
            highlight_color: "#93C5FD".to_string(),
            note_color: "#BFDBFE".to_string(),
            border_color: "#C3DDFD".to_string(),
            header_color: "#EBF4FF".to_string(),
            properties: ThemeProperties {
                font_weight: 350,
                letter_spacing: 0.1,
                paragraph_spacing: 1.4,
                shadow_intensity: 0.1,
                brightness: 0.95,
                contrast: 0.9,
                default_font_size: 16,
                default_line_height: 1.6,
            },
        }
    }
    
    /// Create Focus theme
    fn create_focus_theme() -> ReadingTheme {
        ReadingTheme {
            name: "focus".to_string(),
            display_name: "Focus".to_string(),
            background_color: "#000000".to_string(),
            text_color: "#FFFFFF".to_string(),
            accent_color: "#00FF00".to_string(),
            link_color: "#00BFFF".to_string(),
            selection_color: "#333333".to_string(),
            highlight_color: "#FFD700".to_string(),
            note_color: "#FF69B4".to_string(),
            border_color: "#444444".to_string(),
            header_color: "#111111".to_string(),
            properties: ThemeProperties {
                font_weight: 400,
                letter_spacing: 0.2,
                paragraph_spacing: 1.5,
                shadow_intensity: 0.0,
                brightness: 1.0,
                contrast: 1.2,
                default_font_size: 16,
                default_line_height: 1.5,
            },
        }
    }
    
    /// Add a custom theme
    pub fn add_custom_theme(&mut self, theme: ReadingTheme) {
        self.themes.insert(theme.name.clone(), theme);
    }
    
    /// Remove a theme
    pub fn remove_theme(&mut self, name: &str) -> Option<ReadingTheme> {
        self.themes.remove(name)
    }
    
    /// Export themes to JSON
    pub fn export_themes(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.themes)
    }
    
    /// Import themes from JSON
    pub fn import_themes(&mut self, json: &str) -> Result<(), serde_json::Error> {
        let themes: HashMap<String, ReadingTheme> = serde_json::from_str(json)?;
        for (name, theme) in themes {
            self.themes.insert(name, theme);
        }
        Ok(())
    }
}


impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Theme event for notifying UI changes
#[derive(Debug, Clone)]
pub enum ThemeEvent {
    ThemeChanged(String),
    FontSizeChanged(u16),
    LineHeightChanged(f32),
    BrightnessChanged(f32),
    ContrastChanged(f32),
    FontFamilyChanged(String),
    TwoColumnModeToggled(bool),
    MarginsChanged(u16, u16),
    ReadingWidthChanged(u16),
    PreferencesUpdated(ReadingThemePreferences),
}

/// Theme validation utilities
impl ReadingTheme {
    /// Validate theme colors
    pub fn validate_colors(&self) -> Result<(), String> {
        let colors = [
            &self.background_color,
            &self.text_color,
            &self.accent_color,
            &self.link_color,
            &self.selection_color,
            &self.highlight_color,
            &self.note_color,
            &self.border_color,
            &self.header_color,
        ];
        
        for color in colors {
            if !Self::is_valid_color(color) {
                return Err(format!("Invalid color format: {}", color));
            }
        }
        
        Ok(())
    }
    
    /// Check if color format is valid
    fn is_valid_color(color: &str) -> bool {
        // Basic hex color validation
        if color.starts_with('#') && color.len() == 7 {
            return color.chars().skip(1).all(|c| c.is_ascii_hexdigit());
        }
        
        // RGB/RGBA validation could be added here
        false
    }
    
    /// Calculate contrast ratio between text and background
    pub fn calculate_contrast_ratio(&self) -> f32 {
        // This is a simplified contrast calculation
        // In a real implementation, you'd use proper color space calculations
        let bg_luminance = Self::hex_to_luminance(&self.background_color);
        let text_luminance = Self::hex_to_luminance(&self.text_color);
        
        let lighter = bg_luminance.max(text_luminance);
        let darker = bg_luminance.min(text_luminance);
        
        (lighter + 0.05) / (darker + 0.05)
    }
    
    /// Convert hex color to relative luminance
    fn hex_to_luminance(hex: &str) -> f32 {
        if hex.len() != 7 || !hex.starts_with('#') {
            return 0.5; // Default fallback
        }
        
        let r = u8::from_str_radix(&hex[1..3], 16).unwrap_or(128) as f32 / 255.0;
        let g = u8::from_str_radix(&hex[3..5], 16).unwrap_or(128) as f32 / 255.0;
        let b = u8::from_str_radix(&hex[5..7], 16).unwrap_or(128) as f32 / 255.0;
        
        // Simplified luminance calculation
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }
}