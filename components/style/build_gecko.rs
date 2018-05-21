/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

mod common {
    use std::env;
    use std::path::PathBuf;

    lazy_static! {
        pub static ref OUTDIR_PATH: PathBuf =
            PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("gecko");
    }
}

#[cfg(feature = "bindgen")]
mod bindings {
    use bindgen::{Builder, CodegenConfig};
    use regex::Regex;
    use std::cmp;
    use std::collections::{HashMap, HashSet};
    use std::env;
    use std::fs::{self, File};
    use std::io::{Read, Write};
    use std::path::{Path, PathBuf};
    use std::process::{exit, Command};
    use std::slice;
    use std::sync::Mutex;
    use std::time::SystemTime;
    use super::common::*;
    use super::super::PYTHON;
    use toml;
    use toml::value::Table;

    const STRUCTS_FILE: &'static str = "structs.rs";
    const BINDINGS_FILE: &'static str = "bindings.rs";

    fn read_config(path: &PathBuf) -> Table {
        println!("cargo:rerun-if-changed={}", path.to_str().unwrap());
        update_last_modified(&path);

        let mut contents = String::new();
        File::open(path)
            .expect("Failed to open config file")
            .read_to_string(&mut contents)
            .expect("Failed to read config file");
        match toml::from_str::<toml::value::Table>(&contents) {
            Ok(result) => result,
            Err(e) => panic!("Failed to parse config file: {}", e),
        }
    }

    lazy_static! {
        static ref CONFIG: Table = {
            // Load Gecko's binding generator config from the source tree.
            let path = PathBuf::from(env::var_os("MOZ_SRC").unwrap())
                .join("layout/style/ServoBindings.toml");
            read_config(&path)
        };
        static ref BUILD_CONFIG: Table = {
            // Load build-specific config overrides.
            // FIXME: We should merge with CONFIG above instead of
            // forcing callers to do it.
            let path = PathBuf::from(env::var_os("MOZ_TOPOBJDIR").unwrap())
                .join("layout/style/bindgen.toml");
            read_config(&path)
        };
        static ref TARGET_INFO: HashMap<String, String> = {
            const TARGET_PREFIX: &'static str = "CARGO_CFG_TARGET_";
            let mut result = HashMap::new();
            for (k, v) in env::vars() {
                if k.starts_with(TARGET_PREFIX) {
                    result.insert(k[TARGET_PREFIX.len()..].to_lowercase(), v);
                }
            }
            result
        };
        static ref INCLUDE_RE: Regex = Regex::new(r#"#include\s*"(.+?)""#).unwrap();
        static ref DISTDIR_PATH: PathBuf = {
            let path = PathBuf::from(env::var_os("MOZ_DIST").unwrap());
            if !path.is_absolute() || !path.is_dir() {
                panic!("MOZ_DIST must be an absolute directory, was: {}", path.display());
            }
            path
        };
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
        println!("cargo:rerun-if-changed={}", path.to_str().unwrap());
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
        let file = search_include(name).expect("Include not found!");
        let result = String::from(file.to_str().unwrap());
        add_headers_recursively(file, &mut *added_paths);
        result
    }

    trait BuilderExt {
        fn get_initial_builder() -> Builder;
        fn include<T: Into<String>>(self, file: T) -> Builder;
        fn zero_size_type(self, ty: &str, structs_list: &HashSet<&str>) -> Builder;
        fn borrowed_type(self, ty: &str) -> Builder;
        fn mutable_borrowed_type(self, ty: &str) -> Builder;
    }

    fn add_clang_args(mut builder: Builder, config: &Table, matched_os: &mut bool) -> Builder {
        fn add_args(mut builder: Builder, values: &[toml::Value]) -> Builder {
            for item in values.iter() {
                builder = builder.clang_arg(item.as_str().expect("Expect string in list"));
            }
            builder
        }
        for (k, v) in config.iter() {
            if k == "args" {
                builder = add_args(builder, v.as_array().unwrap().as_slice());
                continue;
            }
            let equal_idx = k.find('=').expect(&format!("Invalid key: {}", k));
            let (target_type, target_value) = k.split_at(equal_idx);
            if TARGET_INFO[target_type] != target_value[1..] {
                continue;
            }
            if target_type == "os" {
                *matched_os = true;
            }
            builder = match *v {
                toml::Value::Table(ref table) => add_clang_args(builder, table, matched_os),
                toml::Value::Array(ref array) => add_args(builder, array),
                _ => panic!("Unknown type"),
            };
        }
        builder
    }

    impl BuilderExt for Builder {
        fn get_initial_builder() -> Builder {
            use bindgen::RustTarget;

            // Disable rust unions, because we replace some types inside of
            // them.
            let mut builder = Builder::default().rust_target(RustTarget::Stable_1_0);
            let rustfmt_path = env::var_os("MOZ_AUTOMATION")
                .and_then(|_| env::var_os("TOOLTOOL_DIR").or_else(|| env::var_os("MOZ_SRC")))
                .map(PathBuf::from);

            builder = match rustfmt_path {
                Some(path) => builder.with_rustfmt(path.join("rustc").join("bin").join("rustfmt")),
                None => builder.rustfmt_bindings(env::var_os("STYLO_RUSTFMT_BINDINGS").is_some()),
            };

            for dir in SEARCH_PATHS.iter() {
                builder = builder.clang_arg("-I").clang_arg(dir.to_str().unwrap());
            }

            builder = builder.include(add_include("mozilla-config.h"));

            if env::var("CARGO_FEATURE_GECKO_DEBUG").is_ok() {
                builder = builder.clang_arg("-DDEBUG=1").clang_arg("-DJS_DEBUG=1");
            }

            let mut matched_os = false;
            let build_config = CONFIG["build"].as_table().expect("Malformed config file");
            builder = add_clang_args(builder, build_config, &mut matched_os);
            let build_config = BUILD_CONFIG["build"]
                .as_table()
                .expect("Malformed config file");
            builder = add_clang_args(builder, build_config, &mut matched_os);
            if !matched_os {
                panic!("Unknown platform");
            }
            builder
        }
        fn include<T: Into<String>>(self, file: T) -> Builder {
            self.clang_arg("-include").clang_arg(file)
        }
        // This makes an FFI-safe void type that can't be matched on
        // &VoidType is UB to have, because you can match on it
        // to produce a reachable unreachable. If it's wrapped in
        // a struct as a private field it becomes okay again
        //
        // Not 100% sure of how safe this is, but it's what we're using
        // in the XPCOM ffi too
        // https://github.com/nikomatsakis/rust-memory-model/issues/2
        fn zero_size_type(self, ty: &str, structs_list: &HashSet<&str>) -> Builder {
            if !structs_list.contains(ty) {
                self.blacklist_type(ty)
                    .raw_line(format!("enum {}Void {{ }}", ty))
                    .raw_line(format!("pub struct {0}({0}Void);", ty))
            } else {
                self
            }
        }
        fn borrowed_type(self, ty: &str) -> Builder {
            self.blacklist_type(format!("{}Borrowed", ty))
                .raw_line(format!("pub type {0}Borrowed<'a> = &'a {0};", ty))
                .blacklist_type(format!("{}BorrowedOrNull", ty))
                .raw_line(format!(
                    "pub type {0}BorrowedOrNull<'a> = Option<&'a {0}>;",
                    ty
                ))
        }
        fn mutable_borrowed_type(self, ty: &str) -> Builder {
            self.borrowed_type(ty)
                .blacklist_type(format!("{}BorrowedMut", ty))
                .raw_line(format!("pub type {0}BorrowedMut<'a> = &'a mut {0};", ty))
                .blacklist_type(format!("{}BorrowedMutOrNull", ty))
                .raw_line(format!(
                    "pub type {0}BorrowedMutOrNull<'a> = Option<&'a mut {0}>;",
                    ty
                ))
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

    fn get_arc_types() -> Vec<String> {
        // Read the file
        let mut list_file = File::open(DISTDIR_PATH.join("include/mozilla/ServoArcTypeList.h"))
            .expect("Unable to open ServoArcTypeList.h");
        let mut content = String::new();
        list_file
            .read_to_string(&mut content)
            .expect("Fail to read ServoArcTypeList.h");
        // Remove comments
        let block_comment_re = Regex::new(r#"(?s)/\*.*?\*/"#).unwrap();
        let content = block_comment_re.replace_all(&content, "");
        // Extract the list
        let re = Regex::new(r#"^SERVO_ARC_TYPE\(\w+,\s*(\w+)\)$"#).unwrap();
        content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .map(|line| {
                re.captures(&line)
                    .expect(&format!(
                        "Unrecognized line in ServoArcTypeList.h: '{}'",
                        line
                    ))
                    .get(1)
                    .unwrap()
                    .as_str()
                    .to_string()
            })
            .collect()
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
                .handle_str_items("hide-types", |b, item| b.blacklist_type(item))
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
                    panic!(format!("Unknown key: {}", key));
                }
            }
            self.builder
        }
    }

    fn generate_structs() {
        let builder = Builder::get_initial_builder()
            .enable_cxx_namespaces()
            .with_codegen_config(CodegenConfig {
                types: true,
                vars: true,
                ..CodegenConfig::nothing()
            });
        let mut fixups = vec![];
        let builder = BuilderWithConfig::new(builder, CONFIG["structs"].as_table().unwrap())
            .handle_common(&mut fixups)
            .handle_str_items("bitfield-enums", |b, item| b.bitfield_enum(item))
            .handle_str_items("rusty-enums", |b, item| b.rustified_enum(item))
            .handle_str_items("whitelist-vars", |b, item| b.whitelist_var(item))
            .handle_str_items("whitelist-types", |b, item| b.whitelist_type(item))
            .handle_str_items("opaque-types", |b, item| b.opaque_type(item))
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
                    rep: format!("::gecko_bindings::structs::{}", gecko_name),
                });
                builder.blacklist_type(gecko).raw_line(format!(
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
        use log;

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
            })).expect("Failed to set logger.");

            true
        } else {
            false
        }
    }

    fn generate_bindings() {
        let builder = Builder::get_initial_builder()
            .disable_name_namespacing()
            .with_codegen_config(CodegenConfig {
                functions: true,
                ..CodegenConfig::nothing()
            });
        let config = CONFIG["bindings"].as_table().unwrap();
        let mut structs_types = HashSet::new();
        let mut fixups = vec![];
        let mut builder = BuilderWithConfig::new(builder, config)
            .handle_common(&mut fixups)
            .handle_str_items("whitelist-functions", |b, item| b.whitelist_function(item))
            .handle_str_items("structs-types", |mut builder, ty| {
                builder = builder.blacklist_type(ty)
                    .raw_line(format!("use gecko_bindings::structs::{};", ty));
                structs_types.insert(ty);
                // TODO this is hacky, figure out a better way to do it without
                // hardcoding everything...
                if ty.starts_with("nsStyle") {
                    builder = builder
                        .raw_line(format!("unsafe impl Send for {} {{}}", ty))
                        .raw_line(format!("unsafe impl Sync for {} {{}}", ty));
                }
                builder
            })
            // TODO This was added due to servo/rust-bindgen#75, but
            // that has been fixed in clang 4.0+. When we switch people
            // to libclang 4.0, we can remove this.
            .handle_table_items("array-types", |builder, item| {
                let cpp_type = item["cpp-type"].as_str().unwrap();
                let rust_type = item["rust-type"].as_str().unwrap();
                builder
                    .raw_line(format!(concat!("pub type nsTArrayBorrowed_{}<'a> = ",
                                              "&'a mut ::gecko_bindings::structs::nsTArray<{}>;"),
                                      cpp_type, rust_type))
            })
            .handle_table_items("servo-owned-types", |mut builder, item| {
                let name = item["name"].as_str().unwrap();
                builder = builder.blacklist_type(format!("{}Owned", name))
                    .raw_line(format!("pub type {0}Owned = ::gecko_bindings::sugar::ownership::Owned<{0}>;", name))
                    .blacklist_type(format!("{}OwnedOrNull", name))
                    .raw_line(format!(concat!("pub type {0}OwnedOrNull = ",
                                              "::gecko_bindings::sugar::ownership::OwnedOrNull<{0}>;"), name))
                    .mutable_borrowed_type(name);
                if item["opaque"].as_bool().unwrap() {
                    builder = builder.zero_size_type(name, &structs_types);
                }
                builder
            })
            .handle_str_items("servo-immutable-borrow-types", |b, ty| b.borrowed_type(ty))
            // Right now the only immutable borrow types are ones which we import
            // from the |structs| module. As such, we don't need to create an opaque
            // type with zero_size_type. If we ever introduce immutable borrow types
            // which _do_ need to be opaque, we'll need a separate mode.
            .handle_str_items("servo-borrow-types", |b, ty| b.mutable_borrowed_type(ty))
            .get_builder();
        for ty in get_arc_types().iter() {
            builder = builder
                .blacklist_type(format!("{}Strong", ty))
                .raw_line(format!(
                    "pub type {0}Strong = ::gecko_bindings::sugar::ownership::Strong<{0}>;",
                    ty
                ))
                .borrowed_type(ty)
                .zero_size_type(ty, &structs_types);
        }
        write_binding_file(builder, BINDINGS_FILE, &fixups);
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
        use std::thread;
        macro_rules! run_tasks {
            ($($task:expr,)+) => {
                if setup_logging() {
                    $($task;)+
                } else {
                    let threads = vec![$( thread::spawn(|| $task) ),+];
                    for thread in threads.into_iter() {
                        thread.join().unwrap();
                    }
                }
            }
        }
        run_tasks! {
            generate_structs(),
            generate_bindings(),
            generate_atoms(),
        }
    }
}

#[cfg(not(feature = "bindgen"))]
mod bindings {
    use std::{env, fs, io};
    use std::path::{Path, PathBuf};
    use super::common::*;

    /// Copy contents of one directory into another.
    /// It currently only does a shallow copy.
    fn copy_dir<P, Q, F>(from: P, to: Q, callback: F) -> io::Result<()>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
        F: Fn(&Path),
    {
        let to = to.as_ref();
        for entry in from.as_ref().read_dir()? {
            let entry = entry?;
            let path = entry.path();
            callback(&path);
            fs::copy(&path, to.join(entry.file_name()))?;
        }
        Ok(())
    }

    pub fn generate() {
        let dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("gecko/generated");
        println!("cargo:rerun-if-changed={}", dir.display());
        copy_dir(&dir, &*OUTDIR_PATH, |path| {
            println!("cargo:rerun-if-changed={}", path.display());
        }).expect("Fail to copy generated files to out dir");
    }
}

pub fn generate() {
    use self::common::*;
    use std::fs;
    println!("cargo:rerun-if-changed=build_gecko.rs");
    fs::create_dir_all(&*OUTDIR_PATH).unwrap();
    bindings::generate();
}
