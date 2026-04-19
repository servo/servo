// Copyright (C) 2021 Microsoft. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.findlastindex
description: >
  Verify predicate this on non-strict mode
info: |
  %TypedArray%.prototype.findLastIndex ( predicate [ , thisArg ] )

  ...
  5. Let k be len - 1.
  6. Repeat, while k ≥ 0
    ...
    c. Let testResult be ! ToBoolean(? Call(predicate, thisArg, « kValue, 𝔽(k), O »)).
  ...
flags: [noStrict]
includes: [testTypedArray.js]
features: [BigInt, TypedArray, array-find-from-last]
---*/

var T = this;

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(1));
  var result;

  sample.findLastIndex(function() {
    result = this;
  });

  assert.sameValue(result, T, "without thisArg, predicate this is the global");

  result = null;
  sample.findLastIndex(function() {
    result = this;
  }, undefined);

  assert.sameValue(result, T, "predicate this is the global when thisArg is undefined");

  var o = {};
  result = null;
  sample.findLastIndex(function() {
    result = this;
  }, o);

  assert.sameValue(result, o, "thisArg becomes the predicate this");
});
