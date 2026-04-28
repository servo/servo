// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.reduce
description: >
  callbackfn is not called on empty instances
info: |
  22.2.3.20 %TypedArray%.prototype.reduce ( callbackfn [ , initialValue ] )

  %TypedArray%.prototype.reduce is a distinct function that implements the same
  algorithm as Array.prototype.reduce as defined in 22.1.3.19 except that the
  this object's [[ArrayLength]] internal slot is accessed in place of performing
  a [[Get]] of "length".

  22.1.3.19 Array.prototype.reduce ( callbackfn [ , initialValue ] )

  ...
  4. If len is 0 and initialValue is not present, throw a TypeError exception.
  ...
  8. Repeat, while k < len
    ...
    c. If kPresent is true, then
      ...
      i. Let accumulator be ? Call(callbackfn, undefined, « accumulator, kValue,
      k, O »).
  ...
includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA) {
  var called = 0;

  new TA().reduce(function() {
    called++;
  }, undefined);

  assert.sameValue(called, 0);
}, null, ["passthrough"]);
