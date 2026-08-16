// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.set-array-offset
description: >
  Type conversions on ToNumber(src property value)
info: |
  22.2.3.23.1 %TypedArray%.prototype.set (array [ , offset ] )

  1. Assert: array is any ECMAScript language value other than an Object with a
  [[TypedArrayName]] internal slot. If it is such an Object, the definition in
  22.2.3.23.2 applies.
  ...
  21. Repeat, while targetByteIndex < limit
    a. Let Pk be ! ToString(k).
    b. Let kNumber be ? ToNumber(? Get(src, Pk)).
    c. If IsDetachedBuffer(targetBuffer) is true, throw a TypeError exception.
    d. Perform SetValueInBuffer(targetBuffer, targetByteIndex, targetType,
    kNumber).
  ...
includes: [testTypedArray.js, compareArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var obj1 = {
      valueOf: function() {
        return 42n;
      }
  };

  var obj2 = {
      toString: function() {
        return "42";
      }
  };

  var arr = [false, true, obj1, [], [1]];

  var sample = new TA(arr.length);
  var expected = new TA(makeCtorArg([0n, 1n, 42n, 0n, 1n]));

  sample.set(arr);

  assert(
    compareArray(sample, expected),
    "sample: [" + sample + "], expected: [" + expected + "]"
  );
}, null, null, ["immutable"]);
