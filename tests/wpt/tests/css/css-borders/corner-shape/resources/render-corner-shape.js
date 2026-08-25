class Vector2D {
  /** @type {number} */
  x;
  /** @type {number} */
  y;
  constructor(x, y) {
    this.x = x;
    this.y = y;
  }

  /**
   *
   * @param {number} s
   * @returns {Vector2D}
   */
  scale(s) {
    return new Vector2D(this.x * s, this.y * s);
  }

  length() {
    return Math.hypot(this.x, this.y);
  }

  normalized() {
    const length = this.length();
    return length ? this.scale(1 / length) : this;
  }

  perpendicular() {
    return new Vector2D(-this.y, this.x);
  }

  /**
   *
   * @param {Vector2D} v1
   * @param {Vector2D} v2
   *
   * @returns {number}
   */
  static cross(v1, v2) {
    return v1.x * v2.y - v1.y * v2.x;
  }

  /**
   *
   * @param {DOMPointReadOnly} p1
   * @param {DOMPointReadOnly} p2
   * @returns {Vector2D}
   */
  static fromPoints(p1, p2) {
    return new Vector2D(p2.x - p1.x, p2.y - p1.y);
  }

  /**
   *
   * @param  {...Vector2D} v
   * @returns {Vector2D}
   */
  static concat(...v) {
    return new Vector2D(
        v.reduce((acc, v) => acc + v.x, 0), v.reduce((acc, v) => acc + v.y, 0));
  }
}

/**
 *
 * @param {DOMPointReadOnly} point
 * @param  {...Vector2D} vectors
 *
 * @return {DOMPointReadOnly}
 */
function extend_point(point, ...vectors) {
  const vector = Vector2D.concat(...vectors);
  return new DOMPointReadOnly(point.x + vector.x, point.y + vector.y);
}

/**
 * Calculates the intersection point of two line segments.
 *
 * @param {[DOMPointReadOnly, DOMPointReadOnly]} line0 - The first line segment
 *     [A0, A1].
 * @param {[DOMPointReadOnly, DOMPointReadOnly]} line1 - The second line segment
 *     [B0, B1].
 * @returns {DOMPointReadOnly} The intersection point, or the starting point if
 *     parallel
 */
function intersection_of([a0, a1], [b0, b1]) {
  const a_length = Vector2D.fromPoints(a0, a1);
  const b_length = Vector2D.fromPoints(b0, b1);
  const denom = Vector2D.cross(a_length, b_length);
  if (Math.abs(denom) < 1e-6) {
    return null;
  }

  const a_scale = Vector2D.cross(Vector2D.fromPoints(a0, b0), b_length) / denom;
  return extend_point(a0, a_length.scale(a_scale));
}

/**
 * @param {number} x
 * @param {number} y
 * @param {Vector2D} vectorTowardsStart
 * @param {Vector2D} vectorTowardsEnd
 * @param {number} curvature
 * @param {number} startInset
 * @param {number} endInset
 * @param {"fill" | "stroke"} mode
 */
function get_path_for_corner(
    x, y, vectorTowardsStart, vectorTowardsEnd, superellipse_param, startInset,
    endInset, mode = 'fill') {
  if (superellipse_param === Infinity || vectorTowardsStart.length() < 0.1 ||
      vectorTowardsEnd.length() < 0.1) {
    const path = new Path2D();
    path.moveTo(x, y);
    return path;
  }

  let curvature =
      superellipse_param < -10 ? 0.01 : Math.pow(2, superellipse_param);
  const outer = new DOMPoint(x, y);
  const inverse = curvature < 1;
  const near_notch = superellipse_param < -8;
  if (inverse)
    curvature = 1 / curvature;

  const corner = {
    start: extend_point(outer, vectorTowardsStart),
    end: extend_point(outer, vectorTowardsEnd),
    outer,
    center: extend_point(outer, vectorTowardsStart, vectorTowardsEnd)
  }

  const extendStart = Vector2D.fromPoints(corner.start, corner.center)
                          .normalized()
                          .scale(startInset);
  const extendEnd = Vector2D.fromPoints(corner.end, corner.center)
                        .normalized()
                        .scale(endInset);
  const clipStart = extend_point(corner.start, extendStart);
  const clipOuter = extend_point(outer, extendStart, extendEnd);
  const clipEnd = extend_point(corner.end, extendEnd);
  const convexClampedHalfCorner =
      near_notch ? 0.75 : Math.pow(0.5, 1 / Math.min(2, curvature));
  const clampedHalfCorner =
      inverse ? 1 - convexClampedHalfCorner : convexClampedHalfCorner;
  const unitVectorFromStartToControlPoint =
      new Vector2D(2 * clampedHalfCorner - 0.5, 1.5 - 2 * clampedHalfCorner);
  const singlePixelStrokeVector =
      unitVectorFromStartToControlPoint.normalized().perpendicular();

  const offsets = [
    Vector2D.fromPoints(corner.start, outer)
        .normalized()
        .scale(startInset * singlePixelStrokeVector.x),
    Vector2D.fromPoints(outer, corner.end)
        .normalized()
        .scale(startInset * singlePixelStrokeVector.y),
    Vector2D.fromPoints(corner.end, corner.center)
        .normalized()
        .scale(endInset * singlePixelStrokeVector.y),
    Vector2D.fromPoints(corner.center, corner.start)
        .normalized()
        .scale(endInset * singlePixelStrokeVector.x)
  ];

  const adjusted_start = extend_point(corner.start, offsets[0], offsets[1]);
  const adjusted_outer = extend_point(corner.outer, offsets[1], offsets[2]);
  const adjusted_end = extend_point(corner.end, offsets[2], offsets[3]);
  const adjusted_center = extend_point(corner.center, offsets[3], offsets[0]);
  const curve_center = inverse ? adjusted_outer : adjusted_center;

  const map_point_to_corner = p => extend_point(
      curve_center, Vector2D.fromPoints(curve_center, adjusted_end).scale(p.x),
      Vector2D.fromPoints(curve_center, adjusted_start).scale(p.y));

  const controlPoint = map_point_to_corner(new DOMPointReadOnly(
      unitVectorFromStartToControlPoint.y,
      1 - unitVectorFromStartToControlPoint.x));
  const axisAlignedCornerStart =
      intersection_of([adjusted_start, controlPoint], [clipStart, clipOuter]) ||
      adjusted_start;
  const axisAlignedCornerEnd =
      intersection_of([adjusted_end, controlPoint], [clipOuter, clipEnd]) ||
      adjusted_end;

  const path = new Path2D();
  const lineTo = ({x, y}) => path.lineTo(x, y);

  path.moveTo(axisAlignedCornerStart.x, axisAlignedCornerStart.y);
  if (near_notch) {
    lineTo(adjusted_center);
  } else {
    const t_set = new Set([0, 1]);
    const denom = Math.log(1 / curvature);
    for (let x = Math.min(adjusted_start.x, adjusted_end.x);
         x < Math.max(adjusted_start.x, adjusted_end.x); x++) {
      const t =
          Math.log(
              (x - adjusted_start.x) / (adjusted_end.x - adjusted_start.x)) /
          denom;
      if (t > 0 && t < 1)
        t_set.add(t);
    }
    for (let y = Math.min(adjusted_start.y, adjusted_end.y);
         y < Math.max(adjusted_start.y, adjusted_end.y); y++) {
      const t =
          Math.log(
              1 -
              (y - adjusted_start.y) / (adjusted_end.y - adjusted_start.y)) /
          denom;
      if (t > 0 && t < 1)
        t_set.add(t);
    }

    for (const t of [...t_set].toSorted((a, b) => a - b)) {
      const a = Math.pow(t, 1 / curvature);
      const b = Math.pow(1 - t, 1 / curvature);
      const point = map_point_to_corner(new DOMPointReadOnly(a, b));
      lineTo(point);
    }
  }
  lineTo(axisAlignedCornerEnd);

  if (mode === 'fill')
    lineTo(clipOuter);
  return path;
}

/**
 *
 * @param {CanvasRenderingContext2D} ctx
 * @param {object} style
 * @param {DOMRectReadOnly} borderEdge
 * @param {{left: number, top: number, right: number, bottom: number}} inset
 * @param {"fill" | "stroke"} mode
 */
function draw_contoured_path(
    ctx, style, borderEdge, inset = {
      left: 0,
      top: 0,
      right: 0,
      bottom: 0
    },
    mode = 'fill') {
  const targetEdge = new DOMRectReadOnly(
      borderEdge.left + inset.left, borderEdge.top + inset.top,
      borderEdge.width - inset.left - inset.right,
      borderEdge.height - inset.top - inset.bottom);

  const add_corner =
      (path) => {
        if (!path)
          return;
        const clip_out_path = new Path2D();
        clip_out_path.rect(
            targetEdge.x, targetEdge.y, targetEdge.width, targetEdge.height);

        if (mode === 'fill') {
          clip_out_path.addPath(path);
          ctx.clip(clip_out_path, 'evenodd');
        } else {
          ctx.clip(clip_out_path, 'evenodd');
          ctx.strokeStyle = 'blue';
          ctx.lineWidth = 3;
          ctx.stroke(path);
        }
      }

  ctx.save();
  add_corner(get_path_for_corner(
      borderEdge.right, borderEdge.top,
      new Vector2D(-style['border-top-right-radius'][0], 0),
      new Vector2D(0, style['border-top-right-radius'][1]),
      style['corner-top-right-shape'], inset.top, inset.right, mode));
  add_corner(get_path_for_corner(
      borderEdge.right, borderEdge.bottom,
      new Vector2D(0, -style['border-bottom-right-radius'][1]),
      new Vector2D(-style['border-bottom-right-radius'][0], 0),
      style['corner-bottom-right-shape'], inset.right, inset.bottom, mode));
  add_corner(get_path_for_corner(
      borderEdge.left, borderEdge.bottom,
      new Vector2D(style['border-bottom-left-radius'][0], 0),
      new Vector2D(0, -style['border-bottom-left-radius'][1]),
      style['corner-bottom-left-shape'], inset.bottom, inset.left, mode));
  add_corner(get_path_for_corner(
      borderEdge.left, borderEdge.top,
      new Vector2D(0, style['border-top-left-radius'][1]),
      new Vector2D(style['border-top-left-radius'][0], 0),
      style['corner-top-left-shape'], inset.left, inset.top, mode));
  if (mode === 'fill') {
    ctx.fillRect(
        targetEdge.x, targetEdge.y, targetEdge.width, targetEdge.height);
  }
  ctx.restore();
}


function adjust_radii(s, spread, width, height) {
  const style = {...s};
  style['border-top-left-radius'] = adjusted_radius(
      width, height, ...style['border-top-left-radius'], spread);
  style['border-top-right-radius'] = adjusted_radius(
      width, height, ...style['border-top-right-radius'], spread);
  style['border-bottom-right-radius'] = adjusted_radius(
      width, height, ...style['border-bottom-right-radius'], spread);
  style['border-bottom-left-radius'] = adjusted_radius(
      width, height, ...style['border-bottom-left-radius'], spread);
  return style;
}

function adjusted_radius(width, height, h_radius, v_radius, outset) {
  const coverage = 2 * Math.min(h_radius / width, v_radius / height);
  return [
    adjusted_radius_dimension(coverage, h_radius, outset) - outset,
    adjusted_radius_dimension(coverage, v_radius, outset) - outset
  ];
}

function adjusted_radius_dimension(coverage, radius, outset) {
  radius = Math.max(radius, 0.01);
  if (radius > outset || coverage > 1 || radius) {
    return radius + outset;
  }
  const ratio = radius / outset;
  return radius + outset * (1 - (1 - ratio) ** 3 * (1 - coverage ** 3));
}

/**
 *
 * @param {object} style
 * @param {CanvasRenderingContext2D} ctx
 * @param {number} width
 * @param {number} height
 */
function render(style, ctx, width, height, mode = 'fill') {
  const border_rect = new DOMRect(0, 0, width, height);
  const shadow_spread = style['shadow-spread'] || 0;
  const shadow_offset =
      [style['shadow-offset-x'] || 0, style['shadow-offset-y'] || 0];
  if (shadow_offset[0] || shadow_offset[1] || shadow_spread) {
    ctx.save();
    ctx.translate(...shadow_offset);
    ctx.fillStyle = 'black';
    draw_contoured_path(
        ctx, adjust_radii(style, shadow_spread, width, height), border_rect, {
          left: -shadow_spread,
          top: -shadow_spread,
          right: -shadow_spread,
          bottom: -shadow_spread
        },
        mode);
    ctx.restore();
  }
  ctx.fillStyle = 'purple';
  draw_contoured_path(
      ctx, style, border_rect, {left: 0, top: 0, right: 0, bottom: 0}, mode);
  ctx.fillStyle = 'yellow';
  draw_contoured_path(
      ctx, style, border_rect, {
        left: style['border-left-width'],
        top: style['border-top-width'],
        right: style['border-right-width'],
        bottom: style['border-bottom-width']
      },
      mode);
}

const padding = 100;
function create_ref_canvas(style, width, height, mode = 'fill') {
  const canvas = document.createElement('canvas');
  canvas.width = width + padding * 2;
  canvas.height = height + padding * 2;
  const ctx = canvas.getContext('2d');
  ctx.translate(padding, padding);
  canvas.style.position = 'absolute';
  canvas.style.top = '0';
  canvas.style.left = '0';
  render(style, ctx, width, height, mode);
  return canvas;
}

function create_ref(style, width, height) {
  const div = document.createElement('div');
  div.style.width = width + 'px';
  div.style.height = height + 'px';
  div.style.position = 'relative';
  const fill_canvas = create_ref_canvas(style, width, height, 'fill');
  const stroke_canvas = create_ref_canvas(style, width, height, 'stroke');
  div.appendChild(fill_canvas);
  div.appendChild(stroke_canvas);
  return div;
}

function create_actual(style, width, height) {
  const div = document.createElement('div');
  div.style.width = width + 'px';
  div.style.height = height + 'px';
  div.style.position = 'relative';
  div.style.left = `${padding}px`;
  div.style.top = `${padding}px`;
  for (const prop
           of ['border-left-width', 'border-top-width', 'border-bottom-width',
               'border-right-width']) {
    div.style[prop] = style[prop] + 'px';
  }

  let border_radius = '';
  for (const prop
           of ['border-top-left-radius', 'border-top-right-radius',
               'border-bottom-right-radius', 'border-bottom-left-radius']) {
    border_radius += style[prop][0] + 'px ';
  }
  border_radius += ' / ';
  for (const prop
           of ['border-top-left-radius', 'border-top-right-radius',
               'border-bottom-right-radius', 'border-bottom-left-radius']) {
    border_radius += style[prop][1] + 'px ';
  }

  for (const prop
           of ['corner-top-left-shape', 'corner-top-right-shape',
               'corner-bottom-right-shape', 'corner-bottom-left-shape']) {
    div.style[prop] = `superellipse(${style[prop]})`;
  }

  div.style.boxShadow = `${style['shadow-offset-x'] || 0}px ${
      style['shadow-offset-y'] ||
      0}px 0px ${style['shadow-spread'] || 0}px black`;

  div.style.borderRadius = border_radius;

  div.style.borderColor = 'purple';
  div.style.borderStyle = 'solid';
  div.style.backgroundColor = 'yellow';
  div.style.boxSizing = 'border-box';
  div.id = 'ref';
  const canvas = create_ref_canvas(style, width, height, 'stroke');
  canvas.style.position = 'absolute';
  canvas.style.left = `${- padding - style['border-left-width']}px`;
  canvas.style.top = `${- padding - style['border-top-width']}px`;
  div.appendChild(canvas);
  return div;
}

const corner_shape_keywords = new Map([
  ['infinity', Infinity],
  ['-infinity', -Infinity],
  ['square', Infinity],
  ['notch', -Infinity],
  ['scoop', -1],
  ['round', 1],
  ['bevel', 0],
  ['squircle', 2],
]);

/**
 *
 * @param {URLSearchParams} params
 * @param {"ref" | "actual"} mode
 * @returns
 */
function create_element_with_corner_shape(params, mode) {
  const style = Object.fromEntries(params.entries());
  const width = +(params.get('width') || 200);
  const height = +(params.get('height') || 100);
  for (const prop
           of ['border-left-width', 'border-top-width', 'border-bottom-width',
               'border-right-width', 'shadow-spread', 'shadow-offset-x',
               'shadow-offset-y']) {
    style[prop] = params.has(prop) ? parseFloat(params.get(prop)) : 0;
  }
  for (const prop
           of ['corner-top-left-shape', 'corner-top-right-shape',
               'corner-bottom-right-shape', 'corner-bottom-left-shape']) {
    const value = params.has(prop) ? params.get(prop) :
        params.has('corner-shape') ? params.get('corner-shape') :
                                     1;
    style[prop] = corner_shape_keywords.has(value) ?
        corner_shape_keywords.get(value) :
        parseFloat(value);
  }

  for (const prop
           of ['border-top-left-radius', 'border-top-right-radius',
               'border-bottom-right-radius', 'border-bottom-left-radius']) {
    style[prop] = params.has(prop) ?
        params.get(prop) :
        (params.has('border-radius') ? params.get('border-radius') : '0');
    style[prop] = style[prop].split(',');
    if (style[prop].length === 1) {
      style[prop] = [style[prop][0], style[prop][0]];
    }
    style[prop] = style[prop].map((v, i) => {
      const n = parseFloat(v);
      if (v.endsWith('%')) {
        return (n / 100) * (i ? height : width);
      }
      return n;
    });
  }

  return mode === 'ref' ? create_ref(style, width, height) :
                          create_actual(style, width, height);
}
