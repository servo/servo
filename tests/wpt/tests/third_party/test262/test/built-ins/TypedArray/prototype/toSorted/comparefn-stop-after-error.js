// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.tosorted
description: >
  %TypedArray%.prototype.toSorted doesn't call copmareFn if there is an error
info: |
  %TypedArray%.prototype.toSorted ( compareFn )

  ...
  9. Sort items using an implementation-defined sequence of
     calls to SortCompare. If any such call returns an abrupt
     completion, stop before performing any further calls to
     SortCompare or steps in this algorithm and return that completion.
  ...
includes: [testTypedArray.js]
features: [TypedArray, change-array-by-copy]
---*/

testWithTypedArrayConstructors((TA, makeCtorArg) => {
  var calls = 0;
  var ta = new TA(makeCtorArg([3, 1, 2]));
  try {
    ta.toSorted(() => {
      ++calls;
      if (calls === 1) {
        throw new Test262Error();
      }
    });
  } catch (e) {}
  assert.sameValue(calls <= 1, true, "compareFn is not called after an error");
});
