use std::env;

fn main() {
    /*let target = env::var("TARGET").unwrap();
    let host = env::var("HOST").unwrap();
    let mut c_cfg = cc::Build::new();
    c_cfg
        .cargo_metadata(false)
        .opt_level(0)
        .debug(false)
        .target(&target)
        .warnings(false)
        .host(&host);
    let mut cxx_cfg = cc::Build::new();
    cxx_cfg
        .cargo_metadata(false)
        .cpp(true)
        .opt_level(0)
        .debug(false)
        .target(&target)
        .warnings(false)
        .host(&host);
    let c_compiler = c_cfg.get_compiler();
    let cxx_compiler = cxx_cfg.get_compiler();
    panic!("{:?} {:?}", c_compiler, cxx_compiler);*/

        let _dst = cmake::Config::new("src")
        .define("BUILD_shared", "OFF")
        .define("BUILD_tools", "OFF")
        .define("BUILD_examples", "OFF")
        .define("BUILD_tests", "OFF")
        .build();
    panic!()
}
