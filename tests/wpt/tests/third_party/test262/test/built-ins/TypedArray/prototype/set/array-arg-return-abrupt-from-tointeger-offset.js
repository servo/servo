// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.set-array-offset
description: >
  Return abrupt from ToInteger(offset)
info: |
  22.2.3.23.1 %TypedArray%.prototype.set (array [ , offset ] )

  1. Assert: array is any ECMAScript language value other than an Object with a
  [[TypedArrayName]] internal slot. If it is such an Object, the definition in
  22.2.3.23.2 applies.
  ...
  6. Let targetOffset be ? ToInteger(offset).
includes: [testTypedArray.js]
features: [TypedArray]
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

testWithTypedArrayConstructors(function(TA) {
  var sample = new TA();

  assert.throws(Test262Error, function() {
    sample.set([], obj1);
  }, "abrupt from valueOf");

  assert.throws(Test262Error, function() {
    sample.set([], obj2);
  }, "abrupt from toString");
}, null, ["passthrough"]);
