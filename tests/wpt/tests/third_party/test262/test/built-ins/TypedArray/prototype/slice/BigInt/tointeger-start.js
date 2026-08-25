// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.slice
description: ToInteger(begin)
info: |
  22.2.3.24 %TypedArray%.prototype.slice ( start, end )

  ...
  4. Let relativeStart be ? ToInteger(start).
  ...
includes: [testTypedArray.js, compareArray.js]
features: [BigInt, TypedArray]
---*/

var obj = {
  valueOf: function() {
    return 2;
  }
};

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([40n, 41n, 42n, 43n]));

  assert(compareArray(sample.slice(false), [40n, 41n, 42n, 43n]), "false");
  assert(compareArray(sample.slice(true), [41n, 42n, 43n]), "true");

  assert(compareArray(sample.slice(NaN), [40n, 41n, 42n, 43n]), "NaN");
  assert(compareArray(sample.slice(null), [40n, 41n, 42n, 43n]), "null");
  assert(compareArray(sample.slice(undefined), [40n, 41n, 42n, 43n]), "undefined");

  assert(compareArray(sample.slice(1.1), [41n, 42n, 43n]), "1.1");
  assert(compareArray(sample.slice(1.5), [41n, 42n, 43n]), "1.5");
  assert(compareArray(sample.slice(0.6), [40n, 41n, 42n, 43n]), "0.6");

  assert(compareArray(sample.slice(-1.5), [43n]), "-1.5");
  assert(compareArray(sample.slice(-1.1), [43n]), "-1.1");
  assert(compareArray(sample.slice(-0.6), [40n, 41n, 42n, 43n]), "-0.6");

  assert(compareArray(sample.slice("3"), [43n]), "string");
  assert(
    compareArray(
      sample.slice(obj),
      [42n, 43n]
    ),
    "object"
  );
});
