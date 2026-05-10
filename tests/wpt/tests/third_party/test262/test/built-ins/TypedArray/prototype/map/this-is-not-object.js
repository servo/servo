// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.map
description: Throws a TypeError exception when `this` is not Object
info: |
  22.2.3.19 %TypedArray%.prototype.map ( callbackfn [ , thisArg ] )

  1. Let O be the this value.
  2. Perform ? ValidateTypedArray(O).
  ...

  22.2.3.5.1 Runtime Semantics: ValidateTypedArray ( O )

  1. If Type(O) is not Object, throw a TypeError exception.
  ...
includes: [testTypedArray.js]
features: [Symbol, TypedArray]
---*/

var map = TypedArray.prototype.map;
var callbackfn = function() {};

assert.throws(TypeError, function() {
  map.call(undefined, callbackfn);
}, "this is undefined");

assert.throws(TypeError, function() {
  map.call(null, callbackfn);
}, "this is null");

assert.throws(TypeError, function() {
  map.call(42, callbackfn);
}, "this is 42");

assert.throws(TypeError, function() {
  map.call("1", callbackfn);
}, "this is a string");

assert.throws(TypeError, function() {
  map.call(true, callbackfn);
}, "this is true");

assert.throws(TypeError, function() {
  map.call(false, callbackfn);
}, "this is false");

var s = Symbol("s");
assert.throws(TypeError, function() {
  map.call(s, callbackfn);
}, "this is a Symbol");
