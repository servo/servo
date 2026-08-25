// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.sort
description: Throws a TypeError exception when `this` is not Object
info: |
  22.2.3.26 %TypedArray%.prototype.sort ( comparefn )

  1. Let obj be the this value as the argument.
  2. Let buffer be ? ValidateTypedArray(obj).
  ...

  22.2.3.5.1 Runtime Semantics: ValidateTypedArray ( O )

  1. If Type(O) is not Object, throw a TypeError exception.
  ...
includes: [testTypedArray.js]
features: [Symbol, TypedArray]
---*/

var sort = TypedArray.prototype.sort;
var comparefn = function() {};

assert.throws(TypeError, function() {
  sort.call(undefined, comparefn);
}, "this is undefined");

assert.throws(TypeError, function() {
  sort.call(null, comparefn);
}, "this is null");

assert.throws(TypeError, function() {
  sort.call(42, comparefn);
}, "this is 42");

assert.throws(TypeError, function() {
  sort.call("1", comparefn);
}, "this is a string");

assert.throws(TypeError, function() {
  sort.call(true, comparefn);
}, "this is true");

assert.throws(TypeError, function() {
  sort.call(false, comparefn);
}, "this is false");

var s = Symbol("s");
assert.throws(TypeError, function() {
  sort.call(s, comparefn);
}, "this is a Symbol");
