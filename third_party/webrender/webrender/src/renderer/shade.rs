/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use api::{ImageBufferKind, units::DeviceSize};
use crate::batch::{BatchKey, BatchKind, BrushBatchKind, BatchFeatures};
use crate::composite::{CompositeFeatures, CompositeSurfaceFormat};
use crate::device::{Device, Program, ShaderError};
use euclid::default::Transform3D;
use crate::glyph_rasterizer::GlyphFormat;
use crate::renderer::{
    desc,
    BlendMode, DebugFlags, RendererError, RendererOptions,
    TextureSampler, VertexArrayKind, ShaderPrecacheFlags,
};

use gleam::gl::GlType;
use time::precise_time_ns;

use std::cell::RefCell;
use std::rc::Rc;

use webrender_build::shader::{ShaderFeatures, ShaderFeatureFlags, get_shader_features};

/// Which extension version to use for texture external support.
#[derive(Clone, Copy, Debug, PartialEq)]
enum TextureExternalVersion {
    // GL_OES_EGL_image_external_essl3 (Compatible with ESSL 3.0 and
    // later shaders, but not supported on all GLES 3 devices.)
    ESSL3,
    // GL_OES_EGL_image_external (Compatible with ESSL 1.0 shaders)
    ESSL1,
}

fn get_feature_string(kind: ImageBufferKind, texture_external_version: TextureExternalVersion) -> &'static str {
    match (kind, texture_external_version) {
        (ImageBufferKind::Texture2D, _) => "TEXTURE_2D",
        (ImageBufferKind::TextureRect, _) => "TEXTURE_RECT",
        (ImageBufferKind::TextureExternal, TextureExternalVersion::ESSL3) => "TEXTURE_EXTERNAL",
        (ImageBufferKind::TextureExternal, TextureExternalVersion::ESSL1) => "TEXTURE_EXTERNAL_ESSL1",
    }
}

fn has_platform_support(kind: ImageBufferKind, gl_type: &GlType) -> bool {
    match (kind, gl_type) {
        (ImageBufferKind::Texture2D, _) => true,
        (ImageBufferKind::TextureRect, &GlType::Gles) => false,
        (ImageBufferKind::TextureRect, &GlType::Gl) => true,
        (ImageBufferKind::TextureExternal, &GlType::Gles) => true,
        (ImageBufferKind::TextureExternal, &GlType::Gl) => false,
    }
}

pub const IMAGE_BUFFER_KINDS: [ImageBufferKind; 3] = [
    ImageBufferKind::Texture2D,
    ImageBufferKind::TextureRect,
    ImageBufferKind::TextureExternal,
];

const ADVANCED_BLEND_FEATURE: &str = "ADVANCED_BLEND";
const ALPHA_FEATURE: &str = "ALPHA_PASS";
const DEBUG_OVERDRAW_FEATURE: &str = "DEBUG_OVERDRAW";
const DITHERING_FEATURE: &str = "DITHERING";
const DUAL_SOURCE_FEATURE: &str = "DUAL_SOURCE_BLENDING";
const FAST_PATH_FEATURE: &str = "FAST_PATH";

pub(crate) enum ShaderKind {
    Primitive,
    Cache(VertexArrayKind),
    ClipCache(VertexArrayKind),
    Brush,
    Text,
    #[allow(dead_code)]
    VectorStencil,
    #[allow(dead_code)]
    VectorCover,
    #[allow(dead_code)]
    Resolve,
    Composite,
    Clear,
}

pub struct LazilyCompiledShader {
    program: Option<Program>,
    name: &'static str,
    kind: ShaderKind,
    cached_projection: Transform3D<f32>,
    features: Vec<&'static str>,
}

impl LazilyCompiledShader {
    pub(crate) fn new(
        kind: ShaderKind,
        name: &'static str,
        unsorted_features: &[&'static str],
        device: &mut Device,
        precache_flags: ShaderPrecacheFlags,
        shader_list: &ShaderFeatures,
    ) -> Result<Self, ShaderError> {
        let mut features = unsorted_features.to_vec();
        features.sort();

        // Ensure this shader config is in the available shader list so that we get
        // alerted if the list gets out-of-date when shaders or features are added.
        let config = features.join(",");
        assert!(
            shader_list.get(name).map_or(false, |f| f.contains(&config)),
            "shader \"{}\" with features \"{}\" not in available shader list",
            name,
            config,
        );

        let mut shader = LazilyCompiledShader {
            program: None,
            name,
            kind,
            //Note: this isn't really the default state, but there is no chance
            // an actual projection passed here would accidentally match.
            cached_projection: Transform3D::identity(),
            features,
        };

        if precache_flags.intersects(ShaderPrecacheFlags::ASYNC_COMPILE | ShaderPrecacheFlags::FULL_COMPILE) {
            let t0 = precise_time_ns();
            shader.get_internal(device, precache_flags)?;
            let t1 = precise_time_ns();
            debug!("[C: {:.1} ms ] Precache {} {:?}",
                (t1 - t0) as f64 / 1000000.0,
                name,
                unsorted_features
            );
        }

        Ok(shader)
    }

    pub fn bind(
        &mut self,
        device: &mut Device,
        projection: &Transform3D<f32>,
        texture_size: Option<DeviceSize>,
        renderer_errors: &mut Vec<RendererError>,
    ) {
        let update_projection = self.cached_projection != *projection;
        let program = match self.get_internal(device, ShaderPrecacheFlags::FULL_COMPILE) {
            Ok(program) => program,
            Err(e) => {
                renderer_errors.push(RendererError::from(e));
                return;
            }
        };
        device.bind_program(program);
        if let Some(texture_size) = texture_size {
            device.set_shader_texture_size(program, texture_size);
        }
        if update_projection {
            device.set_uniforms(program, projection);
            // thanks NLL for this (`program` technically borrows `self`)
            self.cached_projection = *projection;
        }
    }

    fn get_internal(
        &mut self,
        device: &mut Device,
        precache_flags: ShaderPrecacheFlags,
    ) -> Result<&mut Program, ShaderError> {
        if self.program.is_none() {
            let program = match self.kind {
                ShaderKind::Primitive | ShaderKind::Brush | ShaderKind::Text | ShaderKind::Resolve | ShaderKind::Clear => {
                    create_prim_shader(
                        self.name,
                        device,
                        &self.features,
                    )
                }
                ShaderKind::Cache(..) => {
                    create_prim_shader(
                        self.name,
                        device,
                        &self.features,
                    )
                }
                ShaderKind::VectorStencil => {
                    create_prim_shader(
                        self.name,
                        device,
                        &self.features,
                    )
                }
                ShaderKind::VectorCover => {
                    create_prim_shader(
                        self.name,
                        device,
                        &self.features,
                    )
                }
                ShaderKind::Composite => {
                    create_prim_shader(
                        self.name,
                        device,
                        &self.features,
                    )
                }
                ShaderKind::ClipCache(..) => {
                    create_clip_shader(
                        self.name,
                        device,
                        &self.features,
                    )
                }
            };
            self.program = Some(program?);
        }

        let program = self.program.as_mut().unwrap();

        if precache_flags.contains(ShaderPrecacheFlags::FULL_COMPILE) && !program.is_initialized() {
            let vertex_format = match self.kind {
                ShaderKind::Primitive |
                ShaderKind::Brush |
                ShaderKind::Text => VertexArrayKind::Primitive,
                ShaderKind::Cache(format) => format,
                ShaderKind::VectorStencil => VertexArrayKind::VectorStencil,
                ShaderKind::VectorCover => VertexArrayKind::VectorCover,
                ShaderKind::ClipCache(format) => format,
                ShaderKind::Resolve => VertexArrayKind::Resolve,
                ShaderKind::Composite => VertexArrayKind::Composite,
                ShaderKind::Clear => VertexArrayKind::Clear,
            };

            let vertex_descriptor = match vertex_format {
                VertexArrayKind::Primitive => &desc::PRIM_INSTANCES,
                VertexArrayKind::LineDecoration => &desc::LINE,
                VertexArrayKind::FastLinearGradient => &desc::FAST_LINEAR_GRADIENT,
                VertexArrayKind::LinearGradient => &desc::LINEAR_GRADIENT,
                VertexArrayKind::RadialGradient => &desc::RADIAL_GRADIENT,
                VertexArrayKind::ConicGradient => &desc::CONIC_GRADIENT,
                VertexArrayKind::Blur => &desc::BLUR,
                VertexArrayKind::ClipImage => &desc::CLIP_IMAGE,
                VertexArrayKind::ClipRect => &desc::CLIP_RECT,
                VertexArrayKind::ClipBoxShadow => &desc::CLIP_BOX_SHADOW,
                VertexArrayKind::VectorStencil => &desc::VECTOR_STENCIL,
                VertexArrayKind::VectorCover => &desc::VECTOR_COVER,
                VertexArrayKind::Border => &desc::BORDER,
                VertexArrayKind::Scale => &desc::SCALE,
                VertexArrayKind::Resolve => &desc::RESOLVE,
                VertexArrayKind::SvgFilter => &desc::SVG_FILTER,
                VertexArrayKind::Composite => &desc::COMPOSITE,
                VertexArrayKind::Clear => &desc::CLEAR,
            };

            device.link_program(program, vertex_descriptor)?;
            device.bind_program(program);
            match self.kind {
                ShaderKind::ClipCache(..) => {
                    device.bind_shader_samplers(
                        &program,
                        &[
                            ("sColor0", TextureSampler::Color0),
                            ("sTransformPalette", TextureSampler::TransformPalette),
                            ("sRenderTasks", TextureSampler::RenderTasks),
                            ("sGpuCache", TextureSampler::GpuCache),
                            ("sPrimitiveHeadersF", TextureSampler::PrimitiveHeadersF),
                            ("sPrimitiveHeadersI", TextureSampler::PrimitiveHeadersI),
                        ],
                    );
                }
                _ => {
                    device.bind_shader_samplers(
                        &program,
                        &[
                            ("sColor0", TextureSampler::Color0),
                            ("sColor1", TextureSampler::Color1),
                            ("sColor2", TextureSampler::Color2),
                            ("sDither", TextureSampler::Dither),
                            ("sTransformPalette", TextureSampler::TransformPalette),
                            ("sRenderTasks", TextureSampler::RenderTasks),
                            ("sGpuCache", TextureSampler::GpuCache),
                            ("sPrimitiveHeadersF", TextureSampler::PrimitiveHeadersF),
                            ("sPrimitiveHeadersI", TextureSampler::PrimitiveHeadersI),
                            ("sClipMask", TextureSampler::ClipMask),
                        ],
                    );
                }
            }
        }

        Ok(program)
    }

    fn deinit(self, device: &mut Device) {
        if let Some(program) = self.program {
            device.delete_program(program);
        }
    }
}

// A brush shader supports two modes:
// opaque:
//   Used for completely opaque primitives,
//   or inside segments of partially
//   opaque primitives. Assumes no need
//   for clip masks, AA etc.
// alpha:
//   Used for brush primitives in the alpha
//   pass. Assumes that AA should be applied
//   along the primitive edge, and also that
//   clip mask is present.
struct BrushShader {
    opaque: LazilyCompiledShader,
    alpha: LazilyCompiledShader,
    advanced_blend: Option<LazilyCompiledShader>,
    dual_source: Option<LazilyCompiledShader>,
    debug_overdraw: LazilyCompiledShader,
}

impl BrushShader {
    fn new(
        name: &'static str,
        device: &mut Device,
        features: &[&'static str],
        precache_flags: ShaderPrecacheFlags,
        shader_list: &ShaderFeatures,
        use_advanced_blend: bool,
        use_dual_source: bool,
    ) -> Result<Self, ShaderError> {
        let opaque_features = features.to_vec();
        let opaque = LazilyCompiledShader::new(
            ShaderKind::Brush,
            name,
            &opaque_features,
            device,
            precache_flags,
            &shader_list,
        )?;

        let mut alpha_features = opaque_features.to_vec();
        alpha_features.push(ALPHA_FEATURE);

        let alpha = LazilyCompiledShader::new(
            ShaderKind::Brush,
            name,
            &alpha_features,
            device,
            precache_flags,
            &shader_list,
        )?;

        let advanced_blend = if use_advanced_blend {
            let mut advanced_blend_features = alpha_features.to_vec();
            advanced_blend_features.push(ADVANCED_BLEND_FEATURE);

            let shader = LazilyCompiledShader::new(
                ShaderKind::Brush,
                name,
                &advanced_blend_features,
                device,
                precache_flags,
                &shader_list,
            )?;

            Some(shader)
        } else {
            None
        };

        let dual_source = if use_dual_source {
            let mut dual_source_features = alpha_features.to_vec();
            dual_source_features.push(DUAL_SOURCE_FEATURE);

            let shader = LazilyCompiledShader::new(
                ShaderKind::Brush,
                name,
                &dual_source_features,
                device,
                precache_flags,
                &shader_list,
            )?;

            Some(shader)
        } else {
            None
        };

        let mut debug_overdraw_features = features.to_vec();
        debug_overdraw_features.push(DEBUG_OVERDRAW_FEATURE);

        let debug_overdraw = LazilyCompiledShader::new(
            ShaderKind::Brush,
            name,
            &debug_overdraw_features,
            device,
            precache_flags,
            &shader_list,
        )?;

        Ok(BrushShader {
            opaque,
            alpha,
            advanced_blend,
            dual_source,
            debug_overdraw,
        })
    }

    fn get(&mut self, blend_mode: BlendMode, features: BatchFeatures, debug_flags: DebugFlags)
           -> &mut LazilyCompiledShader {
        match blend_mode {
            _ if debug_flags.contains(DebugFlags::SHOW_OVERDRAW) => &mut self.debug_overdraw,
            BlendMode::None => &mut self.opaque,
            BlendMode::Alpha |
            BlendMode::PremultipliedAlpha |
            BlendMode::PremultipliedDestOut |
            BlendMode::SubpixelConstantTextColor(..) |
            BlendMode::SubpixelWithBgColor |
            BlendMode::Screen |
            BlendMode::Exclusion => {
                if features.contains(BatchFeatures::ALPHA_PASS) {
                    &mut self.alpha
                } else {
                    &mut self.opaque
                }
            }
            BlendMode::Advanced(_) => {
                self.advanced_blend
                    .as_mut()
                    .expect("bug: no advanced blend shader loaded")
            }
            BlendMode::SubpixelDualSource |
            BlendMode::MultiplyDualSource => {
                self.dual_source
                    .as_mut()
                    .expect("bug: no dual source shader loaded")
            }
        }
    }

    fn deinit(self, device: &mut Device) {
        self.opaque.deinit(device);
        self.alpha.deinit(device);
        if let Some(advanced_blend) = self.advanced_blend {
            advanced_blend.deinit(device);
        }
        if let Some(dual_source) = self.dual_source {
            dual_source.deinit(device);
        }
        self.debug_overdraw.deinit(device);
    }
}

pub struct TextShader {
    simple: LazilyCompiledShader,
    glyph_transform: LazilyCompiledShader,
    debug_overdraw: LazilyCompiledShader,
}

impl TextShader {
    fn new(
        name: &'static str,
        device: &mut Device,
        features: &[&'static str],
        precache_flags: ShaderPrecacheFlags,
        shader_list: &ShaderFeatures,
    ) -> Result<Self, ShaderError> {
        let mut simple_features = features.to_vec();
        simple_features.push("ALPHA_PASS");
        simple_features.push("TEXTURE_2D");

        let simple = LazilyCompiledShader::new(
            ShaderKind::Text,
            name,
            &simple_features,
            device,
            precache_flags,
            &shader_list,
        )?;

        let mut glyph_transform_features = features.to_vec();
        glyph_transform_features.push("GLYPH_TRANSFORM");
        glyph_transform_features.push("ALPHA_PASS");
        glyph_transform_features.push("TEXTURE_2D");

        let glyph_transform = LazilyCompiledShader::new(
            ShaderKind::Text,
            name,
            &glyph_transform_features,
            device,
            precache_flags,
            &shader_list,
        )?;

        let mut debug_overdraw_features = features.to_vec();
        debug_overdraw_features.push("DEBUG_OVERDRAW");
        debug_overdraw_features.push("TEXTURE_2D");

        let debug_overdraw = LazilyCompiledShader::new(
            ShaderKind::Text,
            name,
            &debug_overdraw_features,
            device,
            precache_flags,
            &shader_list,
        )?;

        Ok(TextShader { simple, glyph_transform, debug_overdraw })
    }

    pub fn get(
        &mut self,
        glyph_format: GlyphFormat,
        debug_flags: DebugFlags,
    ) -> &mut LazilyCompiledShader {
        match glyph_format {
            _ if debug_flags.contains(DebugFlags::SHOW_OVERDRAW) => &mut self.debug_overdraw,
            GlyphFormat::Alpha |
            GlyphFormat::Subpixel |
            GlyphFormat::Bitmap |
            GlyphFormat::ColorBitmap => &mut self.simple,
            GlyphFormat::TransformedAlpha |
            GlyphFormat::TransformedSubpixel => &mut self.glyph_transform,
        }
    }

    fn deinit(self, device: &mut Device) {
        self.simple.deinit(device);
        self.glyph_transform.deinit(device);
        self.debug_overdraw.deinit(device);
    }
}

fn create_prim_shader(
    name: &'static str,
    device: &mut Device,
    features: &[&'static str],
) -> Result<Program, ShaderError> {
    debug!("PrimShader {}", name);

    device.create_program(name, features)
}

fn create_clip_shader(
    name: &'static str,
    device: &mut Device,
    features: &[&'static str],
) -> Result<Program, ShaderError> {
    debug!("ClipShader {}", name);

    device.create_program(name, features)
}

// NB: If you add a new shader here, make sure to deinitialize it
// in `Shaders::deinit()` below.
pub struct Shaders {
    // These are "cache shaders". These shaders are used to
    // draw intermediate results to cache targets. The results
    // of these shaders are then used by the primitive shaders.
    pub cs_blur_a8: LazilyCompiledShader,
    pub cs_blur_rgba8: LazilyCompiledShader,
    pub cs_border_segment: LazilyCompiledShader,
    pub cs_border_solid: LazilyCompiledShader,
    pub cs_scale: Vec<Option<LazilyCompiledShader>>,
    pub cs_line_decoration: LazilyCompiledShader,
    pub cs_fast_linear_gradient: LazilyCompiledShader,
    pub cs_linear_gradient: LazilyCompiledShader,
    pub cs_radial_gradient: LazilyCompiledShader,
    pub cs_conic_gradient: LazilyCompiledShader,
    pub cs_svg_filter: LazilyCompiledShader,

    // Brush shaders
    brush_solid: BrushShader,
    brush_image: Vec<Option<BrushShader>>,
    brush_fast_image: Vec<Option<BrushShader>>,
    brush_blend: BrushShader,
    brush_mix_blend: BrushShader,
    brush_yuv_image: Vec<Option<BrushShader>>,
    brush_linear_gradient: BrushShader,
    brush_opacity: BrushShader,
    brush_opacity_aa: BrushShader,

    /// These are "cache clip shaders". These shaders are used to
    /// draw clip instances into the cached clip mask. The results
    /// of these shaders are also used by the primitive shaders.
    pub cs_clip_rectangle_slow: LazilyCompiledShader,
    pub cs_clip_rectangle_fast: LazilyCompiledShader,
    pub cs_clip_box_shadow: LazilyCompiledShader,
    pub cs_clip_image: LazilyCompiledShader,

    // The are "primitive shaders". These shaders draw and blend
    // final results on screen. They are aware of tile boundaries.
    // Most draw directly to the framebuffer, but some use inputs
    // from the cache shaders to draw. Specifically, the box
    // shadow primitive shader stretches the box shadow cache
    // output, and the cache_image shader blits the results of
    // a cache shader (e.g. blur) to the screen.
    pub ps_text_run: TextShader,
    pub ps_text_run_dual_source: Option<TextShader>,

    ps_split_composite: LazilyCompiledShader,
    pub ps_clear: LazilyCompiledShader,

    // Composite shaders.  These are very simple shaders used to composite
    // picture cache tiles into the framebuffer on platforms that do not have an
    // OS Compositor (or we cannot use it).  Such an OS Compositor (such as
    // DirectComposite or CoreAnimation) handles the composition of the picture
    // cache tiles at a lower level (e.g. in DWM for Windows); in that case we
    // directly hand the picture cache surfaces over to the OS Compositor, and
    // our own Composite shaders below never run.
    // To composite external (RGB) surfaces we need various permutations of
    // shaders with WR_FEATURE flags on or off based on the type of image
    // buffer we're sourcing from (see IMAGE_BUFFER_KINDS).
    pub composite_rgba: Vec<Option<LazilyCompiledShader>>,
    // A faster set of rgba composite shaders that do not support UV clamping
    // or color modulation.
    pub composite_rgba_fast_path: Vec<Option<LazilyCompiledShader>>,
    // The same set of composite shaders but with WR_FEATURE_YUV added.
    pub composite_yuv: Vec<Option<LazilyCompiledShader>>,
}

impl Shaders {
    pub fn new(
        device: &mut Device,
        gl_type: GlType,
        options: &RendererOptions,
    ) -> Result<Self, ShaderError> {
        let use_dual_source_blending =
            device.get_capabilities().supports_dual_source_blending &&
            options.allow_dual_source_blending;
        let use_advanced_blend_equation =
            device.get_capabilities().supports_advanced_blend_equation &&
            options.allow_advanced_blend_equation;

        let texture_external_version = if device.get_capabilities().supports_image_external_essl3 {
            TextureExternalVersion::ESSL3
        } else {
            TextureExternalVersion::ESSL1
        };
        let mut shader_flags = match gl_type {
            GlType::Gl => ShaderFeatureFlags::GL,
            GlType::Gles => {
                let texture_external_flag = match texture_external_version {
                    TextureExternalVersion::ESSL3 => ShaderFeatureFlags::TEXTURE_EXTERNAL,
                    TextureExternalVersion::ESSL1 => ShaderFeatureFlags::TEXTURE_EXTERNAL_ESSL1,
                };
                ShaderFeatureFlags::GLES | texture_external_flag
            }
        };
        shader_flags.set(ShaderFeatureFlags::ADVANCED_BLEND_EQUATION, use_advanced_blend_equation);
        shader_flags.set(ShaderFeatureFlags::DUAL_SOURCE_BLENDING, use_dual_source_blending);
        shader_flags.set(ShaderFeatureFlags::DITHERING, options.enable_dithering);
        let shader_list = get_shader_features(shader_flags);

        let brush_solid = BrushShader::new(
            "brush_solid",
            device,
            &[],
            options.precache_flags,
            &shader_list,
            false /* advanced blend */,
            false /* dual source */,
        )?;

        let brush_blend = BrushShader::new(
            "brush_blend",
            device,
            &[],
            options.precache_flags,
            &shader_list,
            false /* advanced blend */,
            false /* dual source */,
        )?;

        let brush_mix_blend = BrushShader::new(
            "brush_mix_blend",
            device,
            &[],
            options.precache_flags,
            &shader_list,
            false /* advanced blend */,
            false /* dual source */,
        )?;

        let brush_linear_gradient = BrushShader::new(
            "brush_linear_gradient",
            device,
            if options.enable_dithering {
               &[DITHERING_FEATURE]
            } else {
               &[]
            },
            options.precache_flags,
            &shader_list,
            false /* advanced blend */,
            false /* dual source */,
        )?;

        let brush_opacity_aa = BrushShader::new(
            "brush_opacity",
            device,
            &["ANTIALIASING"],
            options.precache_flags,
            &shader_list,
            false /* advanced blend */,
            false /* dual source */,
        )?;

        let brush_opacity = BrushShader::new(
            "brush_opacity",
            device,
            &[],
            options.precache_flags,
            &shader_list,
            false /* advanced blend */,
            false /* dual source */,
        )?;

        let cs_blur_a8 = LazilyCompiledShader::new(
            ShaderKind::Cache(VertexArrayKind::Blur),
            "cs_blur",
            &["ALPHA_TARGET"],
            device,
            options.precache_flags,
            &shader_list,
        )?;

        let cs_blur_rgba8 = LazilyCompiledShader::new(
            ShaderKind::Cache(VertexArrayKind::Blur),
            "cs_blur",
            &["COLOR_TARGET"],
            device,
            options.precache_flags,
            &shader_list,
        )?;

        let cs_svg_filter = LazilyCompiledShader::new(
            ShaderKind::Cache(VertexArrayKind::SvgFilter),
            "cs_svg_filter",
            &[],
            device,
            options.precache_flags,
            &shader_list,
        )?;

        let cs_clip_rectangle_slow = LazilyCompiledShader::new(
            ShaderKind::ClipCache(VertexArrayKind::ClipRect),
            "cs_clip_rectangle",
            &[],
            device,
            options.precache_flags,
            &shader_list,
        )?;

        let cs_clip_rectangle_fast = LazilyCompiledShader::new(
            ShaderKind::ClipCache(VertexArrayKind::ClipRect),
            "cs_clip_rectangle",
            &[FAST_PATH_FEATURE],
            device,
            options.precache_flags,
            &shader_list,
        )?;

        let cs_clip_box_shadow = LazilyCompiledShader::new(
            ShaderKind::ClipCache(VertexArrayKind::ClipBoxShadow),
            "cs_clip_box_shadow",
            &["TEXTURE_2D"],
            device,
            options.precache_flags,
            &shader_list,
        )?;

        let cs_clip_image = LazilyCompiledShader::new(
            ShaderKind::ClipCache(VertexArrayKind::ClipImage),
            "cs_clip_image",
            &["TEXTURE_2D"],
            device,
            options.precache_flags,
            &shader_list,
        )?;

        let mut cs_scale = Vec::new();
        let scale_shader_num = IMAGE_BUFFER_KINDS.len();
        // PrimitiveShader is not clonable. Use push() to initialize the vec.
        for _ in 0 .. scale_shader_num {
            cs_scale.push(None);
        }
        for image_buffer_kind in &IMAGE_BUFFER_KINDS {
            if has_platform_support(*image_buffer_kind, &gl_type) {
                let feature_string = get_feature_string(
                    *image_buffer_kind,
                    texture_external_version,
                );

                let mut features = Vec::new();
                if feature_string != "" {
                    features.push(feature_string);
                }

                let shader = LazilyCompiledShader::new(
                    ShaderKind::Cache(VertexArrayKind::Scale),
                    "cs_scale",
                    &features,
                    device,
                    options.precache_flags,
                    &shader_list,
                 )?;

                 let index = Self::get_compositing_shader_index(
                    *image_buffer_kind,
                 );
                 cs_scale[index] = Some(shader);
            }
        }

        // TODO(gw): The split composite + text shader are special cases - the only
        //           shaders used during normal scene rendering that aren't a brush
        //           shader. Perhaps we can unify these in future?

        let ps_text_run = TextShader::new("ps_text_run",
            device,
            &[],
            options.precache_flags,
            &shader_list,
        )?;

        let ps_text_run_dual_source = if use_dual_source_blending {
            let dual_source_features = vec![DUAL_SOURCE_FEATURE];
            Some(TextShader::new("ps_text_run",
                device,
                &dual_source_features,
                options.precache_flags,
                &shader_list,
            )?)
        } else {
            None
        };

        let ps_split_composite = LazilyCompiledShader::new(
            ShaderKind::Primitive,
            "ps_split_composite",
            &[],
            device,
            options.precache_flags,
            &shader_list,
        )?;

        let ps_clear = LazilyCompiledShader::new(
            ShaderKind::Clear,
            "ps_clear",
            &[],
            device,
            options.precache_flags,
            &shader_list,
        )?;

        // All image configuration.
        let mut image_features = Vec::new();
        let mut brush_image = Vec::new();
        let mut brush_fast_image = Vec::new();
        // PrimitiveShader is not clonable. Use push() to initialize the vec.
        for _ in 0 .. IMAGE_BUFFER_KINDS.len() {
            brush_image.push(None);
            brush_fast_image.push(None);
        }
        for buffer_kind in 0 .. IMAGE_BUFFER_KINDS.len() {
            if !has_platform_support(IMAGE_BUFFER_KINDS[buffer_kind], &gl_type)
                // Brush shaders are not ESSL1 compatible
                || (IMAGE_BUFFER_KINDS[buffer_kind] == ImageBufferKind::TextureExternal
                    && texture_external_version == TextureExternalVersion::ESSL1)
            {
                continue;
            }

            let feature_string = get_feature_string(
                IMAGE_BUFFER_KINDS[buffer_kind],
                texture_external_version,
            );
            if feature_string != "" {
                image_features.push(feature_string);
            }

            brush_fast_image[buffer_kind] = Some(BrushShader::new(
                "brush_image",
                device,
                &image_features,
                options.precache_flags,
                &shader_list,
                use_advanced_blend_equation,
                use_dual_source_blending,
            )?);

            image_features.push("REPETITION");
            image_features.push("ANTIALIASING");

            brush_image[buffer_kind] = Some(BrushShader::new(
                "brush_image",
                device,
                &image_features,
                options.precache_flags,
                &shader_list,
                use_advanced_blend_equation,
                use_dual_source_blending,
            )?);

            image_features.clear();
        }

        // All yuv_image configuration.
        let mut yuv_features = Vec::new();
        let mut rgba_features = Vec::new();
        let mut fast_path_features = Vec::new();
        let yuv_shader_num = IMAGE_BUFFER_KINDS.len();
        let mut brush_yuv_image = Vec::new();
        let mut composite_yuv = Vec::new();
        let mut composite_rgba = Vec::new();
        let mut composite_rgba_fast_path = Vec::new();
        // PrimitiveShader is not clonable. Use push() to initialize the vec.
        for _ in 0 .. yuv_shader_num {
            brush_yuv_image.push(None);
            composite_yuv.push(None);
            composite_rgba.push(None);
            composite_rgba_fast_path.push(None);
        }
        for image_buffer_kind in &IMAGE_BUFFER_KINDS {
            if has_platform_support(*image_buffer_kind, &gl_type) {
                yuv_features.push("YUV");
                fast_path_features.push("FAST_PATH");

                let index = Self::get_compositing_shader_index(
                    *image_buffer_kind,
                );

                let feature_string = get_feature_string(
                    *image_buffer_kind,
                    texture_external_version,
                );
                if feature_string != "" {
                    yuv_features.push(feature_string);
                    rgba_features.push(feature_string);
                    fast_path_features.push(feature_string);
                }

                // YUV shaders are not compatible with ESSL1
                if *image_buffer_kind != ImageBufferKind::TextureExternal ||
                    texture_external_version == TextureExternalVersion::ESSL3 {
                    let brush_shader = BrushShader::new(
                        "brush_yuv_image",
                        device,
                        &yuv_features,
                        options.precache_flags,
                        &shader_list,
                        false /* advanced blend */,
                        false /* dual source */,
                    )?;
                    brush_yuv_image[index] = Some(brush_shader);

                    let composite_yuv_shader = LazilyCompiledShader::new(
                        ShaderKind::Composite,
                        "composite",
                        &yuv_features,
                        device,
                        options.precache_flags,
                        &shader_list,
                    )?;
                    composite_yuv[index] = Some(composite_yuv_shader);
                }

                let composite_rgba_shader = LazilyCompiledShader::new(
                    ShaderKind::Composite,
                    "composite",
                    &rgba_features,
                    device,
                    options.precache_flags,
                    &shader_list,
                )?;

                let composite_rgba_fast_path_shader = LazilyCompiledShader::new(
                    ShaderKind::Composite,
                    "composite",
                    &fast_path_features,
                    device,
                    options.precache_flags,
                    &shader_list,
                )?;

                let index = Self::get_compositing_shader_index(
                    *image_buffer_kind,
                );
                composite_rgba[index] = Some(composite_rgba_shader);
                composite_rgba_fast_path[index] = Some(composite_rgba_fast_path_shader);

                yuv_features.clear();
                rgba_features.clear();
                fast_path_features.clear();
            }
        }

        let cs_line_decoration = LazilyCompiledShader::new(
            ShaderKind::Cache(VertexArrayKind::LineDecoration),
            "cs_line_decoration",
            &[],
            device,
            options.precache_flags,
            &shader_list,
        )?;

        let cs_fast_linear_gradient = LazilyCompiledShader::new(
            ShaderKind::Cache(VertexArrayKind::FastLinearGradient),
            "cs_fast_linear_gradient",
            &[],
            device,
            options.precache_flags,
            &shader_list,
        )?;

        let cs_linear_gradient = LazilyCompiledShader::new(
            ShaderKind::Cache(VertexArrayKind::LinearGradient),
            "cs_linear_gradient",
            &[],
            device,
            options.precache_flags,
            &shader_list,
        )?;

        let cs_radial_gradient = LazilyCompiledShader::new(
            ShaderKind::Cache(VertexArrayKind::RadialGradient),
            "cs_radial_gradient",
            &[],
            device,
            options.precache_flags,
            &shader_list,
        )?;

        let cs_conic_gradient = LazilyCompiledShader::new(
            ShaderKind::Cache(VertexArrayKind::ConicGradient),
            "cs_conic_gradient",
            &[],
            device,
            options.precache_flags,
            &shader_list,
        )?;

        let cs_border_segment = LazilyCompiledShader::new(
            ShaderKind::Cache(VertexArrayKind::Border),
            "cs_border_segment",
             &[],
             device,
             options.precache_flags,
            &shader_list,
        )?;

        let cs_border_solid = LazilyCompiledShader::new(
            ShaderKind::Cache(VertexArrayKind::Border),
            "cs_border_solid",
            &[],
            device,
            options.precache_flags,
            &shader_list,
        )?;

        Ok(Shaders {
            cs_blur_a8,
            cs_blur_rgba8,
            cs_border_segment,
            cs_line_decoration,
            cs_fast_linear_gradient,
            cs_linear_gradient,
            cs_radial_gradient,
            cs_conic_gradient,
            cs_border_solid,
            cs_scale,
            cs_svg_filter,
            brush_solid,
            brush_image,
            brush_fast_image,
            brush_blend,
            brush_mix_blend,
            brush_yuv_image,
            brush_linear_gradient,
            brush_opacity,
            brush_opacity_aa,
            cs_clip_rectangle_slow,
            cs_clip_rectangle_fast,
            cs_clip_box_shadow,
            cs_clip_image,
            ps_text_run,
            ps_text_run_dual_source,
            ps_split_composite,
            ps_clear,
            composite_rgba,
            composite_rgba_fast_path,
            composite_yuv,
        })
    }

    fn get_compositing_shader_index(buffer_kind: ImageBufferKind) -> usize {
        buffer_kind as usize
    }

    pub fn get_composite_shader(
        &mut self,
        format: CompositeSurfaceFormat,
        buffer_kind: ImageBufferKind,
        features: CompositeFeatures,
    ) -> &mut LazilyCompiledShader {
        match format {
            CompositeSurfaceFormat::Rgba => {
                if features.contains(CompositeFeatures::NO_UV_CLAMP)
                    && features.contains(CompositeFeatures::NO_COLOR_MODULATION)
                {
                    let shader_index = Self::get_compositing_shader_index(buffer_kind);
                    self.composite_rgba_fast_path[shader_index]
                        .as_mut()
                        .expect("bug: unsupported rgba fast path shader requested")
                } else {
                    let shader_index = Self::get_compositing_shader_index(buffer_kind);
                    self.composite_rgba[shader_index]
                        .as_mut()
                        .expect("bug: unsupported rgba shader requested")
                }
            }
            CompositeSurfaceFormat::Yuv => {
                let shader_index = Self::get_compositing_shader_index(buffer_kind);
                self.composite_yuv[shader_index]
                    .as_mut()
                    .expect("bug: unsupported yuv shader requested")
            }
        }
    }

    pub fn get_scale_shader(
        &mut self,
        buffer_kind: ImageBufferKind,
    ) -> &mut LazilyCompiledShader {
        let shader_index = Self::get_compositing_shader_index(buffer_kind);
        self.cs_scale[shader_index]
            .as_mut()
            .expect("bug: unsupported scale shader requested")
    }

    pub fn get(&
        mut self,
        key: &BatchKey,
        mut features: BatchFeatures,
        debug_flags: DebugFlags,
        device: &Device,
    ) -> &mut LazilyCompiledShader {
        match key.kind {
            BatchKind::SplitComposite => {
                &mut self.ps_split_composite
            }
            BatchKind::Brush(brush_kind) => {
                // SWGL uses a native anti-aliasing implementation that bypasses the shader.
                // Don't consider it in that case when deciding whether or not to use
                // an alpha-pass shader.
                if device.get_capabilities().uses_native_antialiasing {
                    features.remove(BatchFeatures::ANTIALIASING);
                }
                let brush_shader = match brush_kind {
                    BrushBatchKind::Solid => {
                        &mut self.brush_solid
                    }
                    BrushBatchKind::Image(image_buffer_kind) => {
                        if features.contains(BatchFeatures::ANTIALIASING) ||
                            features.contains(BatchFeatures::REPETITION) {

                            self.brush_image[image_buffer_kind as usize]
                                .as_mut()
                                .expect("Unsupported image shader kind")
                        } else {
                            self.brush_fast_image[image_buffer_kind as usize]
                                .as_mut()
                                .expect("Unsupported image shader kind")
                        }
                    }
                    BrushBatchKind::Blend => {
                        &mut self.brush_blend
                    }
                    BrushBatchKind::MixBlend { .. } => {
                        &mut self.brush_mix_blend
                    }
                    BrushBatchKind::LinearGradient => {
                        // SWGL uses a native clip mask implementation that bypasses the shader.
                        // Don't consider it in that case when deciding whether or not to use
                        // an alpha-pass shader.
                        if device.get_capabilities().uses_native_clip_mask {
                            features.remove(BatchFeatures::CLIP_MASK);
                        }
                        // Gradient brushes can optimistically use the opaque shader even
                        // with a blend mode if they don't require any features.
                        if !features.intersects(
                            BatchFeatures::ANTIALIASING
                                | BatchFeatures::REPETITION
                                | BatchFeatures::CLIP_MASK,
                        ) {
                            features.remove(BatchFeatures::ALPHA_PASS);
                        }
                        match brush_kind {
                            BrushBatchKind::LinearGradient => &mut self.brush_linear_gradient,
                            _ => panic!(),
                        }
                    }
                    BrushBatchKind::YuvImage(image_buffer_kind, ..) => {
                        let shader_index =
                            Self::get_compositing_shader_index(image_buffer_kind);
                        self.brush_yuv_image[shader_index]
                            .as_mut()
                            .expect("Unsupported YUV shader kind")
                    }
                    BrushBatchKind::Opacity => {
                        if features.contains(BatchFeatures::ANTIALIASING) {
                            &mut self.brush_opacity_aa
                        } else {
                            &mut self.brush_opacity
                        }
                    }
                };
                brush_shader.get(key.blend_mode, features, debug_flags)
            }
            BatchKind::TextRun(glyph_format) => {
                let text_shader = match key.blend_mode {
                    BlendMode::SubpixelDualSource => self.ps_text_run_dual_source.as_mut().unwrap(),
                    _ => &mut self.ps_text_run,
                };
                text_shader.get(glyph_format, debug_flags)
            }
        }
    }

    pub fn deinit(self, device: &mut Device) {
        for shader in self.cs_scale {
            if let Some(shader) = shader {
                shader.deinit(device);
            }
        }
        self.cs_blur_a8.deinit(device);
        self.cs_blur_rgba8.deinit(device);
        self.cs_svg_filter.deinit(device);
        self.brush_solid.deinit(device);
        self.brush_blend.deinit(device);
        self.brush_mix_blend.deinit(device);
        self.brush_linear_gradient.deinit(device);
        self.brush_opacity.deinit(device);
        self.brush_opacity_aa.deinit(device);
        self.cs_clip_rectangle_slow.deinit(device);
        self.cs_clip_rectangle_fast.deinit(device);
        self.cs_clip_box_shadow.deinit(device);
        self.cs_clip_image.deinit(device);
        self.ps_text_run.deinit(device);
        if let Some(shader) = self.ps_text_run_dual_source {
            shader.deinit(device);
        }
        for shader in self.brush_image {
            if let Some(shader) = shader {
                shader.deinit(device);
            }
        }
        for shader in self.brush_fast_image {
            if let Some(shader) = shader {
                shader.deinit(device);
            }
        }
        for shader in self.brush_yuv_image {
            if let Some(shader) = shader {
                shader.deinit(device);
            }
        }
        self.cs_border_solid.deinit(device);
        self.cs_fast_linear_gradient.deinit(device);
        self.cs_linear_gradient.deinit(device);
        self.cs_radial_gradient.deinit(device);
        self.cs_conic_gradient.deinit(device);
        self.cs_line_decoration.deinit(device);
        self.cs_border_segment.deinit(device);
        self.ps_split_composite.deinit(device);
        self.ps_clear.deinit(device);

        for shader in self.composite_rgba {
            if let Some(shader) = shader {
                shader.deinit(device);
            }
        }
        for shader in self.composite_rgba_fast_path {
            if let Some(shader) = shader {
                shader.deinit(device);
            }
        }
        for shader in self.composite_yuv {
            if let Some(shader) = shader {
                shader.deinit(device);
            }
        }
    }
}

pub type SharedShaders = Rc<RefCell<Shaders>>;
