use tui_textarea::{TextArea, CursorMove};

fn main() {
    println!("Testing vim-mode copy/paste behavior...");

    // Test 1: Basic text selection and copy
    let mut textarea = TextArea::from([
        "Hello world",
        "This is a test",
        "Another line"
    ]);
    
    // Select "world" in visual mode
    textarea.move_cursor(CursorMove::Jump(0, 6)); // Move to 'w' in "world"
    textarea.start_selection();
    textarea.move_cursor(CursorMove::Jump(0, 11)); // Move to end of "world"
    textarea.copy();
    
    println!("Yanked text: '{}'", textarea.yank_text());
    assert_eq!(textarea.yank_text(), "world");
    
    // Test 2: Paste at different position
    textarea.move_cursor(CursorMove::Jump(1, 5)); // Move to after "This "
    textarea.paste();
    
    println!("After paste: {:?}", textarea.lines());
    
    // Test 3: Line yank simulation (yy command)
    let mut textarea2 = TextArea::from([
        "Line 1",
        "Line 2", 
        "Line 3"
    ]);
    
    // Simulate yy command
    textarea2.move_cursor(CursorMove::Jump(1, 0)); // Move to line 2
    textarea2.move_cursor(CursorMove::Head);
    textarea2.start_selection();
    let cursor = textarea2.cursor();
    textarea2.move_cursor(CursorMove::Down);
    if cursor == textarea2.cursor() {
        // At the last line, select to end of line
        textarea2.move_cursor(CursorMove::End);
    } else {
        // Move to beginning of next line to include the newline
        textarea2.move_cursor(CursorMove::Head);
    }
    textarea2.copy();
    
    println!("Line yanked: '{}'", textarea2.yank_text());
    
    // Paste the line
    textarea2.move_cursor(CursorMove::Jump(0, 0));
    textarea2.paste();
    
    println!("After line paste: {:?}", textarea2.lines());
    
    println!("All tests completed!");
}
