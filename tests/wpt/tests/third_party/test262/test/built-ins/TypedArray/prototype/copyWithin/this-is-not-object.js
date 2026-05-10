// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.copywithin
description: Throws a TypeError exception when `this` is not Object
info: |
  22.2.3.5 %TypedArray%.prototype.copyWithin (target, start [ , end ] )

  This function is not generic. ValidateTypedArray is applied to the this value
  prior to evaluating the algorithm. If its result is an abrupt completion that
  exception is thrown instead of evaluating the algorithm.

  22.2.3.5.1 Runtime Semantics: ValidateTypedArray ( O )

  1. If Type(O) is not Object, throw a TypeError exception.
  ...
includes: [testTypedArray.js]
features: [Symbol, TypedArray]
---*/

var copyWithin = TypedArray.prototype.copyWithin;

assert.throws(TypeError, function() {
  copyWithin.call(undefined, 0, 0);
}, "this is undefined");

assert.throws(TypeError, function() {
  copyWithin.call(null, 0, 0);
}, "this is null");

assert.throws(TypeError, function() {
  copyWithin.call(42, 0, 0);
}, "this is 42");

assert.throws(TypeError, function() {
  copyWithin.call("1", 0, 0);
}, "this is a string");

assert.throws(TypeError, function() {
  copyWithin.call(true, 0, 0);
}, "this is true");

assert.throws(TypeError, function() {
  copyWithin.call(false, 0, 0);
}, "this is false");

var s = Symbol("s");
assert.throws(TypeError, function() {
  copyWithin.call(s, 0, 0);
}, "this is a Symbol");
