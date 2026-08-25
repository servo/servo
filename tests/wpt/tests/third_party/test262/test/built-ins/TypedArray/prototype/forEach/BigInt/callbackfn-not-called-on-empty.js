// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.foreach
description: >
  callbackfn is not called on empty instances
info: |
  22.2.3.12 %TypedArray%.prototype.forEach ( callbackfn [ , thisArg ] )

  %TypedArray%.prototype.forEach is a distinct function that implements the same
  algorithm as Array.prototype.forEach as defined in 22.1.3.10 except that the
  this object's [[ArrayLength]] internal slot is accessed in place of performing
  a [[Get]] of "length"

  22.1.3.10 Array.prototype.forEach ( callbackfn [ , thisArg ] )

  ...
  6. Repeat, while k < len
    ...
    c. If kPresent is true, then
      ...
      ii. Perform ? Call(callbackfn, T, « kValue, k, O »).
  ...
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var called = 0;

  new TA(makeCtorArg(0)).forEach(function() {
    called++;
  });

  assert.sameValue(called, 0);
});
