// Copyright (C) 2021 Microsoft. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.findlastindex
description: >
  Predicate is not called on an empty instance
info: |
  %TypedArray%.prototype.findLastIndex ( predicate [ , thisArg ] )

  ...
  6. Repeat, while k ≥ 0
    ...
    c. Let testResult be ! ToBoolean(? Call(predicate, thisArg, « kValue, 𝔽(k), O »)).
    ...
  7. Return -1.
includes: [testTypedArray.js]
features: [TypedArray, array-find-from-last]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(0));
  var called = false;

  var predicate = function() {
    called = true;
    return true;
  };

  var result = sample.findLastIndex(predicate);

  assert.sameValue(
    called, false,
    "does not call predicate"
  );
  assert.sameValue(
    result, -1,
    "returns -1 on an empty instance"
  );
});
