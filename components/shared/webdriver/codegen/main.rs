mod common;
mod convert;
mod patterns;
mod syntax;
mod util;

use std::{fs, path::Path};

use cddl::{ast::CDDL, cddl_from_str};

use {convert::pattern_map_to_file, patterns::parse_into_patterns, syntax::File};

// fn main() {
//     io(
//         vec![
//             "cddls/remote.cddl".to_string(),
//             "cddls/local.cddl".to_string(),
//         ],
//         None,
//         false,
//     )
// }

pub fn io(inputs: Vec<String>, output: Option<String>, debug: bool) {
    let mut inputs = inputs.iter();
    if let Some(first) = inputs.next() {
        let mut file = cddl_to_rust(first, debug);

        for rest in inputs {
            file.merge(cddl_to_rust(rest, debug));
        }

        match output {
            Some(path) => std::fs::write(path, file.to_string()).unwrap(),
            None => {
                println!("{}", file);
            },
        }
    }
}

fn cddl_to_rust(path: impl AsRef<Path>, debug: bool) -> File {
    let cddl = read_to_cddl(path);
    let parsed = parse_into_patterns(&cddl, debug);
    pattern_map_to_file(parsed)
}

fn read_to_cddl(path: impl AsRef<Path>) -> CDDL<'static> {
    let file = fs::read_to_string(path.as_ref()).unwrap();
    cddl_from_str(file.leak(), false).unwrap()
}
