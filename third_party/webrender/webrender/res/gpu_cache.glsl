/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

uniform HIGHP_SAMPLER_FLOAT sampler2D sGpuCache;

#define VECS_PER_IMAGE_RESOURCE     2

// TODO(gw): This is here temporarily while we have
//           both GPU store and cache. When the GPU
//           store code is removed, we can change the
//           PrimitiveInstance instance structure to
//           use 2x unsigned shorts as vertex attributes
//           instead of an int, and encode the UV directly
//           in the vertices.
ivec2 get_gpu_cache_uv(HIGHP_FS_ADDRESS int address) {
    return ivec2(uint(address) % WR_MAX_VERTEX_TEXTURE_WIDTH,
                 uint(address) / WR_MAX_VERTEX_TEXTURE_WIDTH);
}

vec4[2] fetch_from_gpu_cache_2_direct(ivec2 address) {
    return vec4[2](
        TEXEL_FETCH(sGpuCache, address, 0, ivec2(0, 0)),
        TEXEL_FETCH(sGpuCache, address, 0, ivec2(1, 0))
    );
}

vec4[2] fetch_from_gpu_cache_2(HIGHP_FS_ADDRESS int address) {
    ivec2 uv = get_gpu_cache_uv(address);
    return vec4[2](
        TEXEL_FETCH(sGpuCache, uv, 0, ivec2(0, 0)),
        TEXEL_FETCH(sGpuCache, uv, 0, ivec2(1, 0))
    );
}

vec4 fetch_from_gpu_cache_1_direct(ivec2 address) {
    return texelFetch(sGpuCache, address, 0);
}

vec4 fetch_from_gpu_cache_1(HIGHP_FS_ADDRESS int address) {
    ivec2 uv = get_gpu_cache_uv(address);
    return texelFetch(sGpuCache, uv, 0);
}

#ifdef WR_VERTEX_SHADER

vec4[8] fetch_from_gpu_cache_8(int address) {
    ivec2 uv = get_gpu_cache_uv(address);
    return vec4[8](
        TEXEL_FETCH(sGpuCache, uv, 0, ivec2(0, 0)),
        TEXEL_FETCH(sGpuCache, uv, 0, ivec2(1, 0)),
        TEXEL_FETCH(sGpuCache, uv, 0, ivec2(2, 0)),
        TEXEL_FETCH(sGpuCache, uv, 0, ivec2(3, 0)),
        TEXEL_FETCH(sGpuCache, uv, 0, ivec2(4, 0)),
        TEXEL_FETCH(sGpuCache, uv, 0, ivec2(5, 0)),
        TEXEL_FETCH(sGpuCache, uv, 0, ivec2(6, 0)),
        TEXEL_FETCH(sGpuCache, uv, 0, ivec2(7, 0))
    );
}

vec4[3] fetch_from_gpu_cache_3(int address) {
    ivec2 uv = get_gpu_cache_uv(address);
    return vec4[3](
        TEXEL_FETCH(sGpuCache, uv, 0, ivec2(0, 0)),
        TEXEL_FETCH(sGpuCache, uv, 0, ivec2(1, 0)),
        TEXEL_FETCH(sGpuCache, uv, 0, ivec2(2, 0))
    );
}

vec4[3] fetch_from_gpu_cache_3_direct(ivec2 address) {
    return vec4[3](
        TEXEL_FETCH(sGpuCache, address, 0, ivec2(0, 0)),
        TEXEL_FETCH(sGpuCache, address, 0, ivec2(1, 0)),
        TEXEL_FETCH(sGpuCache, address, 0, ivec2(2, 0))
    );
}

vec4[4] fetch_from_gpu_cache_4_direct(ivec2 address) {
    return vec4[4](
        TEXEL_FETCH(sGpuCache, address, 0, ivec2(0, 0)),
        TEXEL_FETCH(sGpuCache, address, 0, ivec2(1, 0)),
        TEXEL_FETCH(sGpuCache, address, 0, ivec2(2, 0)),
        TEXEL_FETCH(sGpuCache, address, 0, ivec2(3, 0))
    );
}

vec4[4] fetch_from_gpu_cache_4(int address) {
    ivec2 uv = get_gpu_cache_uv(address);
    return vec4[4](
        TEXEL_FETCH(sGpuCache, uv, 0, ivec2(0, 0)),
        TEXEL_FETCH(sGpuCache, uv, 0, ivec2(1, 0)),
        TEXEL_FETCH(sGpuCache, uv, 0, ivec2(2, 0)),
        TEXEL_FETCH(sGpuCache, uv, 0, ivec2(3, 0))
    );
}

//TODO: image resource is too specific for this module

struct ImageResource {
    RectWithEndpoint uv_rect;
    float layer;
    vec3 user_data;
};

ImageResource fetch_image_resource(int address) {
    //Note: number of blocks has to match `renderer::BLOCKS_PER_UV_RECT`
    vec4 data[2] = fetch_from_gpu_cache_2(address);
    RectWithEndpoint uv_rect = RectWithEndpoint(data[0].xy, data[0].zw);
    return ImageResource(uv_rect, data[1].x, data[1].yzw);
}

ImageResource fetch_image_resource_direct(ivec2 address) {
    vec4 data[2] = fetch_from_gpu_cache_2_direct(address);
    RectWithEndpoint uv_rect = RectWithEndpoint(data[0].xy, data[0].zw);
    return ImageResource(uv_rect, data[1].x, data[1].yzw);
}

// Fetch optional extra data for a texture cache resource. This can contain
// a polygon defining a UV rect within the texture cache resource.
// Note: the polygon coordinates are in homogeneous space.
struct ImageResourceExtra {
    vec4 st_tl;
    vec4 st_tr;
    vec4 st_bl;
    vec4 st_br;
};

ImageResourceExtra fetch_image_resource_extra(int address) {
    vec4 data[4] = fetch_from_gpu_cache_4(address + VECS_PER_IMAGE_RESOURCE);
    return ImageResourceExtra(
        data[0],
        data[1],
        data[2],
        data[3]
    );
}

#endif //WR_VERTEX_SHADER
