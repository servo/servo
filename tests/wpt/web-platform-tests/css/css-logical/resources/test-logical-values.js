import {
  testElement,
  writingModes,
  testCSSValues,
  testComputedValues,
  makeDeclaration
} from "./test-shared.js";

/**
 * Tests flow-relative values for a CSS property in different writing modes.
 *
 * @param {string} property
 *        The CSS property to be tested.
 * @param {string[]} values
 *        An array with the flow-relative values to be tested.
 */
export function runTests(property, values) {
  for (const value of values) {
    test(function() {
      const {style} = testElement;
      style.cssText = "";
      style.setProperty(property, value);
      testCSSValues("logical values in inline style", style, [[property, value]]);
      testComputedValues("logical values in computed style", style, [[property, value]]);
    }, `Test that '${property}: ${value}' is supported.`);
  }
}
