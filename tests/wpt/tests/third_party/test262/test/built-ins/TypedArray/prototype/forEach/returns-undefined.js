// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.foreach
description: >
  Returns undefined
info: |
  22.2.3.12 %TypedArray%.prototype.forEach ( callbackfn [ , thisArg ] )

  %TypedArray%.prototype.forEach is a distinct function that implements the same
  algorithm as Array.prototype.forEach as defined in 22.1.3.10 except that the
  this object's [[ArrayLength]] internal slot is accessed in place of performing
  a [[Get]] of "length"
includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample1 = new TA(makeCtorArg(42));

  var result1 = sample1.forEach(function() {
    return 42;
  });

  assert.sameValue(result1, undefined, "result1");

  var sample2 = new TA(makeCtorArg(1));
  var result2 = sample2.forEach(function() {
    return null;
  });

  assert.sameValue(result2, undefined, "result2");
});
