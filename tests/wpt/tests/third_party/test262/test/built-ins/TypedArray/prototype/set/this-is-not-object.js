// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.set-overloaded-offset
description: Throws a TypeError exception when `this` is not Object
info: |
  22.2.3.23 %TypedArray%.prototype.set

  ...
  2. Let target be the this value.
  3. If Type(target) is not Object, throw a TypeError exception.
  ...
includes: [testTypedArray.js]
features: [Symbol, TypedArray]
---*/

var set = TypedArray.prototype.set;

assert.throws(TypeError, function() {
  set.call(undefined, []);
}, "this is undefined");

assert.throws(TypeError, function() {
  set.call(null, []);
}, "this is null");

assert.throws(TypeError, function() {
  set.call(undefined, new Int8Array());
}, "this is undefined");

assert.throws(TypeError, function() {
  set.call(null, new Int8Array());
}, "this is null");

assert.throws(TypeError, function() {
  set.call(42, []);
}, "this is 42");

assert.throws(TypeError, function() {
  set.call("1", []);
}, "this is a string");

assert.throws(TypeError, function() {
  set.call(true, []);
}, "this is true");

assert.throws(TypeError, function() {
  set.call(false, []);
}, "this is false");

var s1 = Symbol("s");
assert.throws(TypeError, function() {
  set.call(s1, []);
}, "this is a Symbol");

assert.throws(TypeError, function() {
  set.call(42, new Int8Array(1));
}, "this is 42");

assert.throws(TypeError, function() {
  set.call("1", new Int8Array(1));
}, "this is a string");

assert.throws(TypeError, function() {
  set.call(true, new Int8Array(1));
}, "this is true");

assert.throws(TypeError, function() {
  set.call(false, new Int8Array(1));
}, "this is false");

var s2 = Symbol("s");
assert.throws(TypeError, function() {
  set.call(s2, new Int8Array(1));
}, "this is a Symbol");
