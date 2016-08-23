#line 1
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

struct Border {
    PrimitiveInfo info;
    vec4 verticalColor;
    vec4 horizontalColor;
    vec4 radii;
    uvec4 border_style_trbl;
};

layout(std140) uniform Items {
    Border borders[WR_MAX_PRIM_ITEMS];
};

uint get_border_style(Border a_border, uint a_edge) {
  switch (a_edge) {
    case PST_TOP:
    case PST_TOP_LEFT:
      return a_border.border_style_trbl.x;
    case PST_BOTTOM_LEFT:
    case PST_LEFT:
      return a_border.border_style_trbl.z;
    case PST_BOTTOM_RIGHT:
    case PST_BOTTOM:
      return a_border.border_style_trbl.w;
    case PST_TOP_RIGHT:
    case PST_RIGHT:
      return a_border.border_style_trbl.y;
  }
}

void main(void) {
    Border border = borders[gl_InstanceID];
#ifdef WR_FEATURE_TRANSFORM
    TransformVertexInfo vi = write_transform_vertex(border.info);
    vLocalPos = vi.local_pos;

    // Local space
    vLocalRect = vi.clipped_local_rect;
#else
    VertexInfo vi = write_vertex(border.info);
    vLocalPos = vi.local_clamped_pos.xy;

    // Local space
    vLocalRect = border.info.local_rect;
#endif

    // This is what was currently sent.
    vVerticalColor = border.verticalColor;
    vHorizontalColor = border.horizontalColor;

    // Just our boring radius position.
    vRadii = border.radii;

    float x0, y0, x1, y1;
    vBorderPart = border.info.layer_tile_part.z;
    switch (vBorderPart) {
        // These are the layer tile part PrimitivePart as uploaded by the tiling.rs
        case PST_TOP_LEFT:
            x0 = border.info.local_rect.x;
            y0 = border.info.local_rect.y;
            // These are width / heights
            x1 = border.info.local_rect.x + border.info.local_rect.z;
            y1 = border.info.local_rect.y + border.info.local_rect.w;

            // The radius here is the border-radius. This is 0, so vRefPoint will
            // just be the top left (x,y) corner.
            vRefPoint = vec2(x0, y0) + vRadii.xy;
            break;
        case PST_TOP_RIGHT:
            x0 = border.info.local_rect.x + border.info.local_rect.z;
            y0 = border.info.local_rect.y;
            x1 = border.info.local_rect.x;
            y1 = border.info.local_rect.y + border.info.local_rect.w;
            vRefPoint = vec2(x0, y0) + vec2(-vRadii.x, vRadii.y);
            break;
        case PST_BOTTOM_LEFT:
            x0 = border.info.local_rect.x;
            y0 = border.info.local_rect.y + border.info.local_rect.w;
            x1 = border.info.local_rect.x + border.info.local_rect.z;
            y1 = border.info.local_rect.y;
            vRefPoint = vec2(x0, y0) + vec2(vRadii.x, -vRadii.y);
            break;
        case PST_BOTTOM_RIGHT:
            x0 = border.info.local_rect.x;
            y0 = border.info.local_rect.y;
            x1 = border.info.local_rect.x + border.info.local_rect.z;
            y1 = border.info.local_rect.y + border.info.local_rect.w;
            vRefPoint = vec2(x1, y1) + vec2(-vRadii.x, -vRadii.y);
            break;
        case PST_TOP:
        case PST_LEFT:
        case PST_BOTTOM:
        case PST_RIGHT:
            vRefPoint = border.info.local_rect.xy;
            x0 = border.info.local_rect.x;
            y0 = border.info.local_rect.y;
            x1 = border.info.local_rect.x + border.info.local_rect.z;
            y1 = border.info.local_rect.y + border.info.local_rect.w;
            break;
    }

    vBorderStyle = get_border_style(border, vBorderPart);

    // y1 - y0 is the height of the corner / line
    // x1 - x0 is the width of the corner / line.
    float width = x1 - x0;
    float height = y1 - y0;
#ifdef WR_FEATURE_TRANSFORM
    vSizeInfo = vec4(x0, y0, width, height);
#else
    // This is just a weighting of the pixel colors it seems?
    vF = (vi.local_clamped_pos.x - x0) * height - (vi.local_clamped_pos.y - y0) * width;

    // These are in device space
    vDevicePos = vi.global_clamped_pos;

    // These are in device space
    vBorders = border.info.local_rect * uDevicePixelRatio;
#endif
}
