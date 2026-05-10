// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.find
description: Throws a TypeError exception when `this` is not Object
info: |
  22.2.3.10 %TypedArray%.prototype.find (predicate [ , thisArg ] )

  This function is not generic. ValidateTypedArray is applied to the this value
  prior to evaluating the algorithm. If its result is an abrupt completion that
  exception is thrown instead of evaluating the algorithm.

  22.2.3.5.1 Runtime Semantics: ValidateTypedArray ( O )

  1. If Type(O) is not Object, throw a TypeError exception.
  ...
includes: [testTypedArray.js]
features: [Symbol, TypedArray]
---*/

var find = TypedArray.prototype.find;
var predicate = function() {};

assert.throws(TypeError, function() {
  find.call(undefined, predicate);
}, "this is undefined");

assert.throws(TypeError, function() {
  find.call(null, predicate);
}, "this is null");

assert.throws(TypeError, function() {
  find.call(42, predicate);
}, "this is 42");

assert.throws(TypeError, function() {
  find.call("1", predicate);
}, "this is a string");

assert.throws(TypeError, function() {
  find.call(true, predicate);
}, "this is true");

assert.throws(TypeError, function() {
  find.call(false, predicate);
}, "this is false");

var s = Symbol("s");
assert.throws(TypeError, function() {
  find.call(s, predicate);
}, "this is a Symbol");
