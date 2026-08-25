// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.indexof
description: Get "length" uses internal ArrayLength
info: |
  22.2.3.13 %TypedArray%.prototype.indexOf (searchElement [ , fromIndex ] )

  %TypedArray%.prototype.indexOf is a distinct function that implements the same
  algorithm as Array.prototype.indexOf as defined in 22.1.3.12 except that the
  this object's [[ArrayLength]] internal slot is accessed in place of performing
  a [[Get]] of "length".

  22.1.3.12 Array.prototype.indexOf ( searchElement [ , fromIndex ] )

  ...
  2. Let len be ? ToLength(? Get(O, "length")).
  ...
includes: [testTypedArray.js]
features: [TypedArray]
---*/

Object.defineProperty(TypedArray.prototype, "length", {value: 0});

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([7]));

  Object.defineProperty(TA.prototype, "length", {value: 0});
  Object.defineProperty(sample, "length", {value: 0});

  assert.sameValue(sample.indexOf(7), 0);
}, null, ["passthrough"]);
