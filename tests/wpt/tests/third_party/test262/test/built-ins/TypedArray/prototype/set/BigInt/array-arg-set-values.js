// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.set-array-offset
description: >
  Set values to target and return undefined
info: |
  22.2.3.23.1 %TypedArray%.prototype.set (array [ , offset ] )

  1. Assert: array is any ECMAScript language value other than an Object with a
  [[TypedArrayName]] internal slot. If it is such an Object, the definition in
  22.2.3.23.2 applies.
  ...
  21. Repeat, while targetByteIndex < limit
    Let Pk be ! ToString(k).
    Let kNumber be ? ToNumber(? Get(src, Pk)).
    If IsDetachedBuffer(targetBuffer) is true, throw a TypeError exception.
    Perform SetValueInBuffer(targetBuffer, targetByteIndex, targetType, kNumber).
    ...
  22. Return undefined.
includes: [testTypedArray.js, compareArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var src = [42n, 43n];
  var srcObj = {
    length: 2,
    '0': 7n,
    '1': 17n
  };
  var sample, result;

  sample = new TA(makeCtorArg([1n, 2n, 3n, 4n]));
  result = sample.set(src, 0);
  assert(compareArray(sample, [42n, 43n, 3n, 4n]), "offset: 0, result: " + sample);
  assert.sameValue(result, undefined, "returns undefined");

  sample = new TA(makeCtorArg([1n, 2n, 3n, 4n]));
  result = sample.set(src, 1);
  assert(compareArray(sample, [1n, 42n, 43n, 4n]), "offset: 1, result: " + sample);
  assert.sameValue(result, undefined, "returns undefined");

  sample = new TA(makeCtorArg([1n, 2n, 3n, 4n]));
  result = sample.set(src, 2);
  assert(compareArray(sample, [1n, 2n, 42n, 43n]), "offset: 2, result: " + sample);
  assert.sameValue(result, undefined, "returns undefined");

  sample = new TA(makeCtorArg([1n, 2n, 3n, 4n]));
  result = sample.set(srcObj, 0);
  assert(compareArray(sample, [7n, 17n, 3n, 4n]), "offset: 0, result: " + sample);
  assert.sameValue(result, undefined, "returns undefined");

  sample = new TA(makeCtorArg([1n, 2n, 3n, 4n]));
  result = sample.set(srcObj, 1);
  assert(compareArray(sample, [1n, 7n, 17n, 4n]), "offset: 1, result: " + sample);
  assert.sameValue(result, undefined, "returns undefined");

  sample = new TA(makeCtorArg([1n, 2n, 3n, 4n]));
  result = sample.set(srcObj, 2);
  assert(compareArray(sample, [1n, 2n, 7n, 17n]), "offset: 2, result: " + sample);
  assert.sameValue(result, undefined, "returns undefined");
}, null, null, ["immutable"]);
