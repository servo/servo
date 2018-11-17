'use strict';

/**
 * Create test that a CSS property computes to the expected value.
 * The document element #target is used to perform the test.
 *
 * @param {string} property  The name of the CSS property being tested.
 * @param {string} specified A specified value for the property.
 * @param {string} computed  The expected computed value. If omitted,
                             defaults to specified.
 */
function test_computed_value(property, specified, computed) {
  if (!computed)
    computed = specified;
  test(() => {
    const target = document.getElementById('target');
    if (!getComputedStyle(target)[property])
      return;
    target.style[property] = '';
    target.style[property] = specified;
    assert_equals(getComputedStyle(target)[property], computed);
    if (computed !== specified) {
      target.style[property] = '';
      target.style[property] = computed;
      assert_equals(getComputedStyle(target)[property], computed, 'computed value should round-trip');
    }
  }, "Property " + property + " value '" + specified + "' computes to '" + computed + "'");
}
