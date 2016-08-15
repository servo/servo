/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// draw a circle at position aDesiredPos with a aRadius
vec4 drawCircle(vec2 aPixel, vec2 aDesiredPos, float aRadius, vec3 aColor) {
  float farFromCenter = length(aDesiredPos - aPixel) - aRadius;
  float pixelInCircle = 1.00 - clamp(farFromCenter, 0.0, 1.0);
  return vec4(aColor, pixelInCircle);
}

// Draw a rectangle at aRect fill it with aColor. Only works on non-rotated
// rects.
vec4 drawRect(vec2 aPixel, vec4 aRect, vec3 aColor) {
   // GLSL origin is bottom left, positive Y is up
   bool inRect = (aRect.x <= aPixel.x) && (aPixel.x <= aRect.x + aRect.z) &&
            (aPixel.y >= aRect.y) && (aPixel.y <= aRect.y + aRect.w);
   return vec4(aColor, float(inRect));
}

vec4 draw_dotted_edge() {
  // Everything here should be in device pixels.
  // We want the dot to be roughly the size of the whole border spacing
  float border_spacing = min(vBorders.w, vBorders.z);
  float radius = floor(border_spacing / 2.0);
  float diameter = radius * 2.0;
  // The amount of space between dots. 2.2 was chosen because it looks kind of
  // like firefox.
  float circleSpacing = diameter * 2.2;

  vec2 size = vBorders.zw;
  // Get our position within this specific segment
  vec2 position = vDevicePos - vBorders.xy;

  // Break our position into square tiles with circles in them.
  vec2 circleCount = floor(size / circleSpacing);
  circleCount = max(circleCount, 1.0);

  vec2 distBetweenCircles = size / circleCount;
  vec2 circleCenter = distBetweenCircles / 2.0;

  // Find out which tile this pixel belongs to.
  vec2 destTile = floor(position / distBetweenCircles);
  destTile = destTile * distBetweenCircles;

  // Where we want to draw the actual circle.
  vec2 tileCenter = destTile + circleCenter;

  // Find the position within the tile
  vec2 positionInTile = mod(position, distBetweenCircles);
  vec2 finalPosition = positionInTile + destTile;

  vec4 white = vec4(1.0, 1.0, 1.0, 1.0);
  // See if we should draw a circle or not
  vec4 circleColor = drawCircle(finalPosition, tileCenter, radius, vVerticalColor.xyz);
  return mix(white, circleColor, circleColor.a);
}

vec4 draw_double_edge(float pos, float len) {
  // Devided border to 3 parts, draw color on first and third part,
  // leave second part blank.
  float one_third_len = len / 3.0;

  float in_first_part = step(pos, one_third_len);
  float in_third_part = step(len - one_third_len, pos);

  // The result of this should be 1.0 if we're in the 1st or 3rd part.
  // And 0.0 for the blank part.
  float should_fill = in_first_part + in_third_part;

  float color_weight = step(0.0, vF);
  vec4 color = mix(vHorizontalColor, vVerticalColor, color_weight);

  vec4 white = vec4(1.0, 1.0, 1.0, 1.0);
  return mix(white, color, should_fill);
}

vec4 draw_double_edge_vertical() {
  // Get our position within this specific segment
  float position = vLocalPos.x - vLocalBorders.x;
  return draw_double_edge(position, vLocalBorders.z);
}

vec4 draw_double_edge_horizontal() {
  // Get our position within this specific segment
  float position = vLocalPos.y - vLocalBorders.y;
  return draw_double_edge(position, vLocalBorders.w);
}

vec4 draw_double_edge_with_radius() {
  // Get our position within this specific segment
  float position = distance(vRefPoint, vLocalPos) - vRadii.z;
  float len = vRadii.x - vRadii.z;
  return draw_double_edge(position, len);
}

vec4 draw_double_edge_corner() {
  if (vRadii.x > 0) {
    return draw_double_edge_with_radius();
  }

  bool is_vertical = (vBorderPart == PST_TOP_LEFT) ? vF < 0 : vF >= 0;
  if (is_vertical) {
    return draw_double_edge_vertical();
  } else {
    return draw_double_edge_horizontal();
  }
}

// Our current edge calculation is based only on
// the size of the border-size, but we need to draw
// the dashes in the center of the segment we're drawing.
// This calculates how much to nudge and which axis to nudge on.
vec2 get_dashed_nudge_factor(vec2 dash_size, bool is_corner) {
  if (is_corner) {
    return vec2(0.0, 0.0);
  }

  bool xAxisFudge = vBorders.z > vBorders.w;
  if (xAxisFudge) {
    return vec2(dash_size.x / 2.0, 0);
  }

  return vec2(0.0, dash_size.y / 2.0);
}

vec4 draw_dashed_edge(bool is_corner) {
  // Everything here should be in device pixels.
  // We want the dot to be roughly the size of the whole border spacing
  // 5.5 here isn't a magic number, it's just what mostly looks like FF/Chrome
  // TODO: Investigate exactly what FF does.
  float dash_interval = min(vBorders.w, vBorders.z) * 5.5;
  vec2 edge_size = vec2(vBorders.z, vBorders.w);
  vec2 dash_size = vec2(dash_interval / 2.0, dash_interval / 2.0);
  vec2 position = vDevicePos - vBorders.xy;

  vec2 dash_count = floor(edge_size/ dash_interval);
  vec2 dist_between_dashes = edge_size / dash_count;

  vec2 target_rect_index = floor(position / dist_between_dashes);
  vec2 target_rect_loc = target_rect_index * dist_between_dashes;
  target_rect_loc += get_dashed_nudge_factor(dash_size, is_corner);
  vec4 target_rect = vec4(target_rect_loc, dash_size);

  vec4 white = vec4(1.0, 1.0, 1.0, 1.0);
  vec4 target_colored_rect = drawRect(position, target_rect, vVerticalColor.xyz);
  return mix(white, target_colored_rect, target_colored_rect.a);
}

void draw_dotted_border(void) {
  switch (vBorderPart) {
    // These are the layer tile part PrimitivePart as uploaded by the tiling.rs
    case PST_TOP_LEFT:
    case PST_TOP_RIGHT:
    case PST_BOTTOM_LEFT:
    case PST_BOTTOM_RIGHT:
    {
      // TODO: Fix for corners with a border-radius
      oFragColor = draw_dotted_edge();
      break;
    }
    case PST_BOTTOM:
    case PST_TOP:
    case PST_LEFT:
    case PST_RIGHT:
    {
      oFragColor = draw_dotted_edge();
      break;
    }
  }
}

void draw_dashed_border(void) {
  switch (vBorderPart) {
    // These are the layer tile part PrimitivePart as uploaded by the tiling.rs
    case PST_TOP_LEFT:
    case PST_TOP_RIGHT:
    case PST_BOTTOM_LEFT:
    case PST_BOTTOM_RIGHT:
    {
      // TODO: Fix for corners with a border-radius
      bool is_corner = true;
      oFragColor = draw_dashed_edge(is_corner);
      break;
    }
    case PST_BOTTOM:
    case PST_TOP:
    case PST_LEFT:
    case PST_RIGHT:
    {
      bool is_corner = false;
      oFragColor = draw_dashed_edge(is_corner);
      break;
    }
  }
}

void draw_double_border(void) {
  switch (vBorderPart) {
    // These are the layer tile part PrimitivePart as uploaded by the tiling.rs
    case PST_TOP_LEFT:
    case PST_TOP_RIGHT:
    case PST_BOTTOM_LEFT:
    case PST_BOTTOM_RIGHT:
    {
      oFragColor = draw_double_edge_corner();
      break;
    }
    case PST_BOTTOM:
    case PST_TOP:
    {
      oFragColor = draw_double_edge_horizontal();
      break;
    }
    case PST_LEFT:
    case PST_RIGHT:
    {
      oFragColor = draw_double_edge_vertical();
      break;
    }
  }
}

// TODO: Investigate performance of this shader and see
//       if it's worthwhile splitting it / removing branches etc.
void main(void) {
	if (vRadii.x > 0.0 &&
		(distance(vRefPoint, vLocalPos) > vRadii.x ||
		 distance(vRefPoint, vLocalPos) < vRadii.z)) {
		discard;
	}

  switch (vBorderStyle) {
    case BORDER_STYLE_DASHED:
    {
      draw_dashed_border();
      break;
    }
    case BORDER_STYLE_DOTTED:
    {
      draw_dotted_border();
      break;
    }
    case BORDER_STYLE_OUTSET:
    case BORDER_STYLE_INSET:
    {
      float color = step(0.0, vF);
      oFragColor = mix(vVerticalColor, vHorizontalColor, color);
      break;
    }
    case BORDER_STYLE_NONE:
    case BORDER_STYLE_SOLID:
    {
      float color = step(0.0, vF);
      oFragColor = mix(vHorizontalColor, vVerticalColor, color);
      break;
    }
    case BORDER_STYLE_DOUBLE:
    {
      draw_double_border();
      break;
    }
    default:
    {
      discard;
    }
  }
}
