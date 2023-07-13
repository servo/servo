/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate webrender_build;

use std::borrow::Cow;
use std::env;
use std::fs::{canonicalize, read_dir, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use webrender_build::shader::*;
use webrender_build::shader_features::{ShaderFeatureFlags, get_shader_features};

// glsopt is known to leak, but we don't particularly care.
#[no_mangle]
pub extern "C" fn __lsan_default_options() -> *const u8 {
    b"detect_leaks=0\0".as_ptr()
}

/// Compute the shader path for insertion into the include_str!() macro.
/// This makes for more compact generated code than inserting the literal
/// shader source into the generated file.
///
/// If someone is building on a network share, I'm sorry.
fn escape_include_path(path: &Path) -> String {
    let full_path = canonicalize(path).unwrap();
    let full_name = full_path.as_os_str().to_str().unwrap();
    let full_name = full_name.replace("\\\\?\\", "");
    let full_name = full_name.replace("\\", "/");

    full_name
}

fn write_unoptimized_shaders(mut glsl_files: Vec<PathBuf>, shader_file: &mut File) -> Result<(), std::io::Error> {
    writeln!(
        shader_file,
        "  pub static ref UNOPTIMIZED_SHADERS: HashMap<&'static str, SourceWithDigest> = {{"
    )?;
    writeln!(shader_file, "    let mut shaders = HashMap::new();")?;

    // Sort the file list so that the shaders.rs file is filled
    // deterministically.
    glsl_files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    for glsl in glsl_files {
        // Compute the shader name.
        assert!(glsl.is_file());
        let shader_name = glsl.file_name().unwrap().to_str().unwrap();
        let shader_name = shader_name.replace(".glsl", "");

        // Compute a digest of the #include-expanded shader source. We store
        // this as a literal alongside the source string so that we don't need
        // to hash large strings at runtime.
        let mut hasher = DefaultHasher::new();
        let base = glsl.parent().unwrap();
        assert!(base.is_dir());
        ShaderSourceParser::new().parse(
            Cow::Owned(shader_source_from_file(&glsl)),
            &|f| Cow::Owned(shader_source_from_file(&base.join(&format!("{}.glsl", f)))),
            &mut |s| hasher.write(s.as_bytes()),
        );
        let digest: ProgramSourceDigest = hasher.into();

        writeln!(
            shader_file,
            "    shaders.insert(\"{}\", SourceWithDigest {{ source: include_str!(\"{}\"), digest: \"{}\"}});",
            shader_name,
            escape_include_path(&glsl),
            digest,
        )?;
    }
    writeln!(shader_file, "    shaders")?;
    writeln!(shader_file, "  }};")?;

    Ok(())
}

#[derive(Clone, Debug)]
struct ShaderOptimizationInput {
    shader_name: &'static str,
    config: String,
    gl_version: ShaderVersion,
}

#[derive(Debug)]
struct ShaderOptimizationOutput {
    full_shader_name: String,
    gl_version: ShaderVersion,
    vert_file_path: PathBuf,
    frag_file_path: PathBuf,
    digest: ProgramSourceDigest,
}

#[derive(Debug)]
struct ShaderOptimizationError {
    shader: ShaderOptimizationInput,
    message: String,
}

fn print_shader_source(shader_src: &str) {
    // For some reason the glsl-opt errors are offset by 1 compared
    // to the provided shader source string.
    println!("0\t|");
    for (n, line) in shader_src.split('\n').enumerate() {
        let line_number = n + 1;
        println!("{}\t|{}", line_number, line);
    }
}

fn write_optimized_shaders(shader_dir: &Path, shader_file: &mut File, out_dir: &str) -> Result<(), std::io::Error> {
    writeln!(
        shader_file,
        "  pub static ref OPTIMIZED_SHADERS: HashMap<(ShaderVersion, &'static str), OptimizedSourceWithDigest> = {{"
    )?;
    writeln!(shader_file, "    let mut shaders = HashMap::new();")?;

    // The full set of optimized shaders can be quite large, so only optimize
    // for the GL version we expect to be used on the target platform. If a different GL
    // version is used we will simply fall back to the unoptimized shaders.
    let shader_versions = match env::var("CARGO_CFG_TARGET_OS").as_ref().map(|s| &**s) {
        Ok("android") | Ok("windows") => [ShaderVersion::Gles],
        _ => [ShaderVersion::Gl],
    };

    let mut shaders = Vec::default();
    for &gl_version in &shader_versions {
        let mut flags = ShaderFeatureFlags::all();
        if gl_version != ShaderVersion::Gl {
            flags.remove(ShaderFeatureFlags::GL);
        }
        if gl_version != ShaderVersion::Gles {
            flags.remove(ShaderFeatureFlags::GLES);
            flags.remove(ShaderFeatureFlags::TEXTURE_EXTERNAL);
        }
        if !matches!(env::var("CARGO_CFG_TARGET_OS").as_ref().map(|s| &**s), Ok("android")) {
            flags.remove(ShaderFeatureFlags::TEXTURE_EXTERNAL_ESSL1);
        }
        flags.remove(ShaderFeatureFlags::DITHERING);

        for (shader_name, configs) in get_shader_features(flags) {
            for config in configs {
                shaders.push(ShaderOptimizationInput {
                    shader_name,
                    config,
                    gl_version,
                });
            }
        }
    }

    let outputs = build_parallel::compile_objects(&|shader: &ShaderOptimizationInput| {
        println!("Optimizing shader {:?}", shader);
        let target = match shader.gl_version {
            ShaderVersion::Gl => glslopt::Target::OpenGl,
            ShaderVersion::Gles => glslopt::Target::OpenGles30,
        };
        let glslopt_ctx = glslopt::Context::new(target);

        let features = shader.config.split(",").filter(|f| !f.is_empty()).collect::<Vec<_>>();

        let (vert_src, frag_src) = build_shader_strings(
            shader.gl_version,
            &features,
            shader.shader_name,
            &|f| Cow::Owned(shader_source_from_file(&shader_dir.join(&format!("{}.glsl", f)))),
        );

        let full_shader_name = if shader.config.is_empty() {
            shader.shader_name.to_string()
        } else {
            format!("{}_{}", shader.shader_name, shader.config.replace(",", "_"))
        };

        let vert = glslopt_ctx.optimize(glslopt::ShaderType::Vertex, vert_src.clone());
        if !vert.get_status() {
            print_shader_source(&vert_src);
            return Err(ShaderOptimizationError {
                shader: shader.clone(),
                message: vert.get_log().to_string(),
            });
        }
        let frag = glslopt_ctx.optimize(glslopt::ShaderType::Fragment, frag_src.clone());
        if !frag.get_status() {
            print_shader_source(&frag_src);
            return Err(ShaderOptimizationError {
                shader: shader.clone(),
                message: frag.get_log().to_string(),
            });
        }

        let vert_source = vert.get_output().unwrap();
        let frag_source = frag.get_output().unwrap();

        // Compute a digest of the optimized shader sources. We store this
        // as a literal alongside the source string so that we don't need
        // to hash large strings at runtime.
        let mut hasher = DefaultHasher::new();

        let vert_file_path = Path::new(out_dir)
            .join(format!("{}_{:?}.vert", full_shader_name, shader.gl_version));
        write_optimized_shader_file(&vert_file_path, vert_source, &shader.shader_name, &features, &mut hasher);

        let frag_file_path = vert_file_path.with_extension("frag");
        write_optimized_shader_file(&frag_file_path, frag_source, &shader.shader_name, &features, &mut hasher);

        let digest: ProgramSourceDigest = hasher.into();

        println!("Finished optimizing shader {:?}", shader);

        Ok(ShaderOptimizationOutput {
            full_shader_name,
            gl_version: shader.gl_version,
            vert_file_path,
            frag_file_path,
            digest,
        })
    }, &shaders);

    match outputs {
        Ok(mut outputs) => {
            // Sort the shader list so that the shaders.rs file is filled
            // deterministically.
            outputs.sort_by(|a, b| {
                (a.gl_version, a.full_shader_name.clone()).cmp(&(b.gl_version, b.full_shader_name.clone()))
            });

            for shader in outputs {
                writeln!(
                    shader_file,
                    "    shaders.insert(({}, \"{}\"), OptimizedSourceWithDigest {{",
                    shader.gl_version.variant_name(),
                    shader.full_shader_name,
                )?;
                writeln!(
                    shader_file,
                    "        vert_source: include_str!(\"{}\"),",
                    escape_include_path(&shader.vert_file_path),
                )?;
                writeln!(
                    shader_file,
                    "        frag_source: include_str!(\"{}\"),",
                    escape_include_path(&shader.frag_file_path),
                )?;
                writeln!(shader_file, "        digest: \"{}\",", shader.digest)?;
                writeln!(shader_file, "    }});")?;
            }
        }
        Err(err) => match err {
            build_parallel::Error::BuildError(err) => {
                panic!("Error optimizing shader {:?}: {}", err.shader, err.message)
            }
            _ => panic!("Error optimizing shaders."),
        }
    }

    writeln!(shader_file, "    shaders")?;
    writeln!(shader_file, "  }};")?;

    Ok(())
}

fn write_optimized_shader_file(
    path: &Path,
    source: &str,
    shader_name: &str,
    features: &[&str],
    hasher: &mut DefaultHasher,
) {
    let mut file = File::create(&path).unwrap();
    for (line_number, line) in source.lines().enumerate() {
        // We embed the shader name and features as a comment in the
        // source to make debugging easier.
        // The #version directive must be on the first line so we insert
        // the extra information on the next line.
        if line_number == 1 {
            let prelude = format!(
                "// {}\n// features: {:?}\n\n",
                shader_name, features
            );
            file.write_all(prelude.as_bytes()).unwrap();
            hasher.write(prelude.as_bytes());
        }
        file.write_all(line.as_bytes()).unwrap();
        file.write_all("\n".as_bytes()).unwrap();
        hasher.write(line.as_bytes());
        hasher.write("\n".as_bytes());
    }
}

fn main() -> Result<(), std::io::Error> {
    let out_dir = env::var("OUT_DIR").unwrap_or("out".to_owned());

    let shaders_file_path = Path::new(&out_dir).join("shaders.rs");
    let mut glsl_files = vec![];

    println!("cargo:rerun-if-changed=res");
    let res_dir = Path::new("res");
    for entry in read_dir(res_dir)? {
        let entry = entry?;
        let path = entry.path();

        if entry.file_name().to_str().unwrap().ends_with(".glsl") {
            println!("cargo:rerun-if-changed={}", path.display());
            glsl_files.push(path.to_owned());
        }
    }

    let mut shader_file = File::create(shaders_file_path)?;

    writeln!(shader_file, "/// AUTO GENERATED BY build.rs\n")?;
    writeln!(shader_file, "use std::collections::HashMap;\n")?;
    writeln!(shader_file, "use webrender_build::shader::ShaderVersion;\n")?;
    writeln!(shader_file, "pub struct SourceWithDigest {{")?;
    writeln!(shader_file, "    pub source: &'static str,")?;
    writeln!(shader_file, "    pub digest: &'static str,")?;
    writeln!(shader_file, "}}\n")?;
    writeln!(shader_file, "pub struct OptimizedSourceWithDigest {{")?;
    writeln!(shader_file, "    pub vert_source: &'static str,")?;
    writeln!(shader_file, "    pub frag_source: &'static str,")?;
    writeln!(shader_file, "    pub digest: &'static str,")?;
    writeln!(shader_file, "}}\n")?;
    writeln!(shader_file, "lazy_static! {{")?;

    write_unoptimized_shaders(glsl_files, &mut shader_file)?;
    writeln!(shader_file, "")?;
    write_optimized_shaders(&res_dir, &mut shader_file, &out_dir)?;
    writeln!(shader_file, "}}")?;

    Ok(())
}
