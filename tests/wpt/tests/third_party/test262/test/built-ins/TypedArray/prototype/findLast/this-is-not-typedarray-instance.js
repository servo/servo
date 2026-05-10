// Copyright (C) 2021 Microsoft. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.findlast
description: >
  Throws a TypeError exception when `this` is not a TypedArray instance
info: |
  %TypedArray%.prototype.findLast (predicate [ , thisArg ] )

  2. Perform ? ValidateTypedArray(O).

  22.2.3.5.1 Runtime Semantics: ValidateTypedArray ( O )

  1. If Type(O) is not Object, throw a TypeError exception.
  2. If O does not have a [[TypedArrayName]] internal slot, throw a TypeError
  exception.
  ...
includes: [testTypedArray.js]
features: [TypedArray, array-find-from-last]
---*/

var findlast = TypedArray.prototype.findlast;
var predicate = function() {};

assert.throws(TypeError, function() {
  findlast.call({}, predicate);
}, "this is an Object");

assert.throws(TypeError, function() {
  findlast.call([], predicate);
}, "this is an Array");

var ab = new ArrayBuffer(8);
assert.throws(TypeError, function() {
  findlast.call(ab, predicate);
}, "this is an ArrayBuffer instance");

var dv = new DataView(new ArrayBuffer(8), 0, 1);
assert.throws(TypeError, function() {
  findlast.call(dv, predicate);
}, "this is a DataView instance");
