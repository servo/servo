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
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var src = [42, 43];
  var srcObj = {
    length: 2,
    '0': 7,
    '1': 17
  };
  var sample, result;

  sample = new TA(makeCtorArg([1, 2, 3, 4]));
  result = sample.set(src, 0);
  assert(compareArray(sample, [42, 43, 3, 4]), "offset: 0, result: " + sample);
  assert.sameValue(result, undefined, "returns undefined");

  sample = new TA(makeCtorArg([1, 2, 3, 4]));
  result = sample.set(src, 1);
  assert(compareArray(sample, [1, 42, 43, 4]), "offset: 1, result: " + sample);
  assert.sameValue(result, undefined, "returns undefined");

  sample = new TA(makeCtorArg([1, 2, 3, 4]));
  result = sample.set(src, 2);
  assert(compareArray(sample, [1, 2, 42, 43]), "offset: 2, result: " + sample);
  assert.sameValue(result, undefined, "returns undefined");

  sample = new TA(makeCtorArg([1, 2, 3, 4]));
  result = sample.set(srcObj, 0);
  assert(compareArray(sample, [7, 17, 3, 4]), "offset: 0, result: " + sample);
  assert.sameValue(result, undefined, "returns undefined");

  sample = new TA(makeCtorArg([1, 2, 3, 4]));
  result = sample.set(srcObj, 1);
  assert(compareArray(sample, [1, 7, 17, 4]), "offset: 1, result: " + sample);
  assert.sameValue(result, undefined, "returns undefined");

  sample = new TA(makeCtorArg([1, 2, 3, 4]));
  result = sample.set(srcObj, 2);
  assert(compareArray(sample, [1, 2, 7, 17]), "offset: 2, result: " + sample);
  assert.sameValue(result, undefined, "returns undefined");
}, null, null, ["immutable"]);
