// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.indexof
description: Returns -1 if length is 0
info: |
  22.2.3.13 %TypedArray%.prototype.indexOf (searchElement [ , fromIndex ] )

  %TypedArray%.prototype.indexOf is a distinct function that implements the same
  algorithm as Array.prototype.indexOf as defined in 22.1.3.12 except that the
  this object's [[ArrayLength]] internal slot is accessed in place of performing
  a [[Get]] of "length".

  22.1.3.12 Array.prototype.indexOf ( searchElement [ , fromIndex ] )

  ...
  2. Let len be ? ToLength(? Get(O, "length")).
  3. If len is 0, return -1.
  ...
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

var fromIndex = {
  valueOf: function() {
    throw new Test262Error();
  }
};

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(0));
  assert.sameValue(sample.indexOf(0n), -1, "returns -1");
  assert.sameValue(
    sample.indexOf(0n, fromIndex), -1,
    "length is checked before ToInteger(fromIndex)"
  );
});
