// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-%typedarray%.prototype.length
description: Throws a TypeError exception when `this` is not Object
info: |
  22.2.3.18 get %TypedArray%.prototype.length

  1. Let O be the this value.
  2. If Type(O) is not Object, throw a TypeError exception.
  ...
includes: [testTypedArray.js]
features: [Symbol, TypedArray]
---*/

var TypedArrayPrototype = TypedArray.prototype;
var getter = Object.getOwnPropertyDescriptor(
  TypedArrayPrototype, "length"
).get;


assert.throws(TypeError, function() {
  getter.call(undefined);
}, "undefined");

assert.throws(TypeError, function() {
  getter.call(null);
}, "null");

assert.throws(TypeError, function() {
  getter.call(42);
}, "number");

assert.throws(TypeError, function() {
  getter.call("1");
}, "string");

assert.throws(TypeError, function() {
  getter.call(true);
}, "true");

assert.throws(TypeError, function() {
  getter.call(false);
}, "false");

var s = Symbol("s");
assert.throws(TypeError, function() {
  getter.call(s);
}, "symbol");
