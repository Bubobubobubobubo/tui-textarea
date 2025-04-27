use syntect::parsing::{SyntaxSet, SyntaxReference};
use syntect::highlighting::{ThemeSet, Theme, Style as SyntectStyle, FontStyle};
use std::sync::Arc;
use crate::ratatui::style::{Color as RatatuiColor, Modifier as RatatuiModifier, Style as RatatuiStyle};

/// Manages syntax highlighting state using syntect.
#[derive(Clone, Debug)] // Add derive for Clone and Debug
pub struct SyntaxHighlighter {
    pub syntax_set: Arc<SyntaxSet>,
    pub theme_set: Arc<ThemeSet>,
}

impl SyntaxHighlighter {
    /// Creates a new SyntaxHighlighter, loading default syntaxes and themes.
    pub fn new() -> Self {
        Self {
            syntax_set: Arc::new(SyntaxSet::load_defaults_newlines()),
            theme_set: Arc::new(ThemeSet::load_defaults()),
        }
    }

    /// Creates a new SyntaxHighlighter from existing SyntaxSet and ThemeSet instances.
    /// Useful for loading custom syntaxes or themes.
    pub fn from_sets(syntax_set: SyntaxSet, theme_set: ThemeSet) -> Self {
        Self {
            syntax_set: Arc::new(syntax_set),
            theme_set: Arc::new(theme_set),
        }
    }

    /// Finds a syntax definition by its name (e.g., "Rust").
    pub fn find_syntax_by_name(&self, name: &str) -> Option<&SyntaxReference> {
        self.syntax_set.find_syntax_by_name(name)
    }

    /// Finds a syntax definition by a file extension (e.g., "rs").
    pub fn find_syntax_by_extension(&self, extension: &str) -> Option<&SyntaxReference> {
        self.syntax_set.find_syntax_by_extension(extension)
    }

    /// Gets a reference to a theme by its name (e.g., "base16-ocean.dark").
    /// The lifetime of the returned reference is tied to the `SyntaxHighlighter`.
    pub fn get_theme(&self, name: &str) -> Option<&Theme> {
        self.theme_set.themes.get(name)
    }

    // TODO: Add methods to:
    // - Highlight lines of text using syntect::easy::HighlightLines
    // - Convert syntect::highlighting::Style to ratatui::style::Style
}

/// Converts a syntect::highlighting::Style to a ratatui::style::Style.
pub(crate) fn syntect_style_to_ratatui(syntect_style: SyntectStyle) -> RatatuiStyle {
    let mut ratatui_style = RatatuiStyle::default();

    // Foreground color
    if syntect_style.foreground.a > 0 { // Check alpha channel for transparency
        ratatui_style.fg = Some(RatatuiColor::Rgb(
            syntect_style.foreground.r,
            syntect_style.foreground.g,
            syntect_style.foreground.b,
        ));
    }

    // --- Background Color Handling --- 
    // Usually we want the terminal's default background or the TextArea's base style background.
    // Applying the theme's background per-span can lead to mismatched background patches.
    // Comment out or remove this section to avoid applying syntect background colors.
    /*
    if syntect_style.background.a > 128 { // Heuristic: Only apply less transparent backgrounds
        ratatui_style.bg = Some(RatatuiColor::Rgb(
            syntect_style.background.r,
            syntect_style.background.g,
            syntect_style.background.b,
        ));
    }
    */
    // --- End Background Color Handling ---

    // Font style modifiers
    if syntect_style.font_style.contains(FontStyle::BOLD) {
        ratatui_style = ratatui_style.add_modifier(RatatuiModifier::BOLD);
    }
    if syntect_style.font_style.contains(FontStyle::ITALIC) {
        ratatui_style = ratatui_style.add_modifier(RatatuiModifier::ITALIC);
    }
    if syntect_style.font_style.contains(FontStyle::UNDERLINE) {
        ratatui_style = ratatui_style.add_modifier(RatatuiModifier::UNDERLINED);
    }

    // Note: syntect also has strikethrough, ratatui might too (check version)
    // If using a newer ratatui version that supports strikethrough:
    // if syntect_style.font_style.contains(FontStyle::STRIKETHROUGH) {
    //     ratatui_style = ratatui_style.add_modifier(RatatuiModifier::CROSSED_OUT); // or RatatuiModifier::STRIKETHROUGH
    // }

    ratatui_style
} 