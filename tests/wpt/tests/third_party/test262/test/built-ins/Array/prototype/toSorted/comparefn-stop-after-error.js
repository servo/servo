// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tosorted
description: >
  Array.prototype.toSorted doesn't call compareFn if there is an error
info: |
  Array.prototype.toSorted ( compareFn )

  ...
  7. Sort items using an implementation-defined sequence of
     calls to SortCompare. If any such call returns an abrupt
     completion, stop before performing any further calls to
     SortCompare or steps in this algorithm and return that completion.
  ...
features: [change-array-by-copy]
---*/

var arrayLike = {
  length: 1,
  get 0() { throw new Test262Error(); },
};

var called = false;
assert.throws(Test262Error, function() {
  Array.prototype.toSorted.call(arrayLike, () => {
    called = true;
  });
});
assert.sameValue(called, false);

called = 0;
assert.throws(Test262Error, function() {
  [1, 2, 3].toSorted(() => {
    ++called;
    if (called === 1) {
      throw new Test262Error();
    }
  });
});
assert.sameValue(called, 1);
