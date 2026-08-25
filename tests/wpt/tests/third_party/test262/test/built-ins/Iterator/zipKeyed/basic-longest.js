// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zipkeyed
description: >
  Basic Iterator.zipKeyed test with "longest" mode.
includes: [compareArray.js, propertyHelper.js, iteratorZipUtils.js]
features: [joint-iteration]
---*/

function testSequence(inputs, inputsLabel, minLength, maxLength) {
  function test(options, optionsLabel, getPaddingForInput) {
    var label = optionsLabel + ", " + inputsLabel;
    var it = Iterator.zipKeyed(inputs, options);
    assertZippedKeyed(it, inputs, minLength, label);

    var expectedKeys = Object.keys(inputs);

    for (var i = minLength; i < maxLength; i++) {
      var itemLabel = label + ", step " + i;

      var result = it.next();
      var value = result.value;

      // Test IteratorResult structure.
      assertIteratorResult(result, value, false, itemLabel);

      // Ensure resulting object has the expected keys and values.
      assert.compareArray(Object.keys(value), expectedKeys, itemLabel + ": result object keys");

      var expectedValues = Object.values(inputs).map(function (input, j) {
        return i < input.length ? input[i] : getPaddingForInput(j);
      });
      assert.compareArray(Object.values(value), expectedValues, itemLabel + ": result object values");
    }
    assertIteratorResult(it.next(), undefined, true, label + ": after completion");
  }

  test(
    { mode: "longest" },
    "options = { mode: 'longest' }",
    function () {
      return undefined;
    },
  );

  test(
    { mode: "longest", padding: {} },
    "options = { mode: 'longest', padding: {} }",
    function () {
      return undefined;
    },
  );

  test(
    { mode: "longest", padding: { prop_0: "pad" } },
    "options = { mode: 'longest', padding: { prop_0: 'pad' } }",
    function (idx) {
      return idx === 0 ? "pad" : undefined;
    },
  );

  test(
    { mode: "longest", padding: { prop_1: "pad" } },
    "options = { mode: 'longest', padding: { prop_1: 'pad' } }",
    function (idx) {
      return idx === 1 ? "pad" : undefined;
    },
  );

  var padding = {};
  for (var key in inputs) {
    padding[key] = "pad";
  }
  test(
    { mode: "longest", padding: padding },
    "options = { mode: 'longest', padding: { prop_0: 'pad', ..., prop_N: 'pad' } }",
    function (idx) {
      return "pad";
    },
  );

  // Object with many properties.
  padding = new Proxy({}, {
    has(target, key) {
      return key.indexOf('_') !== -1;
    },
    get(target, key, receiver) {
      var split = key.split('_');
      if (split.length !== 2) return undefined;
      return 'pad_' + split[1];
    }
  });
  test(
    { mode: "longest", padding: padding },
    "options = { mode: 'longest', padding: { prop_0: 'pad_1', ..., prop_N: 'pad_N' } }",
    function (idx) {
      return 'pad_' + idx;
    },
  );

}

forEachSequenceCombinationKeyed(testSequence);
