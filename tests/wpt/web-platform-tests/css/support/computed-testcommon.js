'use strict';

/**
 * Create test that a CSS property computes to the expected value.
 * The document element #target is used to perform the test.
 *
 * @param {string} property  The name of the CSS property being tested.
 * @param {string} specified A specified value for the property.
 * @param {string|array} computed  The expected computed value,
 *                                 or an array of permitted computed value.
 *                                 If omitted, defaults to specified.
 */
function test_computed_value(property, specified, computed) {
  if (!computed)
    computed = specified;

  let computedDesc = "'" + computed + "'";
  if (Array.isArray(computed))
    computedDesc = '[' + computed.map(e => "'" + e + "'").join(' or ') + ']';

  test(() => {
    const target = document.getElementById('target');
    assert_true(property in getComputedStyle(target), property + " doesn't seem to be supported in the computed style");
    target.style[property] = '';
    target.style[property] = specified;

    let readValue = getComputedStyle(target)[property];
    if (Array.isArray(computed)) {
      assert_in_array(readValue, computed);
    } else {
      assert_equals(readValue, computed);
    }
    if (readValue !== specified) {
      target.style[property] = '';
      target.style[property] = readValue;
      assert_equals(getComputedStyle(target)[property], readValue,
                    'computed value should round-trip');
    }
  }, "Property " + property + " value '" + specified + "' computes to " +
     computedDesc);
}
