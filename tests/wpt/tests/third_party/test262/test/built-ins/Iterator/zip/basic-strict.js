// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zip
description: >
  Basic Iterator.zip test with "strict" mode.
includes: [compareArray.js, propertyHelper.js, iteratorZipUtils.js]
features: [joint-iteration]
---*/

function testSequence(inputs, inputsLabel, minLength, maxLength) {
  function test(options, optionsLabel) {
    var label = optionsLabel + ", " + inputsLabel;
    var it = Iterator.zip(inputs, options);
    assertZipped(it, inputs, minLength, label);

    if (minLength === maxLength) {
      assertIteratorResult(it.next(), undefined, true, label + ": after completion");
    } else {
      assert.throws(TypeError, function() {
        it.next();
      }, label + " should throw after " + minLength + " items.");
    }
  }

  test({ mode: "strict" }, "options = { mode: 'strict' }");
}

forEachSequenceCombination(testSequence);
