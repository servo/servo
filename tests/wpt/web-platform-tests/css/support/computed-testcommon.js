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
function test_computed_value(property, specified, computed, titleExtra) {
  if (!computed)
    computed = specified;

  test(() => {
    const target = document.getElementById('target');
    assert_true(property in getComputedStyle(target), property + " doesn't seem to be supported in the computed style");
    assert_true(CSS.supports(property, specified), "'" + specified + "' is a supported value for " + property + ".");
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
  }, `Property ${property} value '${specified}'${titleExtra ? ' ' + titleExtra : ''}`);
}

function test_pseudo_computed_value(pseudo, property, specified, computed, titleExtra) {
  if (!computed)
    computed = specified;

  test(() => {
    assert_true(/^::\w+$/.test(pseudo), pseudo + " doesn't seem to be a pseudo-element");
    const styleElement = document.createElement("style");
    document.documentElement.appendChild(styleElement);
    try {
      const {sheet} = styleElement;
      sheet.insertRule("#target" + pseudo + "{}");
      const {style} = sheet.cssRules[0];
      const target = document.getElementById('target');

      assert_true(property in getComputedStyle(target, pseudo), property + " doesn't seem to be supported in the computed style");
      assert_true(CSS.supports(property, specified), "'" + specified + "' is a supported value for " + property + ".");
      style[property] = specified;

      let readValue = getComputedStyle(target, pseudo)[property];
      if (Array.isArray(computed)) {
        assert_in_array(readValue, computed);
      } else {
        assert_equals(readValue, computed);
      }
      if (readValue !== specified) {
        style[property] = '';
        style[property] = readValue;
        assert_equals(getComputedStyle(target, pseudo)[property], readValue,
                      'computed value should round-trip');
      }
    } finally {
      document.documentElement.removeChild(styleElement);
    }
  }, `Property ${property} value '${specified}' in ${pseudo}${titleExtra ? ' ' + titleExtra : ''}`);
}
