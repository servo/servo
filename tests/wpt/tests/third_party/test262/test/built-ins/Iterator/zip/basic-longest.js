// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zip
description: >
  Basic Iterator.zip test with "longest" mode.
includes: [compareArray.js, propertyHelper.js, iteratorZipUtils.js]
features: [joint-iteration]
---*/

function testSequence(inputs, inputsLabel, minLength, maxLength) {
  function test(options, optionsLabel, getPaddingForInput) {
    var label = optionsLabel + ", " + inputsLabel;
    var it = Iterator.zip(inputs, options);
    assertZipped(it, inputs, minLength, label);

    for (var i = minLength; i < maxLength; i++) {
      var itemLabel = label + ", step " + i;

      var result = it.next();
      var value = result.value;

      // Test IteratorResult structure.
      assertIteratorResult(result, value, false, itemLabel);

      var expected = inputs.map(function (input, j) {
        return i < input.length ? input[i] : getPaddingForInput(j);
      });
      assert.compareArray(value, expected, itemLabel + ": values");
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
    { mode: "longest", padding: [] },
    "options = { mode: 'longest', padding: [] }",
    function () {
      return undefined;
    },
  );

  test(
    { mode: "longest", padding: ["pad"] },
    "options = { mode: 'longest', padding: ['pad'] }",
    function (idx) {
      return idx === 0 ? "pad" : undefined;
    },
  );

  test(
    { mode: "longest", padding: Array(inputs.length).fill("pad") },
    "options = { mode: 'longest', padding: ['pad', 'pad', ..., 'pad'] }",
    function (idx) {
      return "pad";
    },
  );

  // Yield an infinite amount of numbers.
  var numbers = {
    *[Symbol.iterator]() {
      var i = 0;
      while (true) {
        yield 100 + i++;
      }
    }
  };
  test(
    { mode: "longest", padding: numbers },
    "options = { mode: 'longest', padding: numbers }",
    function (idx) {
      return 100 + idx;
    },
  );
}

forEachSequenceCombination(testSequence);
