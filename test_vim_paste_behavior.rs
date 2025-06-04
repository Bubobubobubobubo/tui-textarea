extern crate tui_textarea;
use tui_textarea::{CursorMove, TextArea};

fn main() {
    println!("Testing vim-like paste behavior");
    
    // Test 1: Character-wise paste (YankText::Piece)
    println!("\n=== Test 1: Character-wise paste ===");
    let mut textarea = TextArea::from(["hello world"]);
    
    // Position cursor at "w" in "world" 
    textarea.move_cursor(CursorMove::Jump(0, 6));
    println!("Initial: {:?}, cursor at {:?}", textarea.lines(), textarea.cursor());
    
    // Set some character-wise yank text
    textarea.set_yank_text("XYZ");
    
    // Test lowercase 'p' - should paste after cursor
    textarea.move_cursor(CursorMove::Jump(0, 6)); // Reset position
    let mut test_textarea = textarea.clone();
    test_textarea.move_cursor(CursorMove::Forward);
    test_textarea.paste();
    println!("After 'p' (paste after): {:?}, cursor at {:?}", test_textarea.lines(), test_textarea.cursor());
    
    // Test uppercase 'P' - should paste before cursor  
    textarea.move_cursor(CursorMove::Jump(0, 6)); // Reset position
    let mut test_textarea = textarea.clone();
    test_textarea.paste(); // Paste at current position (before cursor)
    println!("After 'P' (paste before): {:?}, cursor at {:?}", test_textarea.lines(), test_textarea.cursor());
    
    // Test 2: Line-wise paste (YankText::Chunk) 
    println!("\n=== Test 2: Line-wise paste ===");
    let mut textarea = TextArea::from(["line1", "line2", "line3"]);
    
    // Position cursor in middle of line2
    textarea.move_cursor(CursorMove::Jump(1, 2));
    println!("Initial: {:?}, cursor at {:?}", textarea.lines(), textarea.cursor());
    
    // Set some line-wise yank text (contains newline)
    textarea.set_yank_text("inserted\nlines");
    
    // Test lowercase 'p' - should paste below current line
    textarea.move_cursor(CursorMove::Jump(1, 2)); // Reset position
    let mut test_textarea = textarea.clone();
    test_textarea.move_cursor(CursorMove::End);
    test_textarea.insert_newline();
    test_textarea.paste();
    println!("After 'p' (paste below line): {:?}", test_textarea.lines());
    
    // Test uppercase 'P' - should paste above current line
    textarea.move_cursor(CursorMove::Jump(1, 2)); // Reset position  
    let mut test_textarea = textarea.clone();
    test_textarea.move_cursor(CursorMove::Head);
    test_textarea.paste();
    println!("After 'P' (paste above line): {:?}", test_textarea.lines());
}
