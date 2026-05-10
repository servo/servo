// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.tosorted
description: >
  %TypedArray%.prototype.toSorted verifies that the comparator is callable before reading the length.
info: |
  %TypedArray%.prototype.toSorted ( comparefn )

  1. If comparefn is not undefined and IsCallable(comparefn) is false, throw a TypeError exception.
  2. ...
  3. Let len be ? LengthOfArrayLike(O).
includes: [testTypedArray.js]
features: [TypedArray, change-array-by-copy]
---*/

var invalidComparators = [null, true, false, "", /a/g, 42, 42n, [], {}, Symbol()];

testWithTypedArrayConstructors((TA, makeCtorArg) => {
  const ta = new TA(makeCtorArg([1]));
  for (var i = 0; i < invalidComparators.length; i++) {
    assert.throws(TypeError, function() {
      ta.toSorted(invalidComparators[i]);
    }, String(invalidComparators[i]));
  }
});
