// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.reduce
description: >
  Returns [0] without calling callbackfn if length is 1 and initialValue is not
  present.
info: |
  22.2.3.20 %TypedArray%.prototype.reduce ( callbackfn [ , initialValue ] )

  %TypedArray%.prototype.reduce is a distinct function that implements the same
  algorithm as Array.prototype.reduce as defined in 22.1.3.19 except that the
  this object's [[ArrayLength]] internal slot is accessed in place of performing
  a [[Get]] of "length".

  22.1.3.19 Array.prototype.reduce ( callbackfn [ , initialValue ] )

  ...
  7. Else initialValue is not present,
    ...
    b. Repeat, while kPresent is false and k < len
      ...
      iii. If kPresent is true, then
        1. Let accumulator be ? Get(O, Pk).
      iv. Increase k by 1.
    ...
  8. Repeat, while k < len
    ...
  9. Return accumulator.
includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var called = false;
  var result = new TA(makeCtorArg([42])).reduce(function() {
    called = true;
  });

  assert.sameValue(result, 42);
  assert.sameValue(called, false);
});
