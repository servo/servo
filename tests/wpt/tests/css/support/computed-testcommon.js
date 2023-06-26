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
      if (property == "color")
        colorValuesAlmostEqual(readValue, computed, 0.0001, 1);
      else
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

function colorValuesAlmostEqual(color1, color2, float_epsilon, integer_epsilon) {
  // Legacy color formats use integers in [0, 255] and thus will have wider epsilons
  const epsilon = getNonNumbers(color1).startsWith("rgb") ? integer_epsilon : float_epsilon;
  // Colors can be split by spaces, commas or the '(' character.
  const colorElementDividers = /( |\(|,)/;
  // Return the string stripped of numbers.
  function getNonNumbers(color) {
    return color.replace(/[0-9]/g, '');
  }
  // Return an array of all numbers in the color.
  function getNumbers(color) {
    const result = [];
    // const entries = color.split(colorElementDividers);
    color.split(colorElementDividers).forEach(element => {
      const numberElement = parseFloat(element);
      if (!isNaN(numberElement)) {
        result.push(numberElement);
      }
    });
    return result;
  }

  assert_array_approx_equals(getNumbers(color1), getNumbers(color2), epsilon, "Numeric parameters are approximately equal.");
  // Assert that the text of the two colors are equal. i.e. colorSpace, colorFunction and format.
  assert_equals(getNonNumbers(color1), getNonNumbers(color2), "Color format is correct.");
}

function testComputedValueGreaterOrLowerThan(property, specified, expected, titleExtra) {
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

function testTransformValuesCloseTo(specified, epsilon, expectedValue, description)
{
    if(!description) {
      description = `Property ${specified} value expected same with ${expectedValue} in +/-${epsilon}`
    }

    test(function()
    {
        var targetElement = document.getElementById("target");
        targetElement.style.setProperty('transform', "initial");

        /*
        Since we are running many consecutive tests on the same
        element, then it is necessary to reset its property
        to an initial value before actually re-testing it.
        */

        targetElement.style.setProperty('transform', specified);

        var computedCalcValue = getComputedStyle(targetElement)['transform'];

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

        targetElement.style.setProperty('transform', expectedValue);

        var computedExpectedValue = getComputedStyle(targetElement)['transform'];

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
    } , description);

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
