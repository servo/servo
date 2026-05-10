// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.set-array-offset
description: >
  Get and set each value in order
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
  var calls = [];
  var obj = {
    length: 3
  };
  Object.defineProperty(obj, 0, {
    get: function() {
      calls.push(0);
      calls.push(sample.join());
      return 42n;
    }
  });

  Object.defineProperty(obj, 1, {
    get: function() {
      calls.push(1);
      calls.push(sample.join());
      return 43n;
    }
  });

  Object.defineProperty(obj, 2, {
    get: function() {
      calls.push(2);
      calls.push(sample.join());
      return 44n;
    }
  });

  Object.defineProperty(obj, 3, {
    get: function() {
      throw new Test262Error("Should not call obj[3]");
    }
  });

  sample.set(obj, 1);

  assert(
    compareArray(sample, [0n, 42n, 43n, 44n, 0n]),
    "values are set for src length"
  );

  assert(
    compareArray(calls, [0, "0,0,0,0,0", 1, "0,42,0,0,0", 2, "0,42,43,0,0"]),
    "values are set in order"
  );
}, null, ["passthrough"]);
