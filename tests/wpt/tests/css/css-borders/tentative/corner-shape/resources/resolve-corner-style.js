// Copyright 2025 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

const keywords = {
  notch: -16,
  scoop: -1,
  bevel: 0,
  round: 1,
  squircle: 2,
  square: 16,
};

function resolve_corner_style(style, w, h) {
  ['top', 'bottom'].forEach((vSide) => ['left', 'right'].forEach((hSide) => {
    let shape_param = style[`corner-${vSide}-${hSide}-shape`] ||
        style['corner-shape'] || 'round';
    const match = shape_param.match(/superellipse\((-?(infinity|[0-9]*(\.[0-9]+)?))\)/i);
    shape_param = match ? match[1] : keywords[shape_param];
    const hWidth = parseFloat(style[`border-${hSide}-width`] || style['border-width'] || 0);
    const vWidth = parseFloat(style[`border-${vSide}-width`] || style['border-width'] || 0);
    let radius =
        style[`border-${vSide}-${hSide}-radius`] || style['border-radius'] || 0;
    if (!Array.isArray(radius))
      radius = [radius, radius];
    let shape = 0;
    if (shape_param >= keywords["square"] || shape_param == "infinity")
      shape = 1000;
    else if (shape_param <= keywords["notch"] || shape_param == "-infinity")
      shape = 0;
    else
     shape = Math.pow(2, shape_param);
    if (String(radius[0]).endsWith('%'))
      radius[0] = (parseFloat(radius[0]) * w) / 100;
    if (String(radius[1]).endsWith('%'))
      radius[1] = (parseFloat(radius[1]) * h) / 100;
    radius = radius.map(parseFloat);
    style[`corner-${vSide}-${hSide}-shape`] = shape;
    style[`border-${vSide}-${hSide}-radius`] = radius;
    style[`border-${hSide}-width`] = hWidth;
    style[`border-${vSide}-width`] = vWidth;
    style[`border-${hSide}-color`] = style[`border-${hSide}-color`] || style[`border-color`];
    style[`border-${vSide}-color`] = style[`border-${vSide}-color`] || style[`border-color`];
    if ('box-shadow' in style) {
      const shadows = style['box-shadow'].split(",");
      style.shadow = [];
      const boxShadowRegex = /(?:(-?\d+(?:\.\d+)?)px)\s+(?:(-?\d+(?:\.\d+)?)px)\s+(?:(-?\d+(?:\.\d+)?)(?:px)?)?(?:\s+(?:(-?\d+(?:\.\d+)?)px))?\s+([^\$\s]*(\s+inset)?)/i;
      for (const shadow of shadows.toReversed()) {
        const parsed = shadow.match(boxShadowRegex)
        if (parsed)
          style.shadow.push({offset: [parseFloat(parsed[1]), parseFloat(parsed[2])], blur: parsed[3], spread: parsed[4], color: parsed[5] || "black", inset: !!parsed[6] });
      }
    }
  }));
  return style;
}
