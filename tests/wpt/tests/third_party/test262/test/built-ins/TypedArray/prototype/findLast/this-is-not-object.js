// Copyright (C) 2021 Microsoft. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.findlast
description: Throws a TypeError exception when `this` is not Object
info: |
  %TypedArray%.prototype.findLast (predicate [ , thisArg ] )

  2. Perform ? ValidateTypedArray(O).

  22.2.3.5.1 Runtime Semantics: ValidateTypedArray ( O )

  1. If Type(O) is not Object, throw a TypeError exception.
  ...
includes: [testTypedArray.js]
features: [Symbol, TypedArray, array-find-from-last]
---*/

var findLast = TypedArray.prototype.findLast;
var predicate = function() {};

assert.throws(TypeError, function() {
  findLast.call(undefined, predicate);
}, "this is undefined");

assert.throws(TypeError, function() {
  findLast.call(null, predicate);
}, "this is null");

assert.throws(TypeError, function() {
  findLast.call(42, predicate);
}, "this is 42");

assert.throws(TypeError, function() {
  findLast.call("1", predicate);
}, "this is a string");

assert.throws(TypeError, function() {
  findLast.call(true, predicate);
}, "this is true");

assert.throws(TypeError, function() {
  findLast.call(false, predicate);
}, "this is false");

var s = Symbol("s");
assert.throws(TypeError, function() {
  findLast.call(s, predicate);
}, "this is a Symbol");
