/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::PYTHON;
use bindgen::{Builder, CodegenConfig};
use regex::Regex;
use std::cmp;
use std::collections::HashSet;
use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{exit, Command};
use std::slice;
use std::sync::Mutex;
use std::time::SystemTime;
use toml;
use toml::value::Table;

lazy_static! {
    static ref OUTDIR_PATH: PathBuf = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("gecko");
}

const STRUCTS_FILE: &'static str = "structs.rs";

fn read_config(path: &PathBuf) -> Table {
    println!("cargo:rerun-if-changed={}", path.to_str().unwrap());
    update_last_modified(&path);

    let mut contents = String::new();
    File::open(path)
        .expect("Failed to open config file")
        .read_to_string(&mut contents)
        .expect("Failed to read config file");
    match toml::from_str::<Table>(&contents) {
        Ok(result) => result,
        Err(e) => panic!("Failed to parse config file: {}", e),
    }
}

lazy_static! {
    static ref CONFIG: Table = {
        // Load Gecko's binding generator config from the source tree.
        let path = mozbuild::TOPSRCDIR.join("layout/style/ServoBindings.toml");
        read_config(&path)
    };
    static ref BINDGEN_FLAGS: Vec<String> = {
        // Load build-specific config overrides.
        let path = mozbuild::TOPOBJDIR.join("layout/style/extra-bindgen-flags");
        println!("cargo:rerun-if-changed={}", path.to_str().unwrap());
        fs::read_to_string(path).expect("Failed to read extra-bindgen-flags file")
            .split_whitespace()
            .map(std::borrow::ToOwned::to_owned)
            .collect()
    };
    static ref INCLUDE_RE: Regex = Regex::new(r#"#include\s*"(.+?)""#).unwrap();
    static ref DISTDIR_PATH: PathBuf = mozbuild::TOPOBJDIR.join("dist");
    static ref SEARCH_PATHS: Vec<PathBuf> = vec![
        DISTDIR_PATH.join("include"),
        DISTDIR_PATH.join("include/nspr"),
    ];
    static ref ADDED_PATHS: Mutex<HashSet<PathBuf>> = Mutex::new(HashSet::new());
    static ref LAST_MODIFIED: Mutex<SystemTime> =
        Mutex::new(get_modified_time(&env::current_exe().unwrap())
                   .expect("Failed to get modified time of executable"));
}

fn get_modified_time(file: &Path) -> Option<SystemTime> {
    file.metadata().and_then(|m| m.modified()).ok()
}

fn update_last_modified(file: &Path) {
    let modified = get_modified_time(file).expect("Couldn't get file modification time");
    let mut last_modified = LAST_MODIFIED.lock().unwrap();
    *last_modified = cmp::max(modified, *last_modified);
}

fn search_include(name: &str) -> Option<PathBuf> {
    for path in SEARCH_PATHS.iter() {
        let file = path.join(name);
        if file.is_file() {
            update_last_modified(&file);
            return Some(file);
        }
    }
    None
}

fn add_headers_recursively(path: PathBuf, added_paths: &mut HashSet<PathBuf>) {
    if added_paths.contains(&path) {
        return;
    }
    let mut file = File::open(&path).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    added_paths.insert(path);
    // Find all includes and add them recursively
    for cap in INCLUDE_RE.captures_iter(&content) {
        if let Some(path) = search_include(cap.get(1).unwrap().as_str()) {
            add_headers_recursively(path, added_paths);
        }
    }
}

fn add_include(name: &str) -> String {
    let mut added_paths = ADDED_PATHS.lock().unwrap();
    let file = match search_include(name) {
        Some(file) => file,
        None => panic!("Include not found: {}", name),
    };
    let result = String::from(file.to_str().unwrap());
    add_headers_recursively(file, &mut *added_paths);
    result
}

trait BuilderExt {
    fn get_initial_builder() -> Builder;
    fn include<T: Into<String>>(self, file: T) -> Builder;
}

impl BuilderExt for Builder {
    fn get_initial_builder() -> Builder {
        // Disable rust unions, because we replace some types inside of
        // them.
        let mut builder = Builder::default()
            .size_t_is_usize(true)
            .disable_untagged_union();

        let rustfmt_path = env::var_os("RUSTFMT")
            // This can be replaced with
            // > .filter(|p| !p.is_empty()).map(PathBuf::from)
            // once we can use 1.27+.
            .and_then(|p| {
                if p.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(p))
                }
            });
        if let Some(path) = rustfmt_path {
            builder = builder.with_rustfmt(path);
        }

        for dir in SEARCH_PATHS.iter() {
            builder = builder.clang_arg("-I").clang_arg(dir.to_str().unwrap());
        }

        builder = builder.include(add_include("mozilla-config.h"));

        if env::var("CARGO_FEATURE_GECKO_DEBUG").is_ok() {
            builder = builder.clang_arg("-DDEBUG=1").clang_arg("-DJS_DEBUG=1");
        }

        for item in &*BINDGEN_FLAGS {
            builder = builder.clang_arg(item);
        }

        builder
    }
    fn include<T: Into<String>>(self, file: T) -> Builder {
        self.clang_arg("-include").clang_arg(file)
    }
}

struct Fixup {
    pat: String,
    rep: String,
}

fn write_binding_file(builder: Builder, file: &str, fixups: &[Fixup]) {
    let out_file = OUTDIR_PATH.join(file);
    if let Some(modified) = get_modified_time(&out_file) {
        // Don't generate the file if nothing it depends on was modified.
        let last_modified = LAST_MODIFIED.lock().unwrap();
        if *last_modified <= modified {
            return;
        }
    }
    let command_line_opts = builder.command_line_flags();
    let result = builder.generate();
    let mut result = match result {
        Ok(bindings) => bindings.to_string(),
        Err(_) => {
            panic!(
                "Failed to generate bindings, flags: {:?}",
                command_line_opts
            );
        },
    };

    for fixup in fixups.iter() {
        result = Regex::new(&fixup.pat)
            .unwrap()
            .replace_all(&result, &*fixup.rep)
            .into_owned()
            .into();
    }
    let bytes = result.into_bytes();
    File::create(&out_file)
        .unwrap()
        .write_all(&bytes)
        .expect("Unable to write output");
}

struct BuilderWithConfig<'a> {
    builder: Builder,
    config: &'a Table,
    used_keys: HashSet<&'static str>,
}
impl<'a> BuilderWithConfig<'a> {
    fn new(builder: Builder, config: &'a Table) -> Self {
        BuilderWithConfig {
            builder,
            config,
            used_keys: HashSet::new(),
        }
    }

    fn handle_list<F>(self, key: &'static str, func: F) -> BuilderWithConfig<'a>
    where
        F: FnOnce(Builder, slice::Iter<'a, toml::Value>) -> Builder,
    {
        let mut builder = self.builder;
        let config = self.config;
        let mut used_keys = self.used_keys;
        if let Some(list) = config.get(key) {
            used_keys.insert(key);
            builder = func(builder, list.as_array().unwrap().as_slice().iter());
        }
        BuilderWithConfig {
            builder,
            config,
            used_keys,
        }
    }
    fn handle_items<F>(self, key: &'static str, mut func: F) -> BuilderWithConfig<'a>
    where
        F: FnMut(Builder, &'a toml::Value) -> Builder,
    {
        self.handle_list(key, |b, iter| iter.fold(b, |b, item| func(b, item)))
    }
    fn handle_str_items<F>(self, key: &'static str, mut func: F) -> BuilderWithConfig<'a>
    where
        F: FnMut(Builder, &'a str) -> Builder,
    {
        self.handle_items(key, |b, item| func(b, item.as_str().unwrap()))
    }
    fn handle_table_items<F>(self, key: &'static str, mut func: F) -> BuilderWithConfig<'a>
    where
        F: FnMut(Builder, &'a Table) -> Builder,
    {
        self.handle_items(key, |b, item| func(b, item.as_table().unwrap()))
    }
    fn handle_common(self, fixups: &mut Vec<Fixup>) -> BuilderWithConfig<'a> {
        self.handle_str_items("headers", |b, item| b.header(add_include(item)))
            .handle_str_items("raw-lines", |b, item| b.raw_line(item))
            .handle_str_items("hide-types", |b, item| b.blocklist_type(item))
            .handle_table_items("fixups", |builder, item| {
                fixups.push(Fixup {
                    pat: item["pat"].as_str().unwrap().into(),
                    rep: item["rep"].as_str().unwrap().into(),
                });
                builder
            })
    }

    fn get_builder(self) -> Builder {
        for key in self.config.keys() {
            if !self.used_keys.contains(key.as_str()) {
                panic!("Unknown key: {}", key);
            }
        }
        self.builder
    }
}

fn generate_structs() {
    let builder = Builder::get_initial_builder()
        .enable_cxx_namespaces()
        .with_codegen_config(CodegenConfig::TYPES | CodegenConfig::VARS | CodegenConfig::FUNCTIONS);
    let mut fixups = vec![];
    let builder = BuilderWithConfig::new(builder, CONFIG["structs"].as_table().unwrap())
        .handle_common(&mut fixups)
        .handle_str_items("allowlist-functions", |b, item| b.allowlist_function(item))
        .handle_str_items("bitfield-enums", |b, item| b.bitfield_enum(item))
        .handle_str_items("rusty-enums", |b, item| b.rustified_enum(item))
        .handle_str_items("allowlist-vars", |b, item| b.allowlist_var(item))
        .handle_str_items("allowlist-types", |b, item| b.allowlist_type(item))
        .handle_str_items("opaque-types", |b, item| b.opaque_type(item))
        .handle_table_items("cbindgen-types", |b, item| {
            let gecko = item["gecko"].as_str().unwrap();
            let servo = item["servo"].as_str().unwrap();
            b.blocklist_type(format!("mozilla::{}", gecko))
                .module_raw_line("root::mozilla", format!("pub use {} as {};", servo, gecko))
        })
        .handle_table_items("mapped-generic-types", |builder, item| {
            let generic = item["generic"].as_bool().unwrap();
            let gecko = item["gecko"].as_str().unwrap();
            let servo = item["servo"].as_str().unwrap();
            let gecko_name = gecko.rsplit("::").next().unwrap();
            let gecko = gecko
                .split("::")
                .map(|s| format!("\\s*{}\\s*", s))
                .collect::<Vec<_>>()
                .join("::");

            fixups.push(Fixup {
                pat: format!("\\broot\\s*::\\s*{}\\b", gecko),
                rep: format!("crate::gecko_bindings::structs::{}", gecko_name),
            });
            builder.blocklist_type(gecko).raw_line(format!(
                "pub type {0}{2} = {1}{2};",
                gecko_name,
                servo,
                if generic { "<T>" } else { "" }
            ))
        })
        .get_builder();
    write_binding_file(builder, STRUCTS_FILE, &fixups);
}

fn setup_logging() -> bool {
    struct BuildLogger {
        file: Option<Mutex<fs::File>>,
        filter: String,
    }

    impl log::Log for BuildLogger {
        fn enabled(&self, meta: &log::Metadata) -> bool {
            self.file.is_some() && meta.target().contains(&self.filter)
        }

        fn log(&self, record: &log::Record) {
            if !self.enabled(record.metadata()) {
                return;
            }

            let mut file = self.file.as_ref().unwrap().lock().unwrap();
            let _ = writeln!(
                file,
                "{} - {} - {} @ {}:{}",
                record.level(),
                record.target(),
                record.args(),
                record.file().unwrap_or("<unknown>"),
                record.line().unwrap_or(0)
            );
        }

        fn flush(&self) {
            if let Some(ref file) = self.file {
                file.lock().unwrap().flush().unwrap();
            }
        }
    }

    if let Some(path) = env::var_os("STYLO_BUILD_LOG") {
        log::set_max_level(log::LevelFilter::Debug);
        log::set_boxed_logger(Box::new(BuildLogger {
            file: fs::File::create(path).ok().map(Mutex::new),
            filter: env::var("STYLO_BUILD_FILTER")
                .ok()
                .unwrap_or_else(|| "bindgen".to_owned()),
        }))
        .expect("Failed to set logger.");

        true
    } else {
        false
    }
}

fn generate_atoms() {
    let script = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("gecko")
        .join("regen_atoms.py");
    println!("cargo:rerun-if-changed={}", script.display());
    let status = Command::new(&*PYTHON)
        .arg(&script)
        .arg(DISTDIR_PATH.as_os_str())
        .arg(OUTDIR_PATH.as_os_str())
        .status()
        .unwrap();
    if !status.success() {
        exit(1);
    }
}

pub fn generate() {
    println!("cargo:rerun-if-changed=build_gecko.rs");
    fs::create_dir_all(&*OUTDIR_PATH).unwrap();
    setup_logging();
    generate_structs();
    generate_atoms();

    for path in ADDED_PATHS.lock().unwrap().iter() {
        println!("cargo:rerun-if-changed={}", path.to_str().unwrap());
    }
}
