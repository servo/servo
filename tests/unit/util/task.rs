use std::borrow::ToOwned;
use util::task::spawn_named;

#[test]
fn spawn_named_test() {
    spawn_named("Test".to_owned(), move || {
        println!("I can run!");
    });
}
