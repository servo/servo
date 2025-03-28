/**
 *
 * @param {number} curvature
 * @returns {[number, number]}
 */
function offset_for_curvature(curvature) {
  // Find the superellipse's control point.
  // we do that by approximating the superellipse as a quadratic
  // curve that has the same point at t = 0.5.
  if (curvature <= 0.001)
    return [1, -1];
  const {x} = superellipse(Math.min(2, Math.max(0.5, curvature)));
  const [a, b] = [x, 1 - x].map((m) => 2 * m - 0.5);
  const magnitude = Math.hypot(a, b);
  // Normalize a & b
  const norm_a = a / magnitude;
  const norm_b = b / magnitude;
  return [norm_a, -norm_b];
}

function compute_inner_curvature(curvature, outer_length, inner_length) {
  if (curvature === 0)
    return 0;
  if (curvature < 1)
    return 1 /
        compute_inner_curvature(1 / curvature, outer_length, inner_length);
  const target_length = (inner_length - outer_length) / Math.SQRT2;
  return Math.log(0.5) /
      Math.log(
          (Math.pow(0.5, 1 / curvature) * outer_length + target_length) /
          inner_length);
}

/**
 *
 * @param {number} curvature
 * @param {number} t
 * @returns {{x: number, y: number}}
 */
function superellipse(curvature, t = 0.5) {
  // Make concave shapes symmetrical with convex ones.
  if (curvature < 1) {
    const { x, y } = superellipse(1 / curvature, t);
    return { x: 1 - y, y: 1 - x };
  }

  const x = Math.pow(t, 1 / curvature);
  const y = Math.pow(1 - t, 1 / curvature);
  return { x, y };
}

function superellipse_t_for_x(x, curvature) {
  if (curvature < 1) {
    return 1 - superellipse_t_for_y(1 - x, 1 / curvature);
  } else return Math.log(x) / Math.log(1 / curvature);
}

function superellipse_t_for_y(y, curvature) {
  if (curvature < 1) {
    return 1 - superellipse_t_for_x(1 - y, 1 / curvature);
  } else return 1 - Math.log(y) / Math.log(1 / curvature);
}

function resolve_corner_params(style, width, height, outset = null) {
  const params = {
    "top-right": {
      outer: [
        width - style["border-top-right-radius"][0],
        0,
        width,
        style["border-top-right-radius"][1],
      ],
      inset: [style["border-top-width"], style["border-right-width"]],
    },

    "bottom-right": {
      outer: [
        width,
        height - style["border-bottom-right-radius"][1],
        width - style["border-bottom-right-radius"][0],
        height,
      ],
      inset: [style["border-right-width"], style["border-bottom-width"]],
    },

    "bottom-left": {
      outer: [
        style["border-bottom-left-radius"][0],
        height,
        0,
        height - style["border-bottom-left-radius"][1],
      ],
      inset: [style["border-bottom-width"], style["border-left-width"]],
    },

    "top-left": {
      outer: [
        0,
        style["border-top-left-radius"][1],
        style["border-top-left-radius"][0],
        0,
      ],
      inset: [style["border-left-width"], style["border-top-width"]],
    },
  };

  return Object.fromEntries(
      Object.entries(params).map(([corner, {outer, inset}]) => {
        const outer_rect = outer;
        if (outset !== null)
          inset = [-outset, -outset];
        const shape = style[`corner-${corner}-shape`];
        const s1 = Math.sign(outer[2] - outer[0]);
        const s2 = Math.sign(outer[3] - outer[1]);
        const [sw1, sw2] = inset;
        const inner_offset = [s1 * sw1, s2 * sw1, -s1 * sw2, -s2 * sw2];

        const offset = offset_for_curvature(shape);
        if (Math.sign(inner_offset[0]) === Math.sign(inner_offset[1])) {
          offset.reverse();
        }

        const inner_rect = [
          outer_rect[0] + inner_offset[0] * offset[0],
          outer_rect[1] + inner_offset[1] * offset[1],
          outer_rect[2] + inner_offset[2] * offset[1],
          outer_rect[3] + inner_offset[3] * offset[0],
        ];

        let inner_shape = shape;
        if (shape > 2 || shape < 0.5) {
          const outer_length = Math.hypot(
            outer_rect[2] - outer_rect[0], outer_rect[3] - outer_rect[1]);
          const inner_length = Math.hypot(
            inner_rect[2] - inner_rect[0], inner_rect[3] - inner_rect[1])
          inner_shape =
            compute_inner_curvature(shape, outer_length, inner_length);
        }

        return [
          corner,
          {
            outer_rect,
            shape,
            inner_shape,
            inset,
            inner_rect,
            inner_offset,
            inner_shape
          },
        ];
      }));
}
