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

function resolve_corner_style(style, w, h) {
  ['top', 'bottom'].forEach((vSide) => ['left', 'right'].forEach((hSide) => {
    let shape = style[`corner-${vSide}-${hSide}-shape`] ||
        style['corner-shape'] || 'round';
    const match = shape.match(/superellipse\((\.?[0-9]+(.[0-9]+)?)\)/);
    shape = match ? +match[1] : keywords[shape];
    const hWidth = parseFloat(style[`border-${hSide}-width`] || style['border-width'] || 0);
    const vWidth = parseFloat(style[`border-${vSide}-width`] || style['border-width'] || 0);
    let radius =
        style[`border-${vSide}-${hSide}-radius`] || style['border-radius'] || 0;
    if (!Array.isArray(radius))
      radius = [radius, radius];
    if (shape > 1000)
      shape = 1000;
    if (String(radius[0]).endsWith('%'))
      radius[0] = (parseFloat(radius[0]) * w) / 100;
    if (String(radius[1]).endsWith('%'))
      radius[1] = (parseFloat(radius[1]) * h) / 100;
    radius = radius.map(parseFloat);
    style[`corner-${vSide}-${hSide}-shape`] = shape;
    style[`border-${vSide}-${hSide}-radius`] = radius;
    style[`border-${hSide}-width`] = hWidth;
    style[`border-${vSide}-width`] = vWidth;
  }));
  return style;
}
