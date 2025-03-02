// Copyright 2025 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

const keywords = {
  notch: 0,
  scoop: 0.5,
  bevel: 1,
  round: 2,
  squircle: 4,
  straight: 1000,
};

function superellipse_at(curvature, t = 0.5) {
  return Math.pow(t, 1 / curvature)
}

/**
 * @param {number} s
 * @param {number} t
 * @returns {x: number, y: number}
 */
function se(s, t = 0.5) {
  const curvature = Math.pow(2, s);
  const x = superellipse_at(curvature);
  const y = superellipse_at(curvature, 1 - t);
  return {x, y};
}


/**
 *
 * @param {number} curvature
 * @returns number
 */
function offset_for_curvature(curvature) {
  if (curvature === 0)
    return 1;
  if (curvature >= 2)
    return 0;
  // Find the approximate slope & magnitude of the superellipse's tangent
  const a = superellipse_at(curvature);
  const b = 1 - a;
  const slope = a / b;
  const magnitude = Math.hypot(a, b);
  // Normalize a & b
  const norm_a = a / magnitude;
  const norm_b = b / magnitude;

  // The outer normal offset is the intercept of the line
  // parallel to the tangent, at distance.

  return norm_b + slope * (norm_a - 1);
}


function resolve_corner_style(style, width, height) {
  ['top', 'bottom'].forEach((vSide) => ['left', 'right'].forEach((hSide) => {
    let shape = style[`corner-${vSide}-${hSide}-shape`] ||
        style['corner-shape'] || 'round';
    const match = shape.match(/superellipse\((\.?[0-9]+(.[0-9]+)?)\)/);
    shape = match ? +match[1] : keywords[shape];
    const hWidth = style[`border-${hSide}-width`] ?? style['border-width'] ?? 0;
    const vWidth = style[`border-${vSide}-width`] ?? style['border-width'] ?? 0;
    let radius =
        style[`border-${vSide}-${hSide}-radius`] ?? style['border-radius'];
    if (!Array.isArray(radius))
      radius = [radius, radius];
    if (shape > 1000)
      shape = 1000;
    if (shape < 0.00000001)
      shape = 0.00000001;
    if (String(radius[0]).endsWith('%'))
      radius[0] = (parseFloat(radius[0]) * width) / 100;
    if (String(radius[1]).endsWith('%'))
      radius[1] = (parseFloat(radius[1]) * height) / 100;
    radius = radius.map(r => parseFloat(r));
    ;
    style[`corner-${vSide}-${hSide}-shape`] = shape;
    const offset = offset_for_curvature(shape);
    radius = [
      Math.min(Math.max(radius[0], hWidth), width / 2 - hWidth * offset),
      Math.min(Math.max(radius[1], vWidth), height / 2 - vWidth * offset)
    ];
    style[`border-${vSide}-${hSide}-radius`] = radius;
    style[`border-${hSide}-width`] = hWidth;
    style[`border-${vSide}-width`] = vWidth;
  }));
  return style;
}
