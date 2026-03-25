use std::path::Path;
use std::{fs, io};

// Keep this in sync with lib.rs
fn main() -> io::Result<()> {
    let resources_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../resources/devtools");
    println!("cargo:rerun-if-changed={}", resources_dir.display());

    let mod_path = resources_dir.join("mod.js");
    let mod_file = fs::read_to_string(&mod_path)?;

    let mut entries = fs::read_dir(resources_dir)?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|entry| entry.path())
        .filter(|entry| entry != &mod_path && entry.extension().is_some_and(|ext| ext == "js"))
        .map(fs::read_to_string)
        .collect::<Result<Vec<_>, _>>()?;
    entries.push(mod_file);

    let out_path = Path::new(&std::env::var("OUT_DIR").unwrap()).join("devtools.js");
    fs::write(out_path, entries.join("\n"))
}
