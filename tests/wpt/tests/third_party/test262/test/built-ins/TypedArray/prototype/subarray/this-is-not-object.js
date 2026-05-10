// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.subarray
description: Throws a TypeError exception when `this` is not Object
info: |
  22.2.3.27 %TypedArray%.prototype.subarray( begin , end )

  The following steps are taken:

  1. Let O be the this value.
  2. If Type(O) is not Object, throw a TypeError exception.
  ...
includes: [testTypedArray.js]
features: [Symbol, TypedArray]
---*/

var subarray = TypedArray.prototype.subarray;

assert.throws(TypeError, function() {
  subarray.call(undefined, 0, 0);
}, "this is undefined");

assert.throws(TypeError, function() {
  subarray.call(null, 0, 0);
}, "this is null");

assert.throws(TypeError, function() {
  subarray.call(42, 0, 0);
}, "this is 42");

assert.throws(TypeError, function() {
  subarray.call("1", 0, 0);
}, "this is a string");

assert.throws(TypeError, function() {
  subarray.call(true, 0, 0);
}, "this is true");

assert.throws(TypeError, function() {
  subarray.call(false, 0, 0);
}, "this is false");

var s = Symbol("s");
assert.throws(TypeError, function() {
  subarray.call(s, 0, 0);
}, "this is a Symbol");
