// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.filter
description: Throws a TypeError exception when `this` is not Object
info: |
  22.2.3.9 %TypedArray%.prototype.filter ( callbackfn [ , thisArg ] )

  The following steps are taken:

  1. Let O be the this value.
  2. Perform ? ValidateTypedArray(O).
  ...

  22.2.3.5.1 Runtime Semantics: ValidateTypedArray ( O )

  1. If Type(O) is not Object, throw a TypeError exception.
  ...
includes: [testTypedArray.js]
features: [Symbol, TypedArray]
---*/

var filter = TypedArray.prototype.filter;
var callbackfn = function() {};

assert.throws(TypeError, function() {
  filter.call(undefined, callbackfn);
}, "this is undefined");

assert.throws(TypeError, function() {
  filter.call(null, callbackfn);
}, "this is null");

assert.throws(TypeError, function() {
  filter.call(42, callbackfn);
}, "this is 42");

assert.throws(TypeError, function() {
  filter.call("1", callbackfn);
}, "this is a string");

assert.throws(TypeError, function() {
  filter.call(true, callbackfn);
}, "this is true");

assert.throws(TypeError, function() {
  filter.call(false, callbackfn);
}, "this is false");

var s = Symbol("s");
assert.throws(TypeError, function() {
  filter.call(s, callbackfn);
}, "this is a Symbol");
