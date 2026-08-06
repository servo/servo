'use strict';

/**
 * Create a test that a SVG geometry-property presentation attribute
 * (https://w3c.github.io/svgwg/svg2-draft/geometry.html) parses a value
 * correctly when set via the SVG-attribute syntax (setAttribute), as
 * distinct from the CSS-property syntax already covered by
 * /css/support/parsing-testcommon.js.
 *
 * Per https://w3c.github.io/svgwg/svg2-draft/types.html#presentation-attribute-css-value,
 * presentation attributes for <length-percentage>-typed geometry properties
 * also accept a bare unitless <number> (treated as a length in user units),
 * which the CSS-property grammar rejects — so the two syntaxes need
 * independent valid/invalid value sets, not a shared one.
 *
 * The document element #target is used to perform the test; it must be an
 * element that supports the given attribute (e.g. <rect> for rx/ry,
 * <circle> for r, <circle>/<ellipse> for cx/cy).
 *
 * @param {string} attribute  The name of the presentation attribute.
 * @param {string} value      A specified attribute value.
 * @param {number} expected   The expected SVGAnimatedLength.baseVal.value.
 * @param {string} computed   The expected getComputedStyle() serialization.
 *                            Defaults to `${expected}px` when omitted.
 */
function test_valid_attribute_value(attribute, value, expected, computed) {
  if (computed === undefined)
    computed = `${expected}px`;

  test(() => {
    const target = document.getElementById('target');
    target.removeAttribute(attribute);
    target.setAttribute(attribute, value);
    assert_equals(target.getAttribute(attribute), value,
      'attribute should round-trip in the DOM');
    assert_equals(target[attribute].baseVal.value, expected,
      'SVGAnimatedLength.baseVal.value');
    assert_equals(getComputedStyle(target)[attribute], computed,
      'getComputedStyle()');
  }, `${attribute}="${value}" should be a valid attribute value`);
}

/**
 * Create a test that an invalid presentation-attribute value for a SVG
 * geometry property is ignored — the attribute keeps the initial computed
 * value (per https://w3c.github.io/svgwg/svg2-draft/geometry.html, "invalid
 * and must be ignored").
 *
 * @param {string} attribute       The name of the presentation attribute.
 * @param {string} value           An invalid specified attribute value.
 * @param {number} initialValue    The expected initial baseVal.value.
 * @param {string} initialComputed The expected initial getComputedStyle()
 *                                 serialization.
 */
function test_invalid_attribute_value(attribute, value, initialValue, initialComputed) {
  test(() => {
    const target = document.getElementById('target');
    target.removeAttribute(attribute);
    assert_equals(target[attribute].baseVal.value, initialValue,
      'sanity check: initial baseVal.value before setAttribute');
    target.setAttribute(attribute, value);
    assert_equals(target.getAttribute(attribute), value,
      'invalid value is still reflected in the DOM attribute string');
    assert_equals(target[attribute].baseVal.value, initialValue,
      'SVGAnimatedLength.baseVal.value should keep the initial value');
    assert_equals(getComputedStyle(target)[attribute], initialComputed,
      'getComputedStyle() should keep the initial value');
  }, `${attribute}="${value}" should be an invalid attribute value (ignored)`);
}
