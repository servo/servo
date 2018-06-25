fn main() {
    println!("Hello, world!");
    for arg in std::env::args() {
        println!("{}", arg);
    }
}
