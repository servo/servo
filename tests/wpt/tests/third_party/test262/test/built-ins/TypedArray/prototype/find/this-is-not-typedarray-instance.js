// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.find
description: >
  Throws a TypeError exception when `this` is not a TypedArray instance
info: |
  22.2.3.10 %TypedArray%.prototype.find (predicate [ , thisArg ] )

  This function is not generic. ValidateTypedArray is applied to the this value
  prior to evaluating the algorithm. If its result is an abrupt completion that
  exception is thrown instead of evaluating the algorithm.

  22.2.3.5.1 Runtime Semantics: ValidateTypedArray ( O )

  1. If Type(O) is not Object, throw a TypeError exception.
  2. If O does not have a [[TypedArrayName]] internal slot, throw a TypeError
  exception.
  ...
includes: [testTypedArray.js]
features: [TypedArray]
---*/

var find = TypedArray.prototype.find;
var predicate = function() {};

assert.throws(TypeError, function() {
  find.call({}, predicate);
}, "this is an Object");

assert.throws(TypeError, function() {
  find.call([], predicate);
}, "this is an Array");

var ab = new ArrayBuffer(8);
assert.throws(TypeError, function() {
  find.call(ab, predicate);
}, "this is an ArrayBuffer instance");

var dv = new DataView(new ArrayBuffer(8), 0, 1);
assert.throws(TypeError, function() {
  find.call(dv, predicate);
}, "this is a DataView instance");
