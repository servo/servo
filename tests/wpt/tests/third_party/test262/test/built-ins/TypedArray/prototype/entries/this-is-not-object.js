// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.entries
description: Throws a TypeError exception when `this` is not Object
info: |
  22.2.3.6 %TypedArray%.prototype.entries ( )

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

var entries = TypedArray.prototype.entries;

assert.throws(TypeError, function() {
  entries.call(undefined);
}, "this is undefined");

assert.throws(TypeError, function() {
  entries.call(null);
}, "this is null");

assert.throws(TypeError, function() {
  entries.call(42);
}, "this is 42");

assert.throws(TypeError, function() {
  entries.call("1");
}, "this is a string");

assert.throws(TypeError, function() {
  entries.call(true);
}, "this is true");

assert.throws(TypeError, function() {
  entries.call(false);
}, "this is false");

var s = Symbol("s");
assert.throws(TypeError, function() {
  entries.call(s);
}, "this is a Symbol");
