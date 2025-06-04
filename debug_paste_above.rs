extern crate tui_textarea;
use tui_textarea::{CursorMove, TextArea};

fn main() {
    println!("Debug: Testing P paste above functionality");
    
    // Create test scenario: cursor at line2, wanting to paste "inserted\nlines" above
    let mut textarea = TextArea::from(["line1", "line2", "line3"]);
    textarea.move_cursor(CursorMove::Jump(1, 2)); // Position at line2, col 2
    println!("Initial: {:?}, cursor at {:?}", textarea.lines(), textarea.cursor());
    
    // Set multi-line yank text
    textarea.set_yank_text("inserted\nlines");
    println!("Yank text: {:?}", textarea.yank_text());
    
    // Test what happens when we just paste at current position
    let mut test1 = textarea.clone();
    test1.paste();
    println!("Direct paste: {:?}", test1.lines());
    
    // Test what happens when we move to head and paste
    let mut test2 = textarea.clone();
    test2.move_cursor(CursorMove::Head);
    println!("After Head move, cursor: {:?}", test2.cursor());
    test2.paste();
    println!("Paste after Head: {:?}", test2.lines());
    
    // Test what happens when we move to head, insert newline, move up, then paste
    let mut test3 = textarea.clone();
    test3.move_cursor(CursorMove::Head);
    test3.insert_newline();
    println!("After inserting newline: {:?}, cursor: {:?}", test3.lines(), test3.cursor());
    test3.move_cursor(CursorMove::Up);
    println!("After moving up: {:?}, cursor: {:?}", test3.lines(), test3.cursor());
    test3.paste();
    println!("Final result: {:?}", test3.lines());
}
