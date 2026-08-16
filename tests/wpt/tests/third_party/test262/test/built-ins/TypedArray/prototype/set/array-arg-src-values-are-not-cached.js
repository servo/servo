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
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(5));
  var obj = {
    length: 5,
    '1': 7,
    '2': 7,
    '3': 7,
    '4': 7
  };
  Object.defineProperty(obj, 0, {
    get: function() {
      obj[1] = 43;
      obj[2] = 44;
      obj[3] = 45;
      obj[4] = 46;
      return 42;
    }
  });

  sample.set(obj);

  assert(compareArray(sample, [42, 43, 44, 45, 46]));
}, null, null, ["immutable"]);
