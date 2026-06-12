use webdriver_traits_codegen::io;

fn main() {
    io(
        vec![
            "cddls/remote.cddl".to_string(),
            "cddls/local.cddl".to_string(),
        ],
        None,
        false,
    )
}
