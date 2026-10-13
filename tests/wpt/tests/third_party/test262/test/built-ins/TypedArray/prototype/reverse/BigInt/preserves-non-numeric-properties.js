// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.reverse
description: Preserves non numeric properties
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
features: [BigInt, Symbol, TypedArray]
---*/

var s = Symbol("1");

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample, result;

  sample = new TA(makeCtorArg(2));
  sample.foo = 42;
  sample.bar = "bar";
  sample[s] = 1;
  result = sample.reverse();
  assert.sameValue(result.foo, 42, "sample.foo === 42");
  assert.sameValue(result.bar, "bar", "sample.bar === 'bar'");
  assert.sameValue(result[s], 1, "sample[s] === 1");
}, null, null, ["immutable"]);
