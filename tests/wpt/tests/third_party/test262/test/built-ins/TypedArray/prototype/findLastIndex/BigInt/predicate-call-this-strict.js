// Copyright (C) 2021 Microsoft. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.findlastindex
description: >
  Predicate thisArg as F.call( thisArg, kValue, k, O ) for each array entry.
info: |
  %TypedArray%.prototype.findLastIndex ( predicate [ , thisArg ] )

  ...
  5. Let k be len - 1.
  6. Repeat, while k ≥ 0
    ...
    c. Let testResult be ! ToBoolean(? Call(predicate, thisArg, « kValue, 𝔽(k), O »)).
    ...
flags: [onlyStrict]
includes: [testTypedArray.js]
features: [BigInt, TypedArray, array-find-from-last]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(1));
  var result;

  sample.findLastIndex(function() {
    result = this;
  });

  assert.sameValue(
    result,
    undefined,
    "without thisArg, predicate this is undefined"
  );

  var o = {};
  sample.findLastIndex(function() {
    result = this;
  }, o);

  assert.sameValue(result, o, "thisArg becomes the predicate this");
});
