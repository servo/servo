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

function test_computed_value_greater_or_lower_than(property, specified, expected, titleExtra) {
    test(() => {
      const target = document.getElementById('target');
      assert_true(property in getComputedStyle(target), property + " doesn't seem to be supported in the computed style");
      assert_true(CSS.supports(property, specified), "'" + specified + "' is a supported value for " + property + ".");
      target.style[property] = '';
      target.style[property] = specified;
      let readValue = parseFloat(getComputedStyle(target)[property]);
      assert_true(isFinite(readValue), specified + " expected finite value but got " + readValue)
      assert_false(isNaN(readValue),   specified + " expected finite value but got " + readValue)
      if (expected > 0)
        assert_greater_than_equal(readValue, expected, specified);
      else
        assert_less_than_equal(readValue, expected, specified);
  }, `Property ${property} value '${specified}'${titleExtra ? ' ' + titleExtra : ''}`);
}

function compareValueCloseTo(property_name, calcValue, epsilon, expectedValue, description)
{
    if(!description) {
        description = `Property ${calcValue} value expected same with ${expectedValue} in +/-${epsilon}`
    }

    test(function()
    {
        var targetElement = document.getElementById("target");
        targetElement.style.setProperty(property_name, "initial");

        /*
        Since we are running many consecutive tests on the same
        element, then it is necessary to reset its property
        to an initial value before actually re-testing it.
        */

        targetElement.style.setProperty(property_name, calcValue);

        var computedCalcValue = getComputedStyle(targetElement)[property_name];

        /*
        We first strip out the word "matrix" with the
        opening parenthesis "(" and the closing
        parenthesis ")"
        */

        computedCalcValue = computedCalcValue.replace("matrix(", "").replace(")", "");

        /*
        Then, we split the string at each comma ","
        and store the resulting 6 sub-strings into
        tableSplitComputedCalcValue
        */

        var tableSplitCalcValue = computedCalcValue.split(",");

        /*
        We convert the 6 sub-strings into numerical floating values
        so that mathematical operations (subtraction, absolute value,
        comparison) can be performed.
        */

        tableSplitCalcValue[0] = parseFloat(tableSplitCalcValue[0]);
        tableSplitCalcValue[1] = parseFloat(tableSplitCalcValue[1]);
        tableSplitCalcValue[2] = parseFloat(tableSplitCalcValue[2]);
        tableSplitCalcValue[3] = parseFloat(tableSplitCalcValue[3]);
        tableSplitCalcValue[4] = parseFloat(tableSplitCalcValue[4]);
        tableSplitCalcValue[5] = parseFloat(tableSplitCalcValue[5]);

        /*
        Now, we execute the same steps with the expectedValue
        */

        targetElement.style.setProperty(property_name, expectedValue);

        var computedExpectedValue = getComputedStyle(targetElement)[property_name];

        /*
        We first strip out the word "matrix" with the
        opening parenthesis "(" and the closing
        parenthesis ")"
        */

        computedExpectedValue = computedExpectedValue.replace("matrix(", "").replace(")", "");

        /*
        Then, we split the string at each comma ","
        and store the resulting 6 sub-strings into
        tableSplitComputedCalcValue
        */

        var tableSplitExpectedValue = computedExpectedValue.split(",");

        /*
        We convert the 6 sub-strings into numerical floating values
        so that mathematical operations (subtraction, absolute value,
        comparison) can be performed.
        */

        tableSplitExpectedValue[0] = parseFloat(tableSplitExpectedValue[0]);
        tableSplitExpectedValue[1] = parseFloat(tableSplitExpectedValue[1]);
        tableSplitExpectedValue[2] = parseFloat(tableSplitExpectedValue[2]);
        tableSplitExpectedValue[3] = parseFloat(tableSplitExpectedValue[3]);
        tableSplitExpectedValue[4] = parseFloat(tableSplitExpectedValue[4]);
        tableSplitExpectedValue[5] = parseFloat(tableSplitExpectedValue[5]);

        assert_array_approx_equals(tableSplitCalcValue, tableSplitExpectedValue, epsilon);

        /*
        In this mega-test of 27 sub-tests, we intentionally
        set the tolerance precision (epsilon) to a rather big
        value (0.0001 === 100 millionths). The reason for this
        is we want to verify if browsers and CSS-capable
        applications do the right calculations. We do not want
        to penalize browsers and CSS-capable applications that
        have modest precision (not capable of a 1 millionth
        level precision).
        */

    } , description);

}

/*
    deg
    Degrees. There are 360 degrees in a full circle.

    grad
    Gradians, also known as "gons" or "grades". There are 400 gradians in a full circle.

    rad
    Radians. There are 2π radians in a full circle.

    1rad == 57.295779513°
    https://www.rapidtables.com/convert/number/radians-to-degrees.html

    π == Math.PI == 3.141592653589793

    turn
    Turns. There is 1 turn in a full circle.
*/
    /* Addition of angle units */

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
