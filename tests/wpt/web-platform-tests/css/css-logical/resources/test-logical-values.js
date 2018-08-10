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
    }, `Test that '${property}: ${value}' is supported.`);

    const camelCase = value.replace(/-(.)/g, (match, $1) => $1.toUpperCase());
    for (const writingMode of writingModes) {
      for (const style of writingMode.styles) {
        const writingModeDecl = makeDeclaration(style);
        test(function() {
          const physicalSide = writingMode[camelCase];
          let expected;
          if (physicalSide === writingMode.lineLeft) {
            expected = "left";
          } else if (physicalSide === writingMode.lineRight) {
            expected = "right";
          } else {
            expected = physicalSide;
          }
          testComputedValues(`computed value`,
                             `.test { ${writingModeDecl} }`,
                             [[property, expected]]);
        }, `Test '${property}: ${value}' with '${writingModeDecl}'.`);
      }
    }
  }
}
