/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


#ifdef WR_VERTEX_SHADER
#define VECS_PER_RENDER_TASK        2U

uniform HIGHP_SAMPLER_FLOAT sampler2D sRenderTasks;

struct RenderTaskCommonData {
    RectWithSize task_rect;
    float texture_layer_index;
};

struct RenderTaskData {
    RenderTaskCommonData common_data;
    vec3 user_data;
};

RenderTaskData fetch_render_task_data(int index) {
    ivec2 uv = get_fetch_uv(index, VECS_PER_RENDER_TASK);

    vec4 texel0 = TEXEL_FETCH(sRenderTasks, uv, 0, ivec2(0, 0));
    vec4 texel1 = TEXEL_FETCH(sRenderTasks, uv, 0, ivec2(1, 0));

    RectWithSize task_rect = RectWithSize(
        texel0.xy,
        texel0.zw
    );

    RenderTaskCommonData common_data = RenderTaskCommonData(
        task_rect,
        texel1.x
    );

    RenderTaskData data = RenderTaskData(
        common_data,
        texel1.yzw
    );

    return data;
}

RenderTaskCommonData fetch_render_task_common_data(int index) {
    ivec2 uv = get_fetch_uv(index, VECS_PER_RENDER_TASK);

    vec4 texel0 = TEXEL_FETCH(sRenderTasks, uv, 0, ivec2(0, 0));
    vec4 texel1 = TEXEL_FETCH(sRenderTasks, uv, 0, ivec2(1, 0));

    RectWithSize task_rect = RectWithSize(
        texel0.xy,
        texel0.zw
    );

    RenderTaskCommonData data = RenderTaskCommonData(
        task_rect,
        texel1.x
    );

    return data;
}

#define PIC_TYPE_IMAGE          1
#define PIC_TYPE_TEXT_SHADOW    2

/*
 The dynamic picture that this brush exists on. Right now, it
 contains minimal information. In the future, it will describe
 the transform mode of primitives on this picture, among other things.
 */
struct PictureTask {
    RenderTaskCommonData common_data;
    float device_pixel_scale;
    vec2 content_origin;
};

PictureTask fetch_picture_task(int address) {
    RenderTaskData task_data = fetch_render_task_data(address);

    PictureTask task = PictureTask(
        task_data.common_data,
        task_data.user_data.x,
        task_data.user_data.yz
    );

    return task;
}

#define CLIP_TASK_EMPTY 0x7FFF

struct ClipArea {
    RenderTaskCommonData common_data;
    float device_pixel_scale;
    vec2 screen_origin;
};

ClipArea fetch_clip_area(int index) {
    ClipArea area;

    if (index >= CLIP_TASK_EMPTY) {
        RectWithSize rect = RectWithSize(vec2(0.0), vec2(0.0));

        area.common_data = RenderTaskCommonData(rect, 0.0);
        area.device_pixel_scale = 0.0;
        area.screen_origin = vec2(0.0);
    } else {
        RenderTaskData task_data = fetch_render_task_data(index);

        area.common_data = task_data.common_data;
        area.device_pixel_scale = task_data.user_data.x;
        area.screen_origin = task_data.user_data.yz;
    }

    return area;
}

#endif //WR_VERTEX_SHADER
