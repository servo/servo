// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.set-array-offset
description: >
  Values from src array are not cached
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
  var sample = new TA(makeCtorArg(5));
  var obj = {
    length: 5,
    '1': 7n,
    '2': 7n,
    '3': 7n,
    '4': 7n
  };
  Object.defineProperty(obj, 0, {
    get: function() {
      obj[1] = 43n;
      obj[2] = 44n;
      obj[3] = 45n;
      obj[4] = 46n;
      return 42n;
    }
  });

  sample.set(obj);

  assert(compareArray(sample, [42n, 43n, 44n, 45n, 46n]));
}, null, null, ["immutable"]);
