/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Functionality for managing source code for shaders.
//!
//! This module is used during precompilation (build.rs) and regular compilation,
//! so it has minimal dependencies.

use std::borrow::Cow;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::collections::HashSet;
use std::collections::hash_map::DefaultHasher;
use crate::MAX_VERTEX_TEXTURE_WIDTH;

pub use crate::shader_features::*;

lazy_static! {
    static ref MAX_VERTEX_TEXTURE_WIDTH_STRING: String = MAX_VERTEX_TEXTURE_WIDTH.to_string();
}

#[derive(Clone, Copy, Debug)]
pub enum ShaderKind {
    Vertex,
    Fragment,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ShaderVersion {
    Gl,
    Gles,
}

impl ShaderVersion {
    /// Return the full variant name, for use in code generation.
    pub fn variant_name(&self) -> &'static str {
        match self {
            ShaderVersion::Gl => "ShaderVersion::Gl",
            ShaderVersion::Gles => "ShaderVersion::Gles",
        }
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Default)]
#[cfg_attr(feature = "serialize_program", derive(Deserialize, Serialize))]
pub struct ProgramSourceDigest(u64);

impl ::std::fmt::Display for ProgramSourceDigest {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{:02x}", self.0)
    }
}

impl From<DefaultHasher> for ProgramSourceDigest {
    fn from(hasher: DefaultHasher) -> Self {
        use std::hash::Hasher;
        ProgramSourceDigest(hasher.finish())
    }
}

const SHADER_IMPORT: &str = "#include ";

pub struct ShaderSourceParser {
    included: HashSet<String>,
}

impl ShaderSourceParser {
    pub fn new() -> Self {
        ShaderSourceParser {
            included: HashSet::new(),
        }
    }

    /// Parses a shader string for imports. Imports are recursively processed, and
    /// prepended to the output stream.
    pub fn parse<F: FnMut(&str), G: Fn(&str) -> Cow<'static, str>>(
        &mut self,
        source: Cow<'static, str>,
        get_source: &G,
        output: &mut F,
    ) {
        for line in source.lines() {
            if line.starts_with(SHADER_IMPORT) {
                let imports = line[SHADER_IMPORT.len() ..].split(',');

                // For each import, get the source, and recurse.
                for import in imports {
                    if self.included.insert(import.into()) {
                        let include = get_source(import);
                        self.parse(include, get_source, output);
                    } else {
                        output(&format!("// {} is already included\n", import));
                    }
                }
            } else {
                output(line);
                output("\n");
            }
        }
    }
}

/// Reads a shader source file from disk into a String.
pub fn shader_source_from_file(shader_path: &Path) -> String {
    assert!(shader_path.exists(), "Shader not found {:?}", shader_path);
    let mut source = String::new();
    File::open(&shader_path)
        .expect("Shader not found")
        .read_to_string(&mut source)
        .unwrap();
    source
}

/// Creates heap-allocated strings for both vertex and fragment shaders.
pub fn build_shader_strings<G: Fn(&str) -> Cow<'static, str>>(
    gl_version: ShaderVersion,
    features: &[&str],
    base_filename: &str,
    get_source: &G,
) -> (String, String) {
   let mut vs_source = String::new();
   do_build_shader_string(
       gl_version,
       features,
       ShaderKind::Vertex,
       base_filename,
       get_source,
       |s| vs_source.push_str(s),
   );

   let mut fs_source = String::new();
   do_build_shader_string(
       gl_version,
       features,
       ShaderKind::Fragment,
       base_filename,
       get_source,
       |s| fs_source.push_str(s),
   );

   (vs_source, fs_source)
}

/// Walks the given shader string and applies the output to the provided
/// callback. Assuming an override path is not used, does no heap allocation
/// and no I/O.
pub fn do_build_shader_string<F: FnMut(&str), G: Fn(&str) -> Cow<'static, str>>(
   gl_version: ShaderVersion,
   features: &[&str],
   kind: ShaderKind,
   base_filename: &str,
   get_source: &G,
   mut output: F,
) {
   build_shader_prefix_string(gl_version, features, kind, base_filename, &mut output);
   build_shader_main_string(base_filename, get_source, &mut output);
}

/// Walks the prefix section of the shader string, which manages the various
/// defines for features etc.
pub fn build_shader_prefix_string<F: FnMut(&str)>(
   gl_version: ShaderVersion,
   features: &[&str],
   kind: ShaderKind,
   base_filename: &str,
   output: &mut F,
) {
    // GLSL requires that the version number comes first.
    let gl_version_string = match gl_version {
        ShaderVersion::Gl => "#version 150\n",
        ShaderVersion::Gles => "#version 300 es\n",
    };
    output(gl_version_string);

    // Insert the shader name to make debugging easier.
    output("// shader: ");
    output(base_filename);
    output(" ");
    for (i, feature) in features.iter().enumerate() {
        output(feature);
        if i != features.len() - 1 {
            output(",");
        }
    }
    output("\n");

    // Define a constant depending on whether we are compiling VS or FS.
    let kind_string = match kind {
        ShaderKind::Vertex => "#define WR_VERTEX_SHADER\n",
        ShaderKind::Fragment => "#define WR_FRAGMENT_SHADER\n",
    };
    output(kind_string);

    // Define a constant for the vertex texture width.
    output("#define WR_MAX_VERTEX_TEXTURE_WIDTH ");
    output(&MAX_VERTEX_TEXTURE_WIDTH_STRING);
    output("U\n");

    // Add any defines for features that were passed by the caller.
    for feature in features {
        assert!(!feature.is_empty());
        output("#define WR_FEATURE_");
        output(feature);
        output("\n");
    }
}

/// Walks the main .glsl file, including any imports.
pub fn build_shader_main_string<F: FnMut(&str), G: Fn(&str) -> Cow<'static, str>>(
   base_filename: &str,
   get_source: &G,
   output: &mut F,
) {
   let shared_source = get_source(base_filename);
   ShaderSourceParser::new().parse(
       shared_source,
       &|f| get_source(f),
       output
   );
}
