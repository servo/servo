// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.sort
description: >
  Throws a TypeError exception when `this` is not a TypedArray instance
info: |
  22.2.3.26 %TypedArray%.prototype.sort ( comparefn )

  1. Let obj be the this value as the argument.
  2. Let buffer be ? ValidateTypedArray(obj).
  ...

  22.2.3.5.1 Runtime Semantics: ValidateTypedArray ( O )

  1. If Type(O) is not Object, throw a TypeError exception.
  2. If O does not have a [[TypedArrayName]] internal slot, throw a TypeError
  exception.
  ...
includes: [testTypedArray.js]
features: [TypedArray]
---*/

var sort = TypedArray.prototype.sort;
var comparefn = function() {};

assert.throws(TypeError, function() {
  sort.call({}, comparefn);
}, "this is an Object");

assert.throws(TypeError, function() {
  sort.call([], comparefn);
}, "this is an Array");

var ab = new ArrayBuffer(8);
assert.throws(TypeError, function() {
  sort.call(ab, comparefn);
}, "this is an ArrayBuffer instance");

var dv = new DataView(new ArrayBuffer(8), 0, 1);
assert.throws(TypeError, function() {
  sort.call(dv, comparefn);
}, "this is a DataView instance");
