fn main() {
    // This should trigger an error.
    match self.documents.borrow_mut() {
        _ => {}
    }
    // This should trigger an error.
    match self.documents.borrow() {
        _ => {}
    }
    // This should not trigger an error.
    match { self.documents.borrow().find_window(id) } {
        => {}
    }
    // This should not trigger an error.
    match self.documents_status.borrow() {
        => {}
    }
}
