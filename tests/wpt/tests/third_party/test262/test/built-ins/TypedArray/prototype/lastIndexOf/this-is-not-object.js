// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.lastindexof
description: Throws a TypeError exception when `this` is not Object
info: |
  22.2.3.17 %TypedArray%.prototype.lastIndexOf ( searchElement [ , fromIndex ] )

  This function is not generic. ValidateTypedArray is applied to the this value
  prior to evaluating the algorithm. If its result is an abrupt completion that
  exception is thrown instead of evaluating the algorithm.

  22.2.3.5.1 Runtime Semantics: ValidateTypedArray ( O )

  1. If Type(O) is not Object, throw a TypeError exception.
  ...
includes: [testTypedArray.js]
features: [Symbol, TypedArray]
---*/

var lastIndexOf = TypedArray.prototype.lastIndexOf;

assert.throws(TypeError, function() {
  lastIndexOf.call(undefined, 42);
}, "this is undefined");

assert.throws(TypeError, function() {
  lastIndexOf.call(null, 42);
}, "this is null");

assert.throws(TypeError, function() {
  lastIndexOf.call(42, 42);
}, "this is 42");

assert.throws(TypeError, function() {
  lastIndexOf.call("1", 42);
}, "this is a string");

assert.throws(TypeError, function() {
  lastIndexOf.call(true, 42);
}, "this is true");

assert.throws(TypeError, function() {
  lastIndexOf.call(false, 42);
}, "this is false");

var s = Symbol("s");
assert.throws(TypeError, function() {
  lastIndexOf.call(s, 42);
}, "this is a Symbol");
