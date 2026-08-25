// Helper for SVG WPT tests: assert that two SVGTransform objects are equal.
//
// Per SVG 2 §8.5, an SVGTransform's observable state is (type, angle,
// matrix.{a,b,c,d,e,f}); `angle` is 0 by spec for matrix/translate/scale.
// https://svgwg.org/svg2-draft/coords.html#InterfaceSVGTransform
//
// Default epsilon absorbs tan(45°) round-trip and 32-to-64-bit float
// conversions while still catching real divergence.
//
// Two ways to call it:
//
//   // 1. With an SVGTransform built via the DOM API:
//   const expected = svg.createSVGTransform();
//   expected.setTranslate(50, 0);
//   assert_svg_transform_equals(text.transform.animVal.getItem(0), expected);
//
//   // 2. With a literal descriptor object (no createSVGTransform boilerplate):
//   assert_svg_transform_equals(text.transform.animVal.getItem(0),
//                               {translate: [50, 0]});

function assert_svg_transform_equals(actual, expected, epsilon = 1e-6, description = '') {
  if (!(expected instanceof SVGTransform)) {
    expected = svg_transform_from_descriptor(expected);
  }
  const prefix = description ? description + ': ' : '';
  assert_equals(actual.type, expected.type, prefix + 'type');
  assert_approx_equals(actual.angle, expected.angle, epsilon, prefix + 'angle');
  for (const matrix_component of ['a', 'b', 'c', 'd', 'e', 'f']) {
    assert_approx_equals(actual.matrix[matrix_component], expected.matrix[matrix_component], epsilon,
                         prefix + 'matrix.' + matrix_component);
  }
}

// Build an SVGTransform from a literal descriptor:
//   {translate: [tx, ty?]}        ty defaults to 0
//   {scale: [sx, sy?]}            sy defaults to sx
//   {rotate: [angle, cx?, cy?]}   cx/cy default to 0
//   {skewX: angle} | {skewY: angle}
//   {matrix: [a, b, c, d, e, f]}
function svg_transform_from_descriptor(d) {
  const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
  const t = svg.createSVGTransform();
  if ('translate' in d) {
    const [tx, ty = 0] = d.translate;
    t.setTranslate(tx, ty);
  } else if ('scale' in d) {
    const [sx, sy = sx] = d.scale;
    t.setScale(sx, sy);
  } else if ('rotate' in d) {
    const [angle, cx = 0, cy = 0] = d.rotate;
    t.setRotate(angle, cx, cy);
  } else if ('skewX' in d) {
    t.setSkewX(d.skewX);
  } else if ('skewY' in d) {
    t.setSkewY(d.skewY);
  } else if ('matrix' in d) {
    const m = svg.createSVGMatrix();
    [m.a, m.b, m.c, m.d, m.e, m.f] = d.matrix;
    t.setMatrix(m);
  } else {
    throw new Error('assert_svg_transform_equals: unknown descriptor ' +
                    JSON.stringify(d));
  }
  return t;
}
