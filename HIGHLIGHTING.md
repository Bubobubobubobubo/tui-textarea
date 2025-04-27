# Syntax Highlighting

This document explains how to enable and use syntax highlighting in `tui-textarea`.

The syntax highlighting feature leverages the [`syntect`](https://github.com/trishume/syntect) crate to parse code and apply themed styles.

## Enabling Highlighting

1.  **Add Dependency:** Make sure `syntect` is included in your `Cargo.toml`. `tui-textarea` includes it by default unless default features are disabled.
2.  **Create `SyntaxHighlighter`:** Instantiate a `SyntaxHighlighter` instance. This loads the default syntax definitions and themes provided by `syntect`. You can potentially load custom definitions/themes if needed (refer to `syntect` documentation).
3.  **Configure `TextArea`:**
    *   Pass the `SyntaxHighlighter` instance (or a clone of it) to the `TextArea` using `set_syntax_highlighter`.
    *   Specify the syntax definition to use by its name (e.g., "Rust", "Python") using `set_syntax`. You can find available syntax names from `syntect`'s assets or by inspecting the loaded `SyntaxSet`.
    *   Specify the theme to use by its name (e.g., "base16-ocean.dark") using `set_theme`. You can find available theme names similarly.

## Example

```rust
use tui_textarea::{TextArea, SyntaxHighlighter};
# // Dummy imports for example
# use ratatui::style::Style;
# use ratatui::widgets::Block;

// 1. Create the highlighter (loads syntaxes and themes)
let highlighter = SyntaxHighlighter::new();

// 2. Create a TextArea
let mut textarea = TextArea::default(); // Or load content

// 3. Configure highlighting
textarea.set_syntax_highlighter(highlighter); // Provide the sets
textarea.set_syntax(Some("Rust".to_string()));     // Set the language
textarea.set_theme(Some("base16-ocean.dark".to_string())); // Set the theme

// Add some content
textarea.insert_str("struct Example {\n    field: String,\n}");

// Now, when rendered, the textarea will have syntax highlighting
// f.render_widget(&textarea, area);
```

## How it Works

*   The `SyntaxHighlighter` struct holds the `SyntaxSet` (collection of syntax definitions) and `ThemeSet` (collection of themes) wrapped in `Arc` for efficient cloning.
*   The `TextArea` stores the `SyntaxHighlighter` and the *names* of the desired syntax and theme.
*   During rendering (`line_spans` internal method):
    *   If highlighting is configured, the appropriate `SyntaxReference` and `Theme` are looked up using the stored names.
    *   `syntect`'s `HighlightLines` is used to get styled ranges for each line.
    *   A helper function (`syntect_style_to_ratatui`) converts `syntect` styles (foreground color, font style) to `ratatui` styles. **Note:** Background colors from the theme are intentionally ignored to allow the terminal/widget background to show through consistently.
    *   These base styles are then combined (patched) with other styles like cursor line, selection, and search highlights.
    *   The cursor style is applied by *replacing* the style of the character under the cursor to ensure visibility.

## Advanced Usage

*   **Loading Custom Syntaxes/Themes:** While `SyntaxHighlighter::new()` loads bundled defaults, you might want to load custom `.sublime-syntax` or `.tmTheme` files. You can achieve this using `syntect`'s API and the `SyntaxHighlighter::from_sets` constructor.

    *   Use `syntect::parsing::SyntaxSet::load_syntaxes` or `SyntaxSetBuilder` to load custom syntax definitions.
    *   Use `syntect::highlighting::ThemeSet::load_from_folder` or related methods to load custom themes.
    *   Pass the resulting `SyntaxSet` and `ThemeSet` to `SyntaxHighlighter::from_sets`.

    ```rust
    use std::path::Path;
    use tui_textarea::{TextArea, SyntaxHighlighter};
    use syntect::parsing::SyntaxSetBuilder;
    use syntect::highlighting::ThemeSet;
    # // Dummy imports/structs for example
    # fn load_my_syntaxes(builder: &mut SyntaxSetBuilder) { builder.add_from_folder("syntaxes", true).unwrap(); }
    # fn load_my_themes(path: &Path) -> ThemeSet { ThemeSet::load_defaults() }
    # fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Example: Load custom files (paths are illustrative)
    let mut syntax_builder = SyntaxSetBuilder::new();
    // Assuming a function that adds syntaxes from a folder
    load_my_syntaxes(&mut syntax_builder); 
    let syntax_set = syntax_builder.build();

    // Assuming a function that loads themes from a folder
    let theme_set = load_my_themes(Path::new("themes"));

    // Create highlighter from custom sets
    let highlighter = SyntaxHighlighter::from_sets(syntax_set, theme_set);

    // Configure TextArea as usual
    let mut textarea = TextArea::default();
    textarea.set_syntax_highlighter(highlighter);
    textarea.set_syntax(Some("MyCustomSyntax".to_string()));
    textarea.set_theme(Some("MyCustomTheme".to_string())); 
    # Ok(())
    # }
    ```
    Refer to the [`syntect` documentation](https://docs.rs/syntect/latest/syntect/) for detailed information on its loading mechanisms.

*   **Full Example:** See `examples/editor_highlighting.rs` for a runnable example demonstrating theme loading and switching between different syntaxes.
