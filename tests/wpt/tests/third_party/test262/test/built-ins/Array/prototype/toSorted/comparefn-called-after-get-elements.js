// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tosorted
description: >
  Array.prototype.toSorted reads all the array elements before calling compareFn
info: |
  SortIndexedProperties ( obj, len, SortCompare, skipHoles )

  ...
  3. Repeat, while k < len,
    a. Let Pk be ! ToString(𝔽(k)).
    ...
      i. Let kValue be ? Get(O, Pk).
    ...
  4. Sort items using an implementation-defined sequence of
     calls to SortCompare. If any such call returns an abrupt
     completion, stop before performing any further calls to
     SortCompare or steps in this algorithm and return that
     Completion Record.
  ...
features: [change-array-by-copy]
includes: [compareArray.js]
---*/

var getCalls = [];

var arrayLike = {
  length: 3,
  get 0() { getCalls.push(0); return 2; },
  get 1() { getCalls.push(1); return 1; },
  get 2() { getCalls.push(2); return 3; },

}

assert.throws(Test262Error, function() {
  Array.prototype.toSorted.call(arrayLike, () => {
    throw new Test262Error();
  });
});

assert.compareArray(getCalls, [0, 1, 2]);
