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

            validate(&vs_validator, shader, vs);
            validate(&fs_validator, shader, fs);
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
