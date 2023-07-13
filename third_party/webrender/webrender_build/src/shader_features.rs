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
        const DITHERING = 1 << 10;
        const TEXTURE_EXTERNAL = 1 << 11;
        const TEXTURE_EXTERNAL_ESSL1 = 1 << 12;
        const DEBUG = 1 << 13;
    }
}

pub type ShaderFeatures = HashMap<&'static str, Vec<String>>;

/// Builder for a list of features.
#[derive(Clone)]
struct FeatureList<'a> {
    list: Vec<&'a str>,
}

impl<'a> FeatureList<'a> {
    fn new() -> Self {
        FeatureList {
            list: Vec::new(),
        }
    }

    fn add(&mut self, feature: &'a str) {
        assert!(!feature.contains(','));
        self.list.push(feature);
    }

    fn with(&self, feature: &'a str) -> Self {
        let mut other = self.clone();
        other.add(feature);
        other
    }

    fn concat(&self, other: &Self) -> Self {
        let mut list = self.list.clone();
        list.extend_from_slice(&other.list);
        FeatureList {
            list
        }
    }

    fn finish(&mut self) -> String {
        self.list.sort_unstable();
        self.list.join(",")
    }
}

/// Computes available shaders and their features for the given feature flags.
pub fn get_shader_features(flags: ShaderFeatureFlags) -> ShaderFeatures {
    let mut shaders = ShaderFeatures::new();

    // Clip shaders
    shaders.insert("cs_clip_rectangle", vec![String::new(), "FAST_PATH".to_string()]);
    shaders.insert("cs_clip_image", vec!["TEXTURE_2D".to_string()]);
    shaders.insert("cs_clip_box_shadow", vec!["TEXTURE_2D".to_string()]);

    // Cache shaders
    shaders.insert("cs_blur", vec!["ALPHA_TARGET".to_string(), "COLOR_TARGET".to_string()]);

    for name in &[
        "cs_line_decoration",
        "cs_fast_linear_gradient",
        "cs_border_segment",
        "cs_border_solid",
        "cs_svg_filter",
    ] {
        shaders.insert(name, vec![String::new()]);
    }

    for name in &[
        "cs_linear_gradient",
        "cs_radial_gradient",
        "cs_conic_gradient",
    ] {
        let mut features = Vec::new();
        features.push(String::new());
        if flags.contains(ShaderFeatureFlags::DITHERING) {
            features.push("DITHERING".to_string());
        }
        shaders.insert(name, features);
    }

    let mut base_prim_features = FeatureList::new();

    // Brush shaders
    let mut brush_alpha_features = base_prim_features.with("ALPHA_PASS");
    for name in &["brush_solid", "brush_blend", "brush_mix_blend"] {
        let mut features: Vec<String> = Vec::new();
        features.push(base_prim_features.finish());
        features.push(brush_alpha_features.finish());
        features.push("DEBUG_OVERDRAW".to_string());
        shaders.insert(name, features);
    }
    for name in &["brush_linear_gradient"] {
        let mut features: Vec<String> = Vec::new();
        let mut list = FeatureList::new();
        if flags.contains(ShaderFeatureFlags::DITHERING) {
            list.add("DITHERING");
        }
        features.push(list.concat(&base_prim_features).finish());
        features.push(list.concat(&brush_alpha_features).finish());
        features.push(list.with("DEBUG_OVERDRAW").finish());
        shaders.insert(name, features);
    }

    {
        let mut features: Vec<String> = Vec::new();
        features.push(base_prim_features.finish());
        features.push(brush_alpha_features.finish());
        features.push(base_prim_features.with("ANTIALIASING").finish());
        features.push(brush_alpha_features.with("ANTIALIASING").finish());
        features.push("ANTIALIASING,DEBUG_OVERDRAW".to_string());
        features.push("DEBUG_OVERDRAW".to_string());
        shaders.insert("brush_opacity", features);
    }

    // Image brush shaders
    let mut texture_types = vec!["TEXTURE_2D"];
    if flags.contains(ShaderFeatureFlags::GL) {
        texture_types.push("TEXTURE_RECT");
    }
    if flags.contains(ShaderFeatureFlags::TEXTURE_EXTERNAL) {
        texture_types.push("TEXTURE_EXTERNAL");
    }
    let mut image_features: Vec<String> = Vec::new();
    for texture_type in &texture_types {
        let mut fast = FeatureList::new();
        if !texture_type.is_empty() {
            fast.add(texture_type);
        }
        image_features.push(fast.concat(&base_prim_features).finish());
        image_features.push(fast.concat(&brush_alpha_features).finish());
        image_features.push(fast.with("DEBUG_OVERDRAW").finish());
        let mut slow = fast.clone();
        slow.add("REPETITION");
        slow.add("ANTIALIASING");
        image_features.push(slow.concat(&base_prim_features).finish());
        image_features.push(slow.concat(&brush_alpha_features).finish());
        image_features.push(slow.with("DEBUG_OVERDRAW").finish());
        if flags.contains(ShaderFeatureFlags::ADVANCED_BLEND_EQUATION) {
            let advanced_blend_features = brush_alpha_features.with("ADVANCED_BLEND");
            image_features.push(fast.concat(&advanced_blend_features).finish());
            image_features.push(slow.concat(&advanced_blend_features).finish());
        }
        if flags.contains(ShaderFeatureFlags::DUAL_SOURCE_BLENDING) {
            let dual_source_features = brush_alpha_features.with("DUAL_SOURCE_BLENDING");
            image_features.push(fast.concat(&dual_source_features).finish());
            image_features.push(slow.concat(&dual_source_features).finish());
        }
    }
    shaders.insert("brush_image", image_features);

    let mut composite_texture_types = texture_types.clone();
    if flags.contains(ShaderFeatureFlags::TEXTURE_EXTERNAL_ESSL1) {
        composite_texture_types.push("TEXTURE_EXTERNAL_ESSL1");
    }
    let mut composite_features: Vec<String> = Vec::new();
    for texture_type in &composite_texture_types {
        let base = texture_type.to_string();
        composite_features.push(base);
    }
    shaders.insert("cs_scale", composite_features.clone());

    // YUV image brush and composite shaders
    let mut yuv_features: Vec<String> = Vec::new();
    for texture_type in &texture_types {
        let mut list = FeatureList::new();
        if !texture_type.is_empty() {
            list.add(texture_type);
        }
        list.add("YUV");
        composite_features.push(list.finish());
        yuv_features.push(list.concat(&base_prim_features).finish());
        yuv_features.push(list.concat(&brush_alpha_features).finish());
        yuv_features.push(list.with("DEBUG_OVERDRAW").finish());
    }
    shaders.insert("brush_yuv_image", yuv_features);

    // Fast path composite shaders
    for texture_type in &composite_texture_types {
        let mut list = FeatureList::new();
        if !texture_type.is_empty() {
            list.add(texture_type);
        }
        list.add("FAST_PATH");
        composite_features.push(list.finish());
    }
    shaders.insert("composite", composite_features);

    // Prim shaders
    let mut text_types = vec![""];
    if flags.contains(ShaderFeatureFlags::DUAL_SOURCE_BLENDING) {
        text_types.push("DUAL_SOURCE_BLENDING");
    }
    let mut text_features: Vec<String> = Vec::new();
    for text_type in &text_types {
        let mut list = base_prim_features.with("TEXTURE_2D");
        if !text_type.is_empty() {
            list.add(text_type);
        }
        let mut alpha_list = list.with("ALPHA_PASS");
        text_features.push(alpha_list.finish());
        text_features.push(alpha_list.with("GLYPH_TRANSFORM").finish());
        text_features.push(list.with("DEBUG_OVERDRAW").finish());
    }
    shaders.insert("ps_text_run", text_features);

    shaders.insert("ps_split_composite", vec![base_prim_features.finish()]);

    shaders.insert("ps_clear", vec![base_prim_features.finish()]);

    if flags.contains(ShaderFeatureFlags::DEBUG) {
        for name in &["debug_color", "debug_font"] {
            shaders.insert(name, vec![String::new()]);
        }
    }

    shaders
}

