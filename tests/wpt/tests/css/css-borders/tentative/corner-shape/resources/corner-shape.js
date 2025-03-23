/**
 * Use short lines that follow the superellipse formula to generate
 * a path that approximates a superellipse.
 *
 * @param {CanvasRenderingContext2D} ctx
 * @param {number} ax
 * @param {number} ay
 * @param {number} bx
 * @param {number} by
 * @param {number} curvature
 * @param {*} phase
 * @param {*} direction
 * @returns
 */
function add_corner(ctx, ax, ay, bx, by, curvature) {
  const vertical_first = Math.sign(bx - ax) === Math.sign(by - ay);
  function map_point({ x, y }) {
    if (vertical_first) {
      y = 1 - y;
    } else {
      [x, y] = [1 - y, x];
    }

    return [ax + x * (bx - ax), ay + y * (by - ay)];
  }

  if (curvature > 1000) {
    ctx.lineTo(...map_point({ x: 0, y: 1 }));
    ctx.lineTo(...map_point({ x: 1, y: 1 }));
    ctx.lineTo(...map_point({ x: 0, y: 1 }));
    return;
  }

  if (curvature <= 0.001) {
    ctx.lineTo(...map_point({ x: 0, y: 1 }));
    ctx.lineTo(...map_point({ x: 0, y: 0 }));
    ctx.lineTo(...map_point({ x: 1, y: 0 }));
    return;
  }

  function xy_for_t(t) {
    return map_point(superellipse(curvature, t));
  }

  ctx.lineTo(ax, ay);
  const t_values = new Set();
  const antialiasing_offset = 0.25;
  for (
    let x = Math.min(ax, bx) + antialiasing_offset;
    x < Math.max(ax, bx);
    ++x
  ) {
    const nx = (x - ax) / (bx - ax);
    const t = vertical_first
      ? superellipse_t_for_x(nx, curvature)
      : superellipse_t_for_y(1 - nx, curvature);
    if (t > 0 && t < 1) t_values.add(t);
  }

  for (
    let y = Math.min(ay, by) + antialiasing_offset;
    y < Math.max(ay, by);
    ++y
  ) {
    const ny = (y - ay) / (by - ay);
    const t = vertical_first
      ? superellipse_t_for_y(1 - ny, curvature)
      : superellipse_t_for_x(1 - ny, curvature);
    if (t > 0 && t < 1) t_values.add(t);
  }

  for (const t of [...t_values].sort()) {
    const [x, y] = xy_for_t(t);
    ctx.lineTo(x, y);
  }
  ctx.lineTo(bx, by);
}

/**
 *
 * @param {{
 *  'corner-top-left-shape': number,
 *  'corner-top-right-shape': number,
 *  'corner-bottom-right-shape': number,
 *  'corner-bottom-left-shape': number,
 *  'border-top-left-radius': [number, number],
 *  'border-top-right-radius': [number, number],
 *  'border-bottom-left-radius': [number, number],
 *  'border-bottom-right-radius': [number, number],
 *  'border-top-color': string,
 *  'border-right-color': string,
 *  'border-left-color': string,
 *  'border-bottom-color': string,
 *  'border-top-width': number,
 *  'border-right-width': number,
 *  'border-bottom-width': number,
 *  'border-left-width': number,
 * }} style
 * @param {CanvasRenderingContext2D} ctx
 * @param {number} width
 * @param {number} height
 */
function render_rect_with_corner_shapes(style, ctx, width, height) {
  const corner_params = resolve_corner_params(style, width, height);

  function draw_outer_corner(corner, phase = "both", direction) {
    const params = corner_params[corner];
    add_corner(ctx, ...params.outer_rect, params.shape, phase, direction);
  }

  function draw_inner_corner_from_params(params, phase = "both", direction) {
    add_corner(ctx, ...params.inner_rect, params.shape, phase, direction);
  }

  function draw_inner_corner(corner, phase = "both", direction) {
    draw_inner_corner_from_params(corner_params[corner], phase, direction);
  }

  ctx.beginPath();

  draw_outer_corner("top-right");
  draw_outer_corner("bottom-right");
  draw_outer_corner("bottom-left");
  draw_outer_corner("top-left");
  ctx.closePath();
  ctx.clip("nonzero");

  const inner_rect = [
    style["border-left-width"],
    style["border-top-width"],
    width - style["border-right-width"],
    height - style["border-bottom-width"],
  ];
  ctx.save();
  ctx.beginPath();
  draw_inner_corner("top-right");
  ctx.lineTo(inner_rect[2], inner_rect[3]);
  ctx.lineTo(inner_rect[0], inner_rect[3]);
  ctx.lineTo(inner_rect[0], inner_rect[1]);
  ctx.closePath();
  ctx.clip();
  ctx.beginPath();
  draw_inner_corner("bottom-right");
  ctx.lineTo(inner_rect[0], inner_rect[3]);
  ctx.lineTo(inner_rect[0], inner_rect[1]);
  ctx.lineTo(inner_rect[2], inner_rect[1]);
  ctx.closePath();
  ctx.clip();
  ctx.beginPath();
  draw_inner_corner("bottom-left");
  ctx.lineTo(inner_rect[0], inner_rect[1]);
  ctx.lineTo(inner_rect[2], inner_rect[1]);
  ctx.lineTo(inner_rect[2], inner_rect[3]);
  ctx.closePath();
  ctx.clip();
  ctx.beginPath();
  draw_inner_corner("top-left");
  ctx.lineTo(inner_rect[2], inner_rect[1]);
  ctx.lineTo(inner_rect[2], inner_rect[3]);
  ctx.lineTo(inner_rect[0], inner_rect[3]);
  ctx.closePath();
  ctx.clip();
  ctx.fillStyle = style["background-color"];
  ctx.fill();
  ctx.restore();
}
