// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tosorted
description: >
  Array.prototype.toSorted verifies that the comparator is callable before reading the length.
info: |
  Array.prototype.toSorted ( compareFn )

  1. If comparefn is not undefined and IsCallable(comparefn) is false, throw a TypeError exception.
  2. ...
  3. Let len be ? LengthOfArrayLike(O).
features: [change-array-by-copy]
---*/

var getLengthThrow = {
  get length() {
    throw new Test262Error("IsCallable(comparefn) should be observed before this.length");
  }
};

var invalidComparators = [null, true, false, "", /a/g, 42, 42n, [], {}, Symbol()];

for (var i = 0; i < invalidComparators.length; i++) {
  assert.throws(TypeError, function() {
    [1].toSorted(invalidComparators[i]);
  }, String(invalidComparators[i]) + " on an array");

  assert.throws(TypeError, function() {
    Array.prototype.toSorted.call(getLengthThrow, invalidComparators[i]);
  }, String(invalidComparators[i]) + " on an object whose 'length' throws");
}
