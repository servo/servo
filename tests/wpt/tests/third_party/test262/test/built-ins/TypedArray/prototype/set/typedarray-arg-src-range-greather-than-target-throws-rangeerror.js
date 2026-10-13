// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.set-typedarray-offset
description: >
  If srcLength + targetOffset > targetLength, throw a RangeError exception.
info: |
  22.2.3.23.2 %TypedArray%.prototype.set(typedArray [ , offset ] )

  1. Assert: typedArray has a [[TypedArrayName]] internal slot. If it does not,
  the definition in 22.2.3.23.1 applies.
  ...
  6. Let targetOffset be ? ToInteger(offset).
  ...
  10. Let targetLength be the value of target's [[ArrayLength]] internal slot.
  ...
  20. Let srcLength be the value of typedArray's [[ArrayLength]] internal slot.
  ...
  22. If srcLength + targetOffset > targetLength, throw a RangeError exception.
  ...
includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample, src;

  sample = new TA(makeCtorArg(2));
  src = new TA(makeCtorArg(2));
  assert.throws(RangeError, function() {
    sample.set(src, 1);
  }, "2 + 1 > 2");

  sample = new TA(makeCtorArg(1));
  src = new TA(makeCtorArg(2));
  assert.throws(RangeError, function() {
    sample.set(src, 0);
  }, "2 + 0 > 1");

  sample = new TA(makeCtorArg(1));
  src = new TA(makeCtorArg(0));
  assert.throws(RangeError, function() {
    sample.set(src, 2);
  }, "0 + 2 > 1");

  sample = new TA(makeCtorArg(2));
  src = new TA(makeCtorArg(2));
  assert.throws(RangeError, function() {
    sample.set(src, Infinity);
  }, "2 + Infinity > 2");
}, null, null, ["immutable"]);
