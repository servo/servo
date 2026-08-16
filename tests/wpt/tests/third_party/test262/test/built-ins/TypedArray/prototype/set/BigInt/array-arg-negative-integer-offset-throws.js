// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.set-array-offset
description: >
  Throw a RangeError exception if targetOffset < 0
info: |
  22.2.3.23.1 %TypedArray%.prototype.set (array [ , offset ] )

  1. Assert: array is any ECMAScript language value other than an Object with a
  [[TypedArrayName]] internal slot. If it is such an Object, the definition in
  22.2.3.23.2 applies.
  ...
  6. Let targetOffset be ? ToInteger(offset).
  7. If targetOffset < 0, throw a RangeError exception.
  ...
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(4));

  assert.throws(RangeError, function() {
    sample.set([1n], -1);
  }, "-1");

  assert.throws(RangeError, function() {
    sample.set([1n], -1.00001);
  }, "-1.00001");

  assert.throws(RangeError, function() {
    sample.set([1n], -Infinity);
  }, "-Infinity");
}, null, null, ["immutable"]);
