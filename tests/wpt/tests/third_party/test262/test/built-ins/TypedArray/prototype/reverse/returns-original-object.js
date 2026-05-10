// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.reverse
description: Returns the same object
info: |
  22.2.3.22 %TypedArray%.prototype.reverse ( )

  %TypedArray%.prototype.reverse is a distinct function that implements the same
  algorithm as Array.prototype.reverse as defined in 22.1.3.21 except that the
  this object's [[ArrayLength]] internal slot is accessed in place of performing
  a [[Get]] of "length".

  22.1.3.21 Array.prototype.reverse ( )

  ...
  6. Return O.
includes: [testTypedArray.js]
features: [TypedArray]
---*/

var buffer = new ArrayBuffer(64);

testWithTypedArrayConstructors(function(TA) {
  var sample, result, expectedLength;

  sample = new TA(buffer, 0);
  expectedLength = sample.length;
  result = sample.reverse();
  assert.sameValue(result, sample, "returns the same object");
  assert.sameValue(sample.buffer, buffer, "keeps the same buffer");
  assert.sameValue(sample.length, expectedLength, "length is preserved");

  sample = new TA(buffer, 0, 0);
  result = sample.reverse();
  assert.sameValue(result, sample, "returns the same object (empty instance)");
  assert.sameValue(sample.buffer, buffer, "keeps the same buffer (empty instance)");
  assert.sameValue(sample.length, 0, "length is preserved (empty instance)");
}, null, ["passthrough"]);
