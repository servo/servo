/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate mozangle;
extern crate webrender;
extern crate webrender_build;

use mozangle::shaders::{BuiltInResources, Output, ShaderSpec, ShaderValidator};
use webrender_build::shader::{ShaderFeatureFlags, ShaderVersion, build_shader_strings, get_shader_features};

// from glslang
const FRAGMENT_SHADER: u32 = 0x8B30;
const VERTEX_SHADER: u32 = 0x8B31;

#[test]
fn validate_shaders() {
    mozangle::shaders::initialize().unwrap();

    let resources = BuiltInResources::default();
    let vs_validator =
        ShaderValidator::new(VERTEX_SHADER, ShaderSpec::Gles3, Output::Essl, &resources).unwrap();

    let fs_validator =
        ShaderValidator::new(FRAGMENT_SHADER, ShaderSpec::Gles3, Output::Essl, &resources).unwrap();

    for (shader, configs) in get_shader_features(ShaderFeatureFlags::GLES) {
        for config in configs {
            let features = config.split(",").filter(|f| !f.is_empty()).collect::<Vec<_>>();

            let (vs, fs) = build_shader_strings(
                ShaderVersion::Gles,
                &features,
                shader,
                &|f| webrender::get_unoptimized_shader_source(f, None)
            );

            let full_shader_name = format!("{} {}", shader, config);
            validate(&vs_validator, &full_shader_name, vs);
            validate(&fs_validator, &full_shader_name, fs);
        }
    }
}

fn validate(validator: &ShaderValidator, name: &str, source: String) {
    // Check for each `switch` to have a `default`, see
    // https://github.com/servo/webrender/wiki/Driver-issues#lack-of-default-case-in-a-switch
    assert_eq!(source.matches("switch").count(), source.matches("default:").count(),
        "Shader '{}' doesn't have all `switch` covered with `default` cases", name);
    // Run Angle validator
    match validator.compile_and_translate(&[&source]) {
        Ok(_) => {
            // Ensure that the shader uses at most 16 varying vectors. This counts the number of
            // vectors assuming that the driver does not perform additional packing. The spec states
            // that the driver should pack varyings, however, on some Adreno 3xx devices we have
            // observed that this is not the case. See bug 1695912.
            let varying_vectors = validator.get_num_unpacked_varying_vectors();
            let max_varying_vectors = 16;
            assert!(
                varying_vectors <= max_varying_vectors,
                "Shader {} uses {} varying vectors. Max allowed {}",
                name, varying_vectors, max_varying_vectors
            );

            println!("Shader translated succesfully: {}", name);
        }
        Err(_) => {
            panic!(
                "Shader compilation failed: {}\n{}",
                name,
                validator.info_log()
            );
        }
    }
}
