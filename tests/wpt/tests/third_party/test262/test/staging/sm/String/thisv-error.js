// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

function testName(thisv) {
  var failures = [
    // Not a function
    "length",
    // TODO: Different implementation
    "toString",
    "toSource",
    "valueOf",
    // Aliases
    "trimLeft",
    "trimRight",
  ]

  var keys = Object.getOwnPropertyNames(String.prototype);
  for (var key of keys) {
    if (key === "constructor") {
      assert.sameValue(String.prototype[key].call(thisv), "");
    } else if (failures.includes(key)) {
      assert.throws(TypeError, () => String.prototype[key].call(thisv), key);
    } else {
      var expected = `String.prototype.${key} called on incompatible ${thisv}`;
      assert.throws(TypeError, () => String.prototype[key].call(thisv), expected)
    }
  }
}
testName(null);
testName(undefined);

// On-off test for Symbol.iterator
function testIterator(thisv) {
  assert.throws(TypeError, () => String.prototype[Symbol.iterator].call(thisv),
    `String.prototype[Symbol.iterator] called on incompatible ${thisv}`);
}
testIterator(null);
testIterator(undefined);

