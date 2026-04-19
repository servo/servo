// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.every
description: Throws a TypeError if callbackfn is not callable
info: |
  22.2.3.7 %TypedArray%.prototype.every ( callbackfn [ , thisArg ] )

  %TypedArray%.prototype.every is a distinct function that implements the same
  algorithm as Array.prototype.every as defined in 22.1.3.5 except that the this
  object's [[ArrayLength]] internal slot is accessed in place of performing a
  [[Get]] of "length".

  22.1.3.5 Array.prototype.every ( callbackfn [ , thisArg ] )

  ...
  3. If IsCallable(callbackfn) is false, throw a TypeError exception.
  ...
includes: [testTypedArray.js]
features: [BigInt, Symbol, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(2));

  assert.throws(TypeError, function() {
    sample.every();
  }, "no args");

  assert.throws(TypeError, function() {
    sample.every(null);
  }, "null");

  assert.throws(TypeError, function() {
    sample.every(undefined);
  }, "undefined");

  assert.throws(TypeError, function() {
    sample.every("abc");
  }, "string");

  assert.throws(TypeError, function() {
    sample.every(1);
  }, "number");

  assert.throws(TypeError, function() {
    sample.every(NaN);
  }, "NaN");

  assert.throws(TypeError, function() {
    sample.every(false);
  }, "false");

  assert.throws(TypeError, function() {
    sample.every(true);
  }, "true");

  assert.throws(TypeError, function() {
    sample.every({});
  }, "{}");

  assert.throws(TypeError, function() {
    sample.every(sample);
  }, "same typedArray instance");

  assert.throws(TypeError, function() {
    sample.every(Symbol("1"));
  }, "symbol");
});
