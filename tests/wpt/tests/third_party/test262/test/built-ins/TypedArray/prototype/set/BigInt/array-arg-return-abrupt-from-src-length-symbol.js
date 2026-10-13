// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.set-array-offset
description: >
  Return abrupt getting src "length" as symbol
info: |
  22.2.3.23.1 %TypedArray%.prototype.set (array [ , offset ] )

  1. Assert: array is any ECMAScript language value other than an Object with a
  [[TypedArrayName]] internal slot. If it is such an Object, the definition in
  22.2.3.23.2 applies.
  ...
  16. Let srcLength be ? ToLength(? Get(src, "length")).
  ...
includes: [testTypedArray.js]
features: [BigInt, Symbol, TypedArray]
---*/

var obj = {
  length: Symbol("1")
};

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([1n, 2n, 3n]));

  assert.throws(TypeError, function() {
    sample.set(obj);
  });
}, null, null, ["immutable"]);
