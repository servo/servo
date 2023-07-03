/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

bitflags! {
    #[derive(Default)]
    pub struct ShaderFeatureFlags: u32 {
        const GL = 1 << 0;
        const GLES = 1 << 1;

        const ADVANCED_BLEND_EQUATION = 1 << 8;
        const DUAL_SOURCE_BLENDING = 1 << 9;
        const PIXEL_LOCAL_STORAGE = 1 << 10;
        const DITHERING = 1 << 11;
        const TEXTURE_EXTERNAL = 1 << 12;
    }
}

pub type ShaderFeatures = HashMap<&'static str, Vec<String>>;

macro_rules! features {
    ($($str:expr),*) => { vec![$(String::from($str),)*] as Vec<String> };
}

fn concat_features(a: &str, b: &str) -> String {
    if a.is_empty() {
        b.to_string()
    } else if b.is_empty() {
        a.to_string()
    } else {
        [a, b].join(",")
    }
}

/// Computes available shaders and their features for the given feature flags.
pub fn get_shader_features(flags: ShaderFeatureFlags) -> ShaderFeatures {
    let mut shaders = ShaderFeatures::new();

    // Clip shaders
    shaders.insert("cs_clip_rectangle", features!["", "FAST_PATH"]);
    for name in &["cs_clip_image", "cs_clip_box_shadow"] {
        shaders.insert(name, features![""]);
    }

    // Cache shaders
    shaders.insert("cs_blur", features!["ALPHA_TARGET", "COLOR_TARGET"]);

    for name in &["cs_line_decoration", "cs_gradient", "cs_border_segment", "cs_border_solid", "cs_svg_filter"] {
        shaders.insert(name, features![""]);
    }

    shaders.insert("cs_scale", features![""]);

    // Pixel local storage shaders
    let pls_feature = if flags.contains(ShaderFeatureFlags::PIXEL_LOCAL_STORAGE) {
        for name in &["pls_init", "pls_resolve"] {
            shaders.insert(name, features!["PIXEL_LOCAL_STORAGE"]);
        }

        "PIXEL_LOCAL_STORAGE"
    } else {
        ""
    };

    // Brush shaders
    let brush_alpha_features = concat_features("ALPHA_PASS", pls_feature);
    for name in &["brush_solid", "brush_blend", "brush_mix_blend", "brush_opacity"] {
        shaders.insert(name, features!["", &brush_alpha_features, "DEBUG_OVERDRAW"]);
    }
    for name in &["brush_conic_gradient", "brush_radial_gradient", "brush_linear_gradient"] {
        let mut features: Vec<String> = Vec::new();
        let base = if flags.contains(ShaderFeatureFlags::DITHERING) {
            "DITHERING"
        } else {
            ""
        };
        features.push(base.to_string());
        features.push(concat_features(base, &brush_alpha_features));
        features.push(concat_features(base, "DEBUG_OVERDRAW"));
        shaders.insert(name, features);
    }

    // Image brush shaders
    let mut texture_types = vec!["", "TEXTURE_2D"];
    if flags.contains(ShaderFeatureFlags::GL) {
        texture_types.push("TEXTURE_RECT");
    }
    if flags.contains(ShaderFeatureFlags::TEXTURE_EXTERNAL) {
        texture_types.push("TEXTURE_EXTERNAL");
    }
    let mut image_features: Vec<String> = Vec::new();
    for texture_type in &texture_types {
        let fast = texture_type.to_string();
        image_features.push(fast.clone());
        image_features.push(concat_features(&fast, &brush_alpha_features));
        image_features.push(concat_features(&fast, "DEBUG_OVERDRAW"));
        let slow = concat_features(texture_type, "REPETITION,ANTIALIASING");
        image_features.push(slow.clone());
        image_features.push(concat_features(&slow, &brush_alpha_features));
        image_features.push(concat_features(&slow, "DEBUG_OVERDRAW"));
        if flags.contains(ShaderFeatureFlags::ADVANCED_BLEND_EQUATION) {
            let advanced_blend_features = concat_features(&brush_alpha_features, "ADVANCED_BLEND");
            image_features.push(concat_features(&fast, &advanced_blend_features));
            image_features.push(concat_features(&slow, &advanced_blend_features));
        }
        if flags.contains(ShaderFeatureFlags::DUAL_SOURCE_BLENDING) {
            let dual_source_features = concat_features(&brush_alpha_features, "DUAL_SOURCE_BLENDING");
            image_features.push(concat_features(&fast, &dual_source_features));
            image_features.push(concat_features(&slow, &dual_source_features));
        }
    }
    shaders.insert("brush_image", image_features);

    let mut composite_features: Vec<String> = Vec::new();
    for texture_type in &texture_types {
        let base = concat_features("", texture_type);
        composite_features.push(base.clone());
    }
    // YUV image brush shaders
    let mut yuv_features: Vec<String> = Vec::new();
    for texture_type in &texture_types {
        let base = concat_features("YUV", texture_type);
        composite_features.push(base.clone());
        yuv_features.push(base.clone());
        yuv_features.push(concat_features(&base, &brush_alpha_features));
        yuv_features.push(concat_features(&base, "DEBUG_OVERDRAW"));
    }
    shaders.insert("composite", composite_features);
    shaders.insert("brush_yuv_image", yuv_features);

    // Prim shaders
    let mut text_types = vec![pls_feature];
    if flags.contains(ShaderFeatureFlags::DUAL_SOURCE_BLENDING) {
        text_types.push("DUAL_SOURCE_BLENDING");
    }
    let mut text_features: Vec<String> = Vec::new();
    for text_type in &text_types {
        text_features.push(concat_features(text_type, "ALPHA_PASS"));
        text_features.push(concat_features(text_type, "GLYPH_TRANSFORM,ALPHA_PASS"));
        text_features.push(concat_features(text_type, "DEBUG_OVERDRAW"));
    }
    shaders.insert("ps_text_run", text_features);

    shaders.insert("ps_split_composite", features![pls_feature]);

    shaders.insert("ps_clear", features![""]);

    shaders
}

