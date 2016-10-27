#line 1

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

void discard_pixels_in_rounded_borders(vec2 local_pos) {
  float distanceFromRef = distance(vRefPoint, local_pos);
  if (vRadii.x > 0.0 && (distanceFromRef > vRadii.x || distanceFromRef < vRadii.z)) {
      discard;
  }
}

vec4 get_fragment_color(float distanceFromMixLine, float pixelsPerFragment) {
  // Here we are mixing between the two border colors. We need to convert
  // distanceFromMixLine it to pixel space to properly anti-alias and then push
  // it between the limits accepted by `mix`.
  float colorMix = min(max(distanceFromMixLine / pixelsPerFragment, -0.5), 0.5) + 0.5;
  return mix(vHorizontalColor, vVerticalColor, colorMix);
}

float alpha_for_solid_border(float distance_from_ref,
                             float inner_radius,
                             float outer_radius,
                             float pixels_per_fragment) {
  // We want to start anti-aliasing one pixel in from the border.
  float nudge = pixels_per_fragment;
  inner_radius += nudge;
  outer_radius -= nudge;

  if (distance_from_ref < outer_radius && distance_from_ref > inner_radius) {
    return 1.0;
  }

  float distance_from_border = max(distance_from_ref - outer_radius,
                                   inner_radius - distance_from_ref);

  // Move the distance back into pixels.
  distance_from_border /= pixels_per_fragment;

  // Apply a more gradual fade out to transparent.
  // distance_from_border -= 0.5;

  return 1.0 - smoothstep(0.0, 1.0, distance_from_border);
}

float alpha_for_solid_border_corner(vec2 local_pos,
                                    float inner_radius,
                                    float outer_radius,
                                    float pixels_per_fragment) {
  float distance_from_ref = distance(vRefPoint, local_pos);
  return alpha_for_solid_border(distance_from_ref, inner_radius, outer_radius, pixels_per_fragment);
}

vec4 draw_dotted_edge(vec2 local_pos, vec4 piece_rect, float pixels_per_fragment) {
  // We don't use pixels_per_fragment here, since it can change along the edge
  // of a transformed border edge. We want this calculation to be consistent
  // across the entire edge so that the positioning of the dots stays the same.
  float two_pixels = 2.0 * length(fwidth(vLocalPos.xy));

  // Circle diameter is stroke width, minus a couple pixels to account for anti-aliasing.
  float circle_diameter = max(piece_rect.z - two_pixels, min(piece_rect.z, two_pixels));

  // We want to spread the circles across the edge, but keep one circle diameter at the end
  // reserved for two half-circles which connect to the corners.
  float edge_available = piece_rect.w - (circle_diameter * 2.0);
  float number_of_circles = floor(edge_available / (circle_diameter * 2.0));

  // Here we are initializing the distance from the y coordinate of the center of the circle to
  // the closest end half-circle.
  vec2 relative_pos = local_pos - piece_rect.xy;
  float y_distance = min(relative_pos.y, piece_rect.w - relative_pos.y);

  if (number_of_circles > 0.0) {
    // Spread the circles throughout the edge, to distribute the extra space evenly. We want
    // to ensure that we have at last two pixels of space for each circle so that they aren't
    // touching.
    float space_for_each_circle = ceil(max(edge_available / number_of_circles, two_pixels));

    float first_half_circle_space = circle_diameter;

    float circle_index = (relative_pos.y - first_half_circle_space) / space_for_each_circle;
    circle_index = floor(clamp(circle_index, 0.0, number_of_circles - 1.0));

    float circle_y_pos =
      circle_index * space_for_each_circle + (space_for_each_circle / 2.0) + circle_diameter;
    y_distance = min(abs(circle_y_pos - relative_pos.y), y_distance);
  }

  float distance_from_circle_center = length(vec2(relative_pos.x - (piece_rect.z / 2.0), y_distance));
  float distance_from_circle_edge = distance_from_circle_center - (circle_diameter / 2.0);

  // Don't anti-alias if the circle diameter is small to avoid a blur of color.
  if (circle_diameter < two_pixels && distance_from_circle_edge > 0.0)
    return vec4(0.0);

  // Move the distance back into pixels.
  distance_from_circle_edge /= pixels_per_fragment;

  float alpha = 1.0 - smoothstep(0.0, 1.0, min(1.0, max(0.0, distance_from_circle_edge)));
  return vHorizontalColor * vec4(1.0, 1.0, 1.0, alpha);
}

vec4 draw_dashed_edge(float position, float border_width, float pixels_per_fragment) {
  // TODO: Investigate exactly what FF does.
  float size = border_width * 3.0;
  float segment = floor(position / size);

  float alpha = alpha_for_solid_border(position,
                                       segment * size,
                                       (segment + 1.0) * size,
                                       pixels_per_fragment);

  if (mod(segment + 2.0, 2.0) == 0.0) {
    return vHorizontalColor * vec4(1.0, 1.0, 1.0, 1.0 - alpha);
  } else {
    return vHorizontalColor * vec4(1.0, 1.0, 1.0, alpha);
  }
}

void draw_dashed_or_dotted_border(vec2 local_pos, float distance_from_mix_line) {
  // This is the conversion factor for transformations and device pixel scaling.
  float pixels_per_fragment = length(fwidth(local_pos.xy));

  switch (vBorderPart) {
    // These are the layer tile part PrimitivePart as uploaded by the tiling.rs
    case PST_TOP_LEFT:
    case PST_TOP_RIGHT:
    case PST_BOTTOM_LEFT:
    case PST_BOTTOM_RIGHT:
    {
      oFragColor = get_fragment_color(distance_from_mix_line, pixels_per_fragment);
      if (vRadii.x > 0.0) {
        oFragColor *= vec4(1.0, 1.0, 1.0, alpha_for_solid_border_corner(local_pos,
                                                                  vRadii.z,
                                                                  vRadii.x,
                                                                  pixels_per_fragment));
      }

      break;
    }
    case PST_BOTTOM:
    case PST_TOP: {
      if (vBorderStyle == BORDER_STYLE_DASHED) {
        oFragColor = draw_dashed_edge(vLocalPos.x - vPieceRect.x,
                                      vPieceRect.w,
                                      pixels_per_fragment);
      } else {
        oFragColor = draw_dotted_edge(local_pos.yx, vPieceRect.yxwz, pixels_per_fragment);
      }
      break;
    }
    case PST_LEFT:
    case PST_RIGHT:
    {
      if (vBorderStyle == BORDER_STYLE_DASHED) {
        oFragColor = draw_dashed_edge(vLocalPos.y - vPieceRect.y,
                                      vPieceRect.z,
                                      pixels_per_fragment);
      } else {
        oFragColor = draw_dotted_edge(local_pos.xy, vPieceRect.xyzw, pixels_per_fragment);
      }
      break;
    }
  }
}

vec4 draw_double_edge(float pos,
                      float len,
                      float distance_from_mix_line,
                      float pixels_per_fragment) {
  float total_border_width = len;
  float one_third_width = total_border_width / 3.0;

  // Contribution of the outer border segment.
  float alpha = alpha_for_solid_border(pos,
                                       total_border_width - one_third_width,
                                       total_border_width,
                                       pixels_per_fragment);

  // Contribution of the inner border segment.
  alpha += alpha_for_solid_border(pos, 0.0, one_third_width, pixels_per_fragment);
  return get_fragment_color(distance_from_mix_line, pixels_per_fragment) * vec4(1.0, 1.0, 1.0, alpha);
}

vec4 draw_double_edge_vertical(vec2 local_pos,
                               float distance_from_mix_line,
                               float pixels_per_fragment) {
  // Get our position within this specific segment
  float position = local_pos.x - vLocalRect.x;
  return draw_double_edge(position, vLocalRect.z, distance_from_mix_line, pixels_per_fragment);
}

vec4 draw_double_edge_horizontal(vec2 local_pos,
                                 float distance_from_mix_line,
                                 float pixels_per_fragment) {
  // Get our position within this specific segment
  float position = local_pos.y - vLocalRect.y;
  return draw_double_edge(position, vLocalRect.w, distance_from_mix_line, pixels_per_fragment);
}

vec4 draw_double_edge_corner_with_radius(vec2 local_pos,
                                         float distance_from_mix_line,
                                         float pixels_per_fragment) {
  float total_border_width = vRadii.x - vRadii.z;
  float one_third_width = total_border_width / 3.0;

  // Contribution of the outer border segment.
  float alpha = alpha_for_solid_border_corner(local_pos,
                                              vRadii.x - one_third_width,
                                              vRadii.x,
                                              pixels_per_fragment);

  // Contribution of the inner border segment.
  alpha += alpha_for_solid_border_corner(local_pos,
                                         vRadii.z,
                                         vRadii.z + one_third_width,
                                         pixels_per_fragment);
  return get_fragment_color(distance_from_mix_line, pixels_per_fragment) * vec4(1.0, 1.0, 1.0, alpha);
}

vec4 draw_double_edge_corner(vec2 local_pos,
                             float distance_from_mix_line,
                             float pixels_per_fragment) {
  if (vRadii.x > 0.0) {
      return draw_double_edge_corner_with_radius(local_pos,
                                                 distance_from_mix_line,
                                                 pixels_per_fragment);
  }

  bool is_vertical = (vBorderPart == PST_TOP_LEFT) ? distance_from_mix_line < 0.0 :
                                                     distance_from_mix_line >= 0.0;
  if (is_vertical) {
    return draw_double_edge_vertical(local_pos, distance_from_mix_line, pixels_per_fragment);
  } else {
    return draw_double_edge_horizontal(local_pos, distance_from_mix_line, pixels_per_fragment);
  }
}

void draw_double_border(float distance_from_mix_line, vec2 local_pos) {
  float pixels_per_fragment = length(fwidth(local_pos.xy));
  switch (vBorderPart) {
    // These are the layer tile part PrimitivePart as uploaded by the tiling.rs
    case PST_TOP_LEFT:
    case PST_TOP_RIGHT:
    case PST_BOTTOM_LEFT:
    case PST_BOTTOM_RIGHT:
    {
      oFragColor = draw_double_edge_corner(local_pos, distance_from_mix_line, pixels_per_fragment);
      break;
    }
    case PST_BOTTOM:
    case PST_TOP:
    {
      oFragColor = draw_double_edge_horizontal(local_pos,
                                               distance_from_mix_line,
                                               pixels_per_fragment);
      break;
    }
    case PST_LEFT:
    case PST_RIGHT:
    {
      oFragColor = draw_double_edge_vertical(local_pos,
                                             distance_from_mix_line,
                                             pixels_per_fragment);
      break;
    }
  }
}

void draw_solid_border(float distanceFromMixLine, vec2 localPos) {
  switch (vBorderPart) {
    case PST_TOP_LEFT:
    case PST_TOP_RIGHT:
    case PST_BOTTOM_LEFT:
    case PST_BOTTOM_RIGHT: {
      // This is the conversion factor for transformations and device pixel scaling.
      float pixelsPerFragment = length(fwidth(localPos.xy));
      oFragColor = get_fragment_color(distanceFromMixLine, pixelsPerFragment);

      if (vRadii.x > 0.0) {
        float alpha = alpha_for_solid_border_corner(localPos, vRadii.z, vRadii.x, pixelsPerFragment);
        oFragColor *= vec4(1.0, 1.0, 1.0, alpha);
      }

      break;
    }
    default:
      oFragColor = vHorizontalColor;
      discard_pixels_in_rounded_borders(localPos);
  }
}

// TODO: Investigate performance of this shader and see
//       if it's worthwhile splitting it / removing branches etc.
void main(void) {
#ifdef WR_FEATURE_TRANSFORM
    float alpha = 0.0;
    vec2 local_pos = init_transform_fs(vLocalPos, vLocalRect, alpha);
#else
    vec2 local_pos = vLocalPos;
#endif

#ifdef WR_FEATURE_TRANSFORM
    // TODO(gw): Support other border styles for transformed elements.
    float distance_from_mix_line = (local_pos.x - vPieceRect.x) * vPieceRect.w -
                                   (local_pos.y - vPieceRect.y) * vPieceRect.z;
    distance_from_mix_line /= vPieceRectHypotenuseLength;
#else
    float distance_from_mix_line = vDistanceFromMixLine;
#endif

    switch (vBorderStyle) {
        case BORDER_STYLE_DASHED:
        case BORDER_STYLE_DOTTED:
          draw_dashed_or_dotted_border(local_pos, distance_from_mix_line);
          break;
        case BORDER_STYLE_DOUBLE:
          draw_double_border(distance_from_mix_line, local_pos);
          break;
        case BORDER_STYLE_OUTSET:
        case BORDER_STYLE_INSET:
        case BORDER_STYLE_SOLID:
        case BORDER_STYLE_NONE:
          draw_solid_border(distance_from_mix_line, local_pos);
          break;
        default:
          discard;

    }

#ifdef WR_FEATURE_TRANSFORM
    oFragColor *= vec4(1.0, 1.0, 1.0, alpha);
#endif
}
