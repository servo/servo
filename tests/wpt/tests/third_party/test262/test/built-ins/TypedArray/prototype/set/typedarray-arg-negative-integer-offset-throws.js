// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.set-typedarray-offset
description: >
  Throw a RangeError exception if targetOffset < 0
info: |
  22.2.3.23.2 %TypedArray%.prototype.set(typedArray [ , offset ] )

  1. Assert: typedArray has a [[TypedArrayName]] internal slot. If it does not,
  the definition in 22.2.3.23.1 applies.
  ...
  6. Let targetOffset be ? ToInteger(offset).
  7. If targetOffset < 0, throw a RangeError exception.
includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(0));

  assert.throws(RangeError, function() {
    sample.set(sample, -1);
  }, "-1");

  assert.throws(RangeError, function() {
    sample.set(sample, -1.00001);
  }, "-1.00001");

  assert.throws(RangeError, function() {
    sample.set(sample, -Infinity);
  }, "-Infinity");
}, null, null, ["immutable"]);
