// Copyright (C) 2021 Microsoft. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.findlast
description: >
  Predicate called as F.call( thisArg, kValue, k, O ) for each array entry.
info: |
  %TypedArray%.prototype.findLast (predicate [ , thisArg ] )

  5. Let k be len - 1.
  6. Repeat, while k ≥ 0,
  ...
    c. Let testResult be ! ToBoolean(? Call(predicate, thisArg, « kValue, 𝔽(k), O »)).
  ...
includes: [testTypedArray.js]
features: [TypedArray, array-find-from-last]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([39, 2, 62]));
  var results = [];
  var result;

  sample.foo = "bar"; // Ignores non integer index properties

  sample.findLast(function() {
    results.push(arguments);
  });

  assert.sameValue(results.length, 3, "predicate is called for each index");

  result = results[0];
  assert.sameValue(result[0], 62, "results[0][0] === 62, value");
  assert.sameValue(result[1], 2, "results[0][1] === 2, index");
  assert.sameValue(result[2], sample, "results[0][2] === sample, instance");
  assert.sameValue(result.length, 3, "results[0].length === 3 arguments");

  result = results[1];
  assert.sameValue(result[0], 2, "results[1][0] === 2, value");
  assert.sameValue(result[1], 1, "results[1][1] === 1, index");
  assert.sameValue(result[2], sample, "results[1][2] === sample, instance");
  assert.sameValue(result.length, 3, "results[1].length === 3 arguments");

  result = results[2];
  assert.sameValue(result[0], 39, "results[2][0] === 39, value");
  assert.sameValue(result[1], 0, "results[2][1] === 0, index");
  assert.sameValue(result[2], sample, "results[2][2] === sample, instance");
  assert.sameValue(result.length, 3, "results[2].length === 3 arguments");
});
