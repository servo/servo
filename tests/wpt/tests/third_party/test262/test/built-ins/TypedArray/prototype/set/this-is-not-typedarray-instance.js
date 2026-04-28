// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.set-overloaded-offset
description: >
  Throws a TypeError exception when `this` is not a TypedArray instance
info: |
  22.2.3.23 %TypedArray%.prototype.set

  ...
  2. Let target be the this value.
  3. If Type(target) is not Object, throw a TypeError exception.
  4. If target does not have a [[TypedArrayName]] internal slot, throw a
  TypeError exception.
  ...
includes: [testTypedArray.js]
features: [TypedArray]
---*/

var set = TypedArray.prototype.set;

assert.throws(TypeError, function() {
  set.call({}, []);
}, "this is an Object");

assert.throws(TypeError, function() {
  set.call([], []);
}, "this is an Array");

var ab1 = new ArrayBuffer(8);
assert.throws(TypeError, function() {
  set.call(ab1, []);
}, "this is an ArrayBuffer instance");

var dv1 = new DataView(new ArrayBuffer(8), 0, 1);
assert.throws(TypeError, function() {
  set.call(dv1, []);
}, "this is a DataView instance");

assert.throws(TypeError, function() {
  set.call({}, new Int8Array());
}, "this is an Object");

assert.throws(TypeError, function() {
  set.call([], new Int8Array());
}, "this is an Array");

var ab2 = new ArrayBuffer(8);
assert.throws(TypeError, function() {
  set.call(ab2, new Int8Array());
}, "this is an ArrayBuffer instance");

var dv2 = new DataView(new ArrayBuffer(8), 0, 1);
assert.throws(TypeError, function() {
  set.call(dv2, new Int8Array());
}, "this is a DataView instance");
