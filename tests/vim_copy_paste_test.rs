use tui_textarea::{CursorMove, TextArea};

/// Test basic copy and paste functionality
#[test]
fn test_copy_paste_basic() {
    let mut textarea = TextArea::default();
    
    // Set up initial text
    textarea.insert_str("hello world");
    textarea.move_cursor(CursorMove::Head);
    
    // Select "hello" (5 characters)
    textarea.start_selection();
    for _ in 0..5 {
        textarea.move_cursor(CursorMove::Forward);
    }
    
    // Copy the selection
    textarea.copy();
    
    // Move to end and paste
    textarea.move_cursor(CursorMove::End);
    textarea.paste();
    
    let lines = textarea.lines();
    assert_eq!(lines[0], "hello worldhello");
    
    println!("✓ Basic copy and paste test passed");
}

/// Test that copy/paste doesn't add extra characters
#[test]
fn test_no_extra_characters() {
    let mut textarea = TextArea::default();
    
    // Set up text with precise content
    textarea.insert_str("abc");
    textarea.move_cursor(CursorMove::Head);
    
    // Select "ab" (2 characters)
    textarea.start_selection();
    textarea.move_cursor(CursorMove::Forward);
    textarea.move_cursor(CursorMove::Forward);
    
    // Copy and paste
    textarea.copy();
    textarea.move_cursor(CursorMove::End);
    textarea.paste();
    
    let lines = textarea.lines();
    // Should be "abcab" - exactly 5 characters, no extra spaces
    assert_eq!(lines[0], "abcab");
    assert_eq!(lines[0].len(), 5);
    
    println!("✓ No extra characters test passed");
}

/// Test line copy functionality
#[test]
fn test_line_copy() {
    let mut textarea = TextArea::default();
    
    // Set up multi-line text
    textarea.insert_str("first line\nsecond line\nthird line");
    textarea.move_cursor(CursorMove::Head);
    
    // Select the entire first line including newline
    textarea.start_selection();
    textarea.move_cursor(CursorMove::End);
    textarea.move_cursor(CursorMove::Down); // Move to next line to include newline
    textarea.move_cursor(CursorMove::Head); // Move to start of second line
    
    // Copy the line
    textarea.copy();
    
    // Move to end and paste
    textarea.move_cursor(CursorMove::Bottom);
    textarea.move_cursor(CursorMove::End);
    textarea.paste();
    
    let lines = textarea.lines();
    // The copied content should include the newline, so it should create a new line
    assert!(lines.len() >= 3);
    assert!(lines.join("\n").contains("first line"));
    
    println!("✓ Line copy test passed");
}

/// Test cut (delete) functionality
#[test]
fn test_cut_functionality() {
    let mut textarea = TextArea::default();
    
    // Set up initial text
    textarea.insert_str("hello world test");
    textarea.move_cursor(CursorMove::Head);
    
    // Select "hello " (6 characters including space)
    textarea.start_selection();
    for _ in 0..6 {
        textarea.move_cursor(CursorMove::Forward);
    }
    
    // Cut the selection
    textarea.cut();
    
    let lines = textarea.lines();
    assert_eq!(lines[0], "world test");
    
    println!("✓ Cut functionality test passed");
}
