'use strict';

/**
 * Set up a test for color properties that does not expect exact equality for
 * numeric values within the color. This is necessary for color-mix and
 * relative color syntax, which perform float arithmetic on color channels.
 *
 * @param {number} epsilon       Epsilon for comparison of numeric values.
 */
function set_up_fuzzy_color_test(epsilon) {
  if (!epsilon) {
    epsilon = 0.01;
  }

  // The function
  function fuzzy_compare_colors(input, expected) {
    const colorElementDividers = /( |\(|,)/;
    // Return the string stripped of numbers.
    function getNonNumbers(color) {
      return color.replace(/[0-9\.]/g, '');
    }
    // Return an array of all numbers in the color.
    function getNumbers(color) {
      const result = [];
      color.split(colorElementDividers).forEach(element => {
        const numberElement = parseFloat(element);
        if (!isNaN(numberElement)) {
          result.push(numberElement);
        }
      });
      return result;
    }

    try {
      assert_array_approx_equals(getNumbers(input), getNumbers(expected), epsilon, "Numeric parameters are approximately equal.");
      // Assert that the text of the two colors are equal. i.e. colorSpace, colorFunction and format.
      assert_equals(getNonNumbers(input), getNonNumbers(expected), "Color format is correct.");
    } catch (error) {
      throw `Colors do not match.\nActual:   ${input}\nExpected: ${expected}.\n${error}`
    }
  }

  return fuzzy_compare_colors;
}

/**
 * Test the computed value of a color with some tolerance for numeric parameters.
 *
 * @param {string} specified  A specified value for the color.
 * @param {string} computed   The expected computed color. If omitted, defaults
 *                            to the default test_computed_value test, as
 *                            fuzziness is unnecessary.
 * @param {object} epsilon    Epsilon for comparison of numeric values.
 */

function fuzzy_test_computed_color(specified, computed, epsilon) {
  if (!computed) {
    test_computed_value("color", specified);
    return;
  }

  test_computed_value("color", specified, computed, undefined /* titleExtra */, {comparisonFunction: set_up_fuzzy_color_test(epsilon)});
}

/**
 * Test the parsed value of a color.
 *
 * @param {string} specified  A specified value for the property.
 * @param {string} parsed     The expected parsed color. If omitted, defaults
 *                            to the default test_valid_value test, as
 *                            fuzziness is unnecessary.
 * @param {object} epsilon    Epsilon for comparison of numeric values.
 */
function fuzzy_test_valid_color(specified, parsed, epsilon) {
  if (!parsed) {
    test_valid_value("color", specified);
    return;
  }

  test_valid_value("color", specified, parsed, {comparisonFunction: set_up_fuzzy_color_test(epsilon)});
}
