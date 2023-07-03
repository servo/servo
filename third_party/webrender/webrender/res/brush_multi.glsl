/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// The multi-brush shader is capable of rendering most types of brushes
// if they are enabled via WR_FEATURE_*_BRUSH defines.
// This type of uber-shader comes at a cost so the goal for this is to
// provide opportunities for aggressive batching when the number of draw
// calls so high that reducing the number of draw calls is worth the
// cost of this "uber-shader".


#define WR_FEATURE_MULTI_BRUSH

// These constants must match the BrushShaderKind enum in gpu_types.rs.
#define BRUSH_KIND_SOLID            1
#define BRUSH_KIND_IMAGE            2
#define BRUSH_KIND_TEXT             3
#define BRUSH_KIND_LINEAR_GRADIENT  4
#define BRUSH_KIND_RADIAL_GRADIENT  5
#define BRUSH_KIND_CONIC_GRADIENT   6
#define BRUSH_KIND_BLEND            7
#define BRUSH_KIND_MIX_BLEND        8
#define BRUSH_KIND_YV               9
#define BRUSH_KIND_OPACITY          10

int vecs_per_brush(int brush_kind);


#ifdef WR_FEATURE_TEXT_BRUSH
// Before including the brush source, if we need support for text we override
// the vertex shader's main entry point with one that can call into the text
// shader or the regular brush shaders.

// Foward-declare the new entry point.
void multi_brush_main_vs(
    Instance instance,
    PrimitiveHeader ph,
    Transform transform,
    PictureTask pic_task,
    ClipArea clip_area
);

// Override the default entry point.
#define WR_VERTEX_SHADER_MAIN_FUNCTION multi_brush_main_vs

#endif


#include shared,prim_shared,brush


#ifdef WR_FEATURE_IMAGE_BRUSH
#include brush_image
#endif

#undef VECS_PER_SPECIFIC_BRUSH
#undef WR_BRUSH_VS_FUNCTION
#undef WR_BRUSH_FS_FUNCTION

#ifdef WR_FEATURE_SOLID_BRUSH
#include brush_solid
#endif

#undef VECS_PER_SPECIFIC_BRUSH
#undef WR_BRUSH_VS_FUNCTION
#undef WR_BRUSH_FS_FUNCTION

#ifdef WR_FEATURE_BLEND_BRUSH
#include brush_blend
#endif

#undef VECS_PER_SPECIFIC_BRUSH
#undef WR_BRUSH_VS_FUNCTION
#undef WR_BRUSH_FS_FUNCTION

#ifdef WR_FEATURE_MIX_BLEND_BRUSH
#include brush_mix_blend
#endif

#undef VECS_PER_SPECIFIC_BRUSH
#undef WR_BRUSH_VS_FUNCTION
#undef WR_BRUSH_FS_FUNCTION

#ifdef WR_FEATURE_LINEAR_GRADIENT_BRUSH
#include brush_linear_gradient
#endif

#undef VECS_PER_SPECIFIC_BRUSH
#undef WR_BRUSH_VS_FUNCTION
#undef WR_BRUSH_FS_FUNCTION

#ifdef WR_FEATURE_RADIAL_GRADIENT_BRUSH
#include brush_radial_gradient
#endif

#undef VECS_PER_SPECIFIC_BRUSH
#undef WR_BRUSH_VS_FUNCTION
#undef WR_BRUSH_FS_FUNCTION

#ifdef WR_FEATURE_CONIC_GRADIENT_BRUSH
#include brush_conic_gradient
#endif

#undef VECS_PER_SPECIFIC_BRUSH
#undef WR_BRUSH_VS_FUNCTION
#undef WR_BRUSH_FS_FUNCTION

#ifdef WR_FEATURE_OPACITY_BRUSH
#include brush_opacity
#endif

#undef VECS_PER_SPECIFIC_BRUSH
#undef WR_BRUSH_VS_FUNCTION
#undef WR_BRUSH_FS_FUNCTION

#ifdef WR_FEATURE_TEXT_BRUSH
#include ps_text_run

// Special entry point when text support is needed.
void multi_brush_main_vs(
    Instance instance,
    PrimitiveHeader ph,
    Transform transform,
    PictureTask pic_task,
    ClipArea clip_area
) {
    if (instance.brush_kind == BRUSH_SHADER_KIND_TEXT) {
        text_shader_main(instance, ph, transform, task, clip_area);
    } else {
        brush_shader_main(instance, ph, transform, task, clip_area);
    }
}

#endif

int vecs_per_brush(int brush_kind) {
    switch (brush_kind) {
        // The default arm should never be taken, we let it point to whichever shader
        // is enabled first to satisfy ANGLE validation.
        default:

        #ifdef WR_FEATURE_IMAGE_BRUSH
        case BRUSH_KIND_IMAGE: return VECS_PER_IMAGE_BRUSH;
        #endif

        #ifdef WR_FEATURE_IMAGE_BRUSH
        case BRUSH_KIND_SOLID: return VECS_PER_SOLID_BRUSH;
        #endif

        #ifdef WR_FEATURE_BLEND_BRUSH
        case BRUSH_KIND_BLEND: return VECS_PER_BLEND_BRUSH;
        #endif

        #ifdef WR_FEATURE_MIX_BLEND_BRUSH
        case BRUSH_KIND_MIX_BLEND: return VECS_PER_MIX_BLEND_BRUSH;
        #endif

        #ifdef WR_FEATURE_LINEAR_GRADIENT_BRUSH
        case BRUSH_KIND_LINEAR_GRADIENT: return VECS_PER_LINEAR_GRADIENT_BRUSH;
        #endif

        #ifdef WR_FEATURE_RADIAL_GRADIENT_BRUSH
        case BRUSH_KIND_RADIAL_GRADIENT: return VECS_PER_RADIAL_GRADIENT_BRUSH;
        #endif


        #ifdef WR_FEATURE_CONIC_GRADIENT_BRUSH
        case BRUSH_KIND_CONIC_GRADIENT: return VECS_PER_CONIC_GRADIENT_BRUSH;
        #endif

        #ifdef WR_FEATURE_OPACITY_BRUSH
        case BRUSH_KIND_OPACITY: return VECS_PER_OPACITY_BRUSH;
        #endif
    }
}

#define BRUSH_VS_PARAMS vi, prim_address, local_rect, segment_rect, \
    prim_user_data, specific_resource_address, transform, pic_task, \
    brush_flags, texel_rect


#ifdef WR_VERTEX_SHADER
void multi_brush_vs(
    VertexInfo vi,
    int prim_address,
    RectWithSize local_rect,
    RectWithSize segment_rect,
    ivec4 prim_user_data,
    int specific_resource_address,
    mat4 transform,
    PictureTask pic_task,
    int brush_flags,
    vec4 texel_rect,
    int brush_kind
) {
    switch (brush_kind) {
        default:

        #ifdef WR_FEATURE_IMAGE_BRUSH
        case BRUSH_KIND_IMAGE:
            image_brush_vs(BRUSH_VS_PARAMS);
            break;
        #endif

        #ifdef WR_FEATURE_SOLID_BRUSH
        case BRUSH_KIND_SOLID:
            solid_brush_vs(BRUSH_VS_PARAMS);
            break;
        #endif

        #ifdef WR_FEATURE_BLEND_BRUSH
        case BRUSH_KIND_BLEND:
            blend_brush_vs(BRUSH_VS_PARAMS);
            break;
        #endif

        #ifdef WR_FEATURE_MIX_BLEND_BRUSH
        case BRUSH_KIND_MIX_BLEND:
            mix_blend_brush_vs(BRUSH_VS_PARAMS);
            break;
        #endif

        #ifdef WR_FEATURE_LINEAR_GRADIENT_BRUSH
        case BRUSH_KIND_LINEAR_GRADIENT:
            linear_gradient_brush_vs(BRUSH_VS_PARAMS);
            break;
        #endif

        #ifdef WR_FEATURE_RADIAL_GRADIENT_BRUSH
        case BRUSH_KIND_RADIAL_GRADIENT:
            radial_gradient_brush_vs(BRUSH_VS_PARAMS);
            break;
        #endif

        #ifdef WR_FEATURE_CONIC_GRADIENT_BRUSH
        case BRUSH_KIND_CONIC_GRADIENT:
            conic_gradient_brush_vs(BRUSH_VS_PARAMS);
            break;
        #endif

        #ifdef WR_FEATURE_OPACITY_BRUSH
        case BRUSH_KIND_OPACITY:
            opacity_brush_vs(BRUSH_VS_PARAMS);
            break;
        #endif
    }
}

#endif // WR_VERTEX_SHADER

#ifdef WR_FRAGMENT_SHADER

Fragment multi_brush_fs(int brush_kind) {
    switch (brush_kind) {
        default:

        #ifdef WR_FEATURE_IMAGE_BRUSH
        case BRUSH_KIND_IMAGE: return image_brush_fs();
        #endif

        #ifdef WR_FEATURE_SOLID_BRUSH
        case BRUSH_KIND_SOLID: return solid_brush_fs();
        #endif

        #ifdef WR_FEATURE_BLEND_BRUSH
        case BRUSH_KIND_BLEND: return blend_brush_fs();
        #endif

        #ifdef WR_FEATURE_MIX_BLEND_BRUSH
        case BRUSH_KIND_MIX_BLEND: return mix_blend_brush_fs();
        #endif

        #ifdef WR_FEATURE_LINEAR_GRADIENT_BRUSH
        case BRUSH_KIND_LINEAR_GRADIENT: return linear_gradient_brush_fs();
        #endif

        #ifdef WR_FEATURE_RADIAL_GRADIENT_BRUSH
        case BRUSH_KIND_RADIAL_GRADIENT: return radial_gradient_brush_fs();
        #endif

        #ifdef WR_FEATURE_CONIC_GRADIENT_BRUSH
        case BRUSH_KIND_CONIC_GRADIENT: return conic_gradient_brush_fs();
        #endif

        #ifdef WR_FEATURE_OPACITY_BRUSH
        case BRUSH_KIND_OPACITY: return opacity_brush_fs();
        #endif

        #ifdef WR_FEATURE_TEXT_BRUSH
        case BRUSH_KIND_TEXT: return text_brush_fs();
        #endif
    }
}

#endif
