/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate cc;
extern crate glsl_to_cxx;
extern crate webrender_build;

use std::collections::HashSet;
use std::fmt::Write;
use webrender_build::shader::{ShaderFeatureFlags, get_shader_features};

// Shader key is in "name feature,feature" format.
// File name needs to be formatted as "name_feature_feature".
fn shader_file(shader_key: &str) -> String {
    shader_key.replace(' ', "_").replace(',', "_")
}

fn write_load_shader(shader_keys: &[String]) {
    let mut load_shader = String::new();
    for s in shader_keys {
        let _ = write!(load_shader, "#include \"{}.h\"\n", shader_file(s));
    }
    load_shader.push_str("ProgramLoader load_shader(const char* name) {\n");
    for s in shader_keys {
        let _ = write!(load_shader, "  if (!strcmp(name, \"{}\")) {{ return {}_program::loader; }}\n",
                       s, shader_file(s));
    }
    load_shader.push_str("  return nullptr;\n}\n");
    std::fs::write(std::env::var("OUT_DIR").unwrap() + "/load_shader.h", load_shader).unwrap();
}

fn process_imports(shader_dir: &str, shader: &str, included: &mut HashSet<String>, output: &mut String) {
    if !included.insert(shader.into()) {
        return;
    }
    println!("cargo:rerun-if-changed={}/{}.glsl", shader_dir, shader);
    let source = std::fs::read_to_string(format!("{}/{}.glsl", shader_dir, shader)).unwrap();
    for line in source.lines() {
        if line.starts_with("#include ") {
            let imports = line["#include ".len() ..].split(',');
            for import in imports {
                process_imports(shader_dir, import, included, output);
            }
        } else if line.starts_with("#version ") || line.starts_with("#extension ") {
            // ignore
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }
}

fn translate_shader(shader_key: &str, shader_dir: &str) {
    let mut imported = String::from("#define SWGL 1\n");
    let _ = write!(imported, "#define WR_MAX_VERTEX_TEXTURE_WIDTH {}U\n",
                   webrender_build::MAX_VERTEX_TEXTURE_WIDTH);

    let (basename, features) =
        shader_key.split_at(shader_key.find(' ').unwrap_or(shader_key.len()));
    if !features.is_empty() {
        for feature in features.trim().split(',') {
            let _ = write!(imported, "#define WR_FEATURE_{}\n", feature);
        }
    }

    process_imports(shader_dir, basename, &mut HashSet::new(), &mut imported);

    let shader = shader_file(shader_key);

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let imp_name = format!("{}/{}.c", out_dir, shader);
    std::fs::write(&imp_name, imported).unwrap();

    let mut build = cc::Build::new();
    if build.get_compiler().is_like_msvc() {
        build.flag("/EP");
    } else {
        build.flag("-xc").flag("-P");
    }
    build.file(&imp_name);
    let vs = build.clone()
        .define("WR_VERTEX_SHADER", Some("1"))
        .expand();
    let fs = build.clone()
        .define("WR_FRAGMENT_SHADER", Some("1"))
        .expand();
    let vs_name = format!("{}/{}.vert", out_dir, shader);
    let fs_name = format!("{}/{}.frag", out_dir, shader);
    std::fs::write(&vs_name, vs).unwrap();
    std::fs::write(&fs_name, fs).unwrap();

    let mut args = vec![
        "glsl_to_cxx".to_string(),
        vs_name,
        fs_name,
    ];
    let frag_include = format!("{}/{}.frag.h", shader_dir, shader);
    if std::path::Path::new(&frag_include).exists() {
        println!("cargo:rerun-if-changed={}/{}.frag.h", shader_dir, shader);
        args.push(frag_include);
    }
    let result = glsl_to_cxx::translate(&mut args.into_iter());
    std::fs::write(format!("{}/{}.h", out_dir, shader), result).unwrap();
}

fn main() {
    let shader_dir = match std::env::var("MOZ_SRC") {
        Ok(dir) => dir + "/gfx/wr/webrender/res",
        Err(_) => std::env::var("CARGO_MANIFEST_DIR").unwrap() + "/../webrender/res",
    };

    let shader_flags =
        ShaderFeatureFlags::GL |
        ShaderFeatureFlags::DUAL_SOURCE_BLENDING;
    let mut shaders: Vec<String> = Vec::new();
    for (name, features) in get_shader_features(shader_flags) {
        shaders.extend(features.iter().map(|f| {
            if f.is_empty() { name.to_owned() } else { format!("{} {}", name, f) }
        }));
    }

    shaders.sort();

    for shader in &shaders {
        translate_shader(shader, &shader_dir);
    }

    write_load_shader(&shaders);

    println!("cargo:rerun-if-changed=src/gl_defs.h");
    println!("cargo:rerun-if-changed=src/glsl.h");
    println!("cargo:rerun-if-changed=src/program.h");
    println!("cargo:rerun-if-changed=src/texture.h");
    println!("cargo:rerun-if-changed=src/vector_type.h");
    println!("cargo:rerun-if-changed=src/gl.cc");
    cc::Build::new()
        .cpp(true)
        .file("src/gl.cc")
        .flag("-std=c++14")
        .flag("-UMOZILLA_CONFIG_H")
        .flag("-fno-exceptions")
        .flag("-fno-rtti")
        .flag("-fno-math-errno")
        .define("_GLIBCXX_USE_CXX11_ABI", Some("0"))
        .include(shader_dir)
        .include("src")
        .include(std::env::var("OUT_DIR").unwrap())
        .compile("gl_cc");
}
