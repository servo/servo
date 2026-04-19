// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.findindex
description: >
  Predicate called as F.call( thisArg, kValue, k, O ) for each array entry.
info: |
  22.2.3.11 %TypedArray%.prototype.findIndex ( predicate [ , thisArg ] )

  %TypedArray%.prototype.findIndex is a distinct function that implements the
  same algorithm as Array.prototype.findIndex as defined in 22.1.3.9 except that
  the this object's [[ArrayLength]] internal slot is accessed in place of
  performing a [[Get]] of "length".

  ...

  22.1.3.9 Array.prototype.findIndex ( predicate[ , thisArg ] )

  ...
  4. If thisArg was supplied, let T be thisArg; else let T be undefined.
  5. Let k be 0.
  6. Repeat, while k < len
    ...
    c. Let testResult be ToBoolean(? Call(predicate, T, « kValue, k, O »)).
  ...
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([39n, 2n, 62n]));
  var results = [];
  var result;

  sample.foo = "bar"; // Ignores non integer index properties

  sample.findIndex(function() {
    results.push(arguments);
  });

  assert.sameValue(results.length, 3, "predicate is called for each index");

  result = results[0];
  assert.sameValue(result[0], 39n, "results[0][0] === 39, value");
  assert.sameValue(result[1], 0, "results[0][1] === 0, index");
  assert.sameValue(result[2], sample, "results[0][2] === sample, instance");
  assert.sameValue(result.length, 3, "results[0].length === 3, arguments");

  result = results[1];
  assert.sameValue(result[0], 2n, "results[1][0] === 2, value");
  assert.sameValue(result[1], 1, "results[1][1] === 1, index");
  assert.sameValue(result[2], sample, "results[1][2] === sample, instance");
  assert.sameValue(result.length, 3, "results[1].length === 3, arguments");

  result = results[2];
  assert.sameValue(result[0], 62n, "results[2][0] === 62, value");
  assert.sameValue(result[1], 2, "results[2][1] === 2, index");
  assert.sameValue(result[2], sample, "results[2][2] === sample, instance");
  assert.sameValue(result.length, 3, "results[2].length === 3, arguments");
});
