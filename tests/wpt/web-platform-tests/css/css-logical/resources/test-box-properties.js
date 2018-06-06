"use strict";
(function(exports) {
  const sheet = document.head.appendChild(document.createElement("style"));

  // Specify size for outer <div> to avoid unconstrained-size warnings
  // when writing-mode of the inner test <div> is vertical-*
  const wrapper = document.body.appendChild(document.createElement("div"));
  wrapper.style.cssText = "width:100px; height: 100px;";
  const testElement = wrapper.appendChild(document.createElement("div"));
  testElement.id = testElement.className = "test";

  // Values to use while testing
  const testValues = {
    "length": ["1px", "2px", "3px", "4px", "5px"],
    "color": ["rgb(1, 1, 1)", "rgb(2, 2, 2)", "rgb(3, 3, 3)", "rgb(4, 4, 4)", "rgb(5, 5, 5)"],
    "border-style": ["solid", "dashed", "dotted", "double", "groove"],
  };

  // Six unique overall writing modes for property-mapping purposes.
  const writingModes = [
    {
      styles: [
        {"writing-mode": "horizontal-tb", "direction": "ltr"},
      ],
      blockStart: "top", blockEnd: "bottom", inlineStart: "left", inlineEnd: "right",
      block: "vertical", inline: "horizontal" },
    {
      styles: [
        {"writing-mode": "horizontal-tb", "direction": "rtl"},
      ],
      blockStart: "top", blockEnd: "bottom", inlineStart: "right", inlineEnd: "left",
      block: "vertical", inline: "horizontal" },
    {
      styles: [
        {"writing-mode": "vertical-rl", "direction": "rtl"},
        {"writing-mode": "sideways-rl", "direction": "rtl"},
      ],
      blockStart: "right", blockEnd: "left", inlineStart: "bottom", inlineEnd: "top",
      block: "horizontal", inline: "vertical" },
    {
      styles: [
        {"writing-mode": "vertical-rl", "direction": "ltr"},
        {"writing-mode": "sideways-rl", "direction": "ltr"},
      ],
      blockStart: "right", blockEnd: "left", inlineStart: "top", inlineEnd: "bottom",
      block: "horizontal", inline: "vertical" },
    {
      styles: [
        {"writing-mode": "vertical-lr", "direction": "rtl"},
        {"writing-mode": "sideways-lr", "direction": "ltr"},
      ],
      blockStart: "left", blockEnd: "right", inlineStart: "bottom", inlineEnd: "top",
      block: "horizontal", inline: "vertical" },
    {
      styles: [
        {"writing-mode": "vertical-lr", "direction": "ltr"},
        {"writing-mode": "sideways-lr", "direction": "rtl"},
      ],
      blockStart: "left", blockEnd: "right", inlineStart: "top", inlineEnd: "bottom",
      block: "horizontal", inline: "vertical" },
  ];

  function testCSSValues(testName, style, expectedValues) {
    for (const [property, value] of expectedValues) {
      assert_equals(style.getPropertyValue(property), value, `${testName}, ${property}`);
    }
  }

  function testComputedValues(testName, rules, expectedValues) {
    sheet.textContent = rules;
    const cs = getComputedStyle(testElement);
    testCSSValues(testName, cs, expectedValues);
    sheet.textContent = "";
  }

  function makeDeclaration(object = {}, replacement = "*") {
    let decl = "";
    for (const [property, value] of Object.entries(object)) {
      decl += `${property.replace("*", replacement)}: ${value}; `;
    }
    return decl;
  }

  /**
   * Creates a group of physical and logical box properties, such as
   *
   * { physical: {
   *     left: "margin-left", right: "margin-right",
   *     top: "margin-top", bottom: "margin-bottom",
   *   }, logical: {
   *     inlineStart: "margin-inline-start", inlineEnd: "margin-inline-end",
   *     blockStart: "margin-block-start", blockEnd: "margin-block-end",
   *   }, type: "length", prerequisites: "...", property: "'margin-*'" }
   *
   * @param {string} property
   *        A string representing the property names, like "margin-*".
   * @param {Object} descriptor
   * @param {string} descriptor.type
   *        Describes the kind of values accepted by the property, like "length".
   *        Must be a key from the `testValues` object.
   * @param {Object={}} descriptor.prerequisites
   *        Represents property declarations that are needed by `property` to work.
   *        For example, border-width properties require a border style.
   */
  exports.createBoxPropertyGroup = function(property, descriptor) {
    const logical = {};
    const physical = {};
    for (const logicalSide of ["inline-start", "inline-end", "block-start", "block-end"]) {
      const camelCase = logicalSide.replace(/-(.)/g, (match, $1) => $1.toUpperCase());
      logical[camelCase] = property.replace("*", logicalSide);
    }
    const isInset = property === "inset-*";
    let prerequisites = "";
    for (const physicalSide of ["left", "right", "top", "bottom"]) {
      physical[physicalSide] = isInset ? physicalSide : property.replace("*", physicalSide);
      prerequisites += makeDeclaration(descriptor.prerequisites, physicalSide);
    }
    return {name, logical, physical, type: descriptor.type, prerequisites, property};
  };

  /**
   * Creates a group of physical and logical sizing properties.
   *
   * @param {string} prefix
   *        One of "", "max-" or "min-".
   */
  exports.createSizingPropertyGroup = function(prefix) {
    return {
      logical: {
        inline: `${prefix}inline-size`,
        block: `${prefix}block-size`,
      },
      physical: {
        horizontal: `${prefix}width`,
        vertical: `${prefix}height`,
      },
      type: "length",
      prerequisites: makeDeclaration({display: "block"}),
      property: (prefix ? prefix.slice(0, -1) + " " : "") + "sizing",
    };
  };

  /**
   * Tests a grup of logical and physical properties in different writing modes.
   *
   * @param {Object} group
   *        An object returned by createBoxPropertyGroup or createSizingPropertyGroup.
   */
  exports.runTests = function(group) {
    const values = testValues[group.type];
    const logicals = Object.values(group.logical);
    const physicals = Object.values(group.physical);

    test(function() {
      const expected = [];
      for (const [i, logicalProp] of logicals.entries()) {
        testElement.style.setProperty(logicalProp, values[i]);
        expected.push([logicalProp, values[i]]);
      }
      testCSSValues("logical properties in inline style", testElement.style, expected);
      testElement.style.cssText = "";
    }, `Test that logical ${group.property} properties are supported.`);

    for (const writingMode of writingModes) {
      for (const style of writingMode.styles) {
        const writingModeDecl = makeDeclaration(style);

        const associated = {};
        for (const [logicalSide, logicalProp] of Object.entries(group.logical)) {
          const physicalProp = group.physical[writingMode[logicalSide]];
          associated[logicalProp] = physicalProp;
          associated[physicalProp] = logicalProp;
        }

        // Test that logical properties are converted to their physical
        // equivalent correctly when all in the group are present on a single
        // declaration, with no overwriting of previous properties and
        // no physical properties present.  We put the writing mode properties
        // on a separate declaration to test that the computed values of these
        // properties are used, rather than those on the same declaration.
        test(function() {
          let decl = group.prerequisites;
          const expected = [];
          for (const [i, logicalProp] of logicals.entries()) {
            decl += `${logicalProp}: ${values[i]}; `;
            expected.push([logicalProp, values[i]]);
            expected.push([associated[logicalProp], values[i]]);
          }
          testComputedValues("logical properties on one declaration, writing " +
                             `mode properties on another, '${writingModeDecl}'`,
                             `.test { ${writingModeDecl} } .test { ${decl} }`,
                             expected);
        }, `Test that logical ${group.property} properties share computed values `
         + `with their physical associates, with '${writingModeDecl}'.`);

        // Test that logical and physical properties are cascaded together,
        // honoring their relative order on a single declaration
        // (a) with a single logical property after the physical ones
        // (b) with a single physical property after the logical ones
        test(function() {
          for (const lastIsLogical of [true, false]) {
            const lasts = lastIsLogical ? logicals : physicals;
            const others = lastIsLogical ? physicals : logicals;
            for (const lastProp of lasts) {
              let decl = writingModeDecl + group.prerequisites;
              const expected = [];
              for (const [i, prop] of others.entries()) {
                decl += `${prop}: ${values[i]}; `;
                const valueIdx = associated[prop] === lastProp ? others.length : i;
                expected.push([prop, values[valueIdx]]);
                expected.push([associated[prop], values[valueIdx]]);
              }
              decl += `${lastProp}: ${values[others.length]}; `;
              testComputedValues(`'${lastProp}' last on single declaration, '${writingModeDecl}'`,
                                 `.test { ${decl} }`,
                                 expected);
            }
          }
        }, `Test that ${group.property} properties honor order of appearance when both `
         + `logical and physical associates are declared, with '${writingModeDecl}'.`);

        // Test that logical and physical properties are cascaded properly when
        // on different declarations
        // (a) with a logical property in the high specificity rule
        // (b) with a physical property in the high specificity rule
        test(function() {
          for (const highIsLogical of [true, false]) {
            let lowDecl = writingModeDecl + group.prerequisites;
            const high = highIsLogical ? logicals : physicals;
            const others = highIsLogical ? physicals : logicals;
            for (const [i, prop] of others.entries()) {
              lowDecl += `${prop}: ${values[i]}; `;
            }
            for (const highProp of high) {
              const highDecl = `${highProp}: ${values[others.length]}; `;
              const expected = [];
              for (const [i, prop] of others.entries()) {
                const valueIdx = associated[prop] === highProp ? others.length : i;
                expected.push([prop, values[valueIdx]]);
                expected.push([associated[prop], values[valueIdx]]);
              }
              testComputedValues(`'${highProp}', two declarations, '${writingModeDecl}'`,
                                 `#test { ${highDecl} } .test { ${lowDecl} }`,
                                 expected);
            }
          }
        }, `Test that ${group.property} properties honor selector specificty when both `
         + `logical and physical associates are declared, with '${writingModeDecl}'.`);
      }
    }
  };
})(window);
