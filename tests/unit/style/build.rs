fn main() {
    // Dummy build script, just so the code can get env!("OUT_DIR").
    println!("cargo:rerun-if-changed=build.rs");
}
