// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.foreach
description: >
  [[ArrayLength]] is accessed in place of performing a [[Get]] of "length"
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
  var loop = 0;

  Object.defineProperty(sample1, "length", {value: 1});

  sample1.forEach(function() {
    loop++;
  });

  assert.sameValue(loop, 42, "data descriptor");

  var sample2 = new TA(makeCtorArg(7));
  loop = 0;

  Object.defineProperty(sample2, "length", {
    get: function() {
      throw new Test262Error(
        "Does not return abrupt getting length property"
      );
    }
  });

  sample2.forEach(function() {
    loop++;
  });

  assert.sameValue(loop, 7, "accessor descriptor");
}, null, ["passthrough"]);

