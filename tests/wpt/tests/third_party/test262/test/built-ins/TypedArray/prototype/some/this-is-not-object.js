// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.some
description: Throws a TypeError exception when `this` is not Object
info: |
  22.2.3.25 %TypedArray%.prototype.some ( callbackfn [ , thisArg ] )

  This function is not generic. ValidateTypedArray is applied to the this value
  prior to evaluating the algorithm. If its result is an abrupt completion that
  exception is thrown instead of evaluating the algorithm.

  22.2.3.5.1 Runtime Semantics: ValidateTypedArray ( O )

  1. If Type(O) is not Object, throw a TypeError exception.
  ...
includes: [testTypedArray.js]
features: [Symbol, TypedArray]
---*/

var some = TypedArray.prototype.some;
var callbackfn = function() {};

assert.throws(TypeError, function() {
  some.call(undefined, callbackfn);
}, "this is undefined");

assert.throws(TypeError, function() {
  some.call(null, callbackfn);
}, "this is null");

assert.throws(TypeError, function() {
  some.call(42, callbackfn);
}, "this is 42");

assert.throws(TypeError, function() {
  some.call("1", callbackfn);
}, "this is a string");

assert.throws(TypeError, function() {
  some.call(true, callbackfn);
}, "this is true");

assert.throws(TypeError, function() {
  some.call(false, callbackfn);
}, "this is false");

var s = Symbol("s");
assert.throws(TypeError, function() {
  some.call(s, callbackfn);
}, "this is a Symbol");
