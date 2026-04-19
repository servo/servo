// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.every
description: >
  Returns true if every callbackfn returns a coerced true.
info: |
  22.2.3.7 %TypedArray%.prototype.every ( callbackfn [ , thisArg ] )

  %TypedArray%.prototype.every is a distinct function that implements the same
  algorithm as Array.prototype.every as defined in 22.1.3.5 except that the this
  object's [[ArrayLength]] internal slot is accessed in place of performing a
  [[Get]] of "length".

  22.1.3.5 Array.prototype.every ( callbackfn [ , thisArg ] )

  ...
  7. Return true.
includes: [testTypedArray.js]
features: [Symbol, TypedArray]
---*/

testWithTypedArrayConstructors(function(TA) {
  var called = 0;
  var values = [
    true,
    1,
    "test262",
    Symbol("1"),
    {},
    [],
    -1,
    Infinity,
    -Infinity,
    0.1,
    -0.1
  ];
  var sample = new TA(values.length);
  var result = sample.every(function() {
    called++;
    return values.unshift();
  });

  assert.sameValue(called, sample.length, "callbackfn called for each index");
  assert.sameValue(result, true, "return is true");
}, null, ["passthrough"]);
