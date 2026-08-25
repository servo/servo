// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.set-typedarray-offset
description: >
  Return abrupt from ToInteger(offset)
info: |
  22.2.3.23.2 %TypedArray%.prototype.set(typedArray [ , offset ] )

  1. Assert: typedArray has a [[TypedArrayName]] internal slot. If it does not,
  the definition in 22.2.3.23.1 applies.
  ...
  6. Let targetOffset be ? ToInteger(offset).
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

var obj1 = {
  valueOf: function() {
    throw new Test262Error();
  }
};

var obj2 = {
  toString: function() {
    throw new Test262Error();
  }
};

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(0));

  assert.throws(Test262Error, function() {
    sample.set(sample, obj1);
  }, "abrupt from valueOf");

  assert.throws(Test262Error, function() {
    sample.set(sample, obj2);
  }, "abrupt from toString");
}, null, null, ["immutable"]);
