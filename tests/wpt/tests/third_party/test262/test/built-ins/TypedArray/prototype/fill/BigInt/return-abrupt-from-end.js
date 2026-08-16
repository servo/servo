// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.fill
description: >
  Return abrupt from ToInteger(end).
info: |
  22.2.3.8 %TypedArray%.prototype.fill (value [ , start [ , end ] ] )

  %TypedArray%.prototype.fill is a distinct function that implements the same
  algorithm as Array.prototype.fill as defined in 22.1.3.6 except that the this
  object's [[ArrayLength]] internal slot is accessed in place of performing a
  [[Get]] of "length". The implementation of the algorithm may be optimized with
  the knowledge that the this value is an object that has a fixed length and
  whose integer indexed properties are not sparse. However, such optimization
  must not introduce any observable changes in the specified behaviour of the
  algorithm.

  ...

  22.1.3.6 Array.prototype.fill (value [ , start [ , end ] ] )

  ...
  5. If end is undefined, let relativeEnd be len; else let relativeEnd be ?
  ToInteger(end).
  ...
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

var end = {
  valueOf: function() {
    throw new Test262Error();
  }
};

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(0));
  assert.throws(Test262Error, function() {
    sample.fill(1n, 0, end);
  });
}, null, null, ["immutable"]);
