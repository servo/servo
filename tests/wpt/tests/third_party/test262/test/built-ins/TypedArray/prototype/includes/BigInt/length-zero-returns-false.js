// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.includes
description: Returns false if length is 0
info: |
  22.2.3.13 %TypedArray%.prototype.includes ( searchElement [ , fromIndex ] )

  %TypedArray%.prototype.includes is a distinct function that implements the
  same algorithm as Array.prototype.includes as defined in 22.1.3.11 except that
  the this object's [[ArrayLength]] internal slot is accessed in place of
  performing a [[Get]] of "length".

  22.1.3.11 Array.prototype.includes ( searchElement [ , fromIndex ] )

  ...
  2. Let len be ? ToLength(? Get(O, "length")).
  3. If len is 0, return false.
  ...
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

var fromIndex = {
  valueOf: function() {
    throw new Test262Error();
  }
};

testWithBigIntTypedArrayConstructors(function(TA) {
  var sample = new TA();
  assert.sameValue(sample.includes(0), false, "returns false");
  assert.sameValue(sample.includes(), false, "returns false - no arg");
  assert.sameValue(
    sample.includes(0n, fromIndex), false,
    "length is checked before ToInteger(fromIndex)"
  );
}, null, ["passthrough"]);
