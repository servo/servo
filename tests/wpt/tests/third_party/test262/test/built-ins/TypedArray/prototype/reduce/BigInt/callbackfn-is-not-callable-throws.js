// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.reduce
description: >
  Throws TypeError if callbackfn is not callable
info: |
  22.2.3.20 %TypedArray%.prototype.reduce ( callbackfn [ , initialValue ] )

  %TypedArray%.prototype.reduce is a distinct function that implements the same
  algorithm as Array.prototype.reduce as defined in 22.1.3.19 except that the
  this object's [[ArrayLength]] internal slot is accessed in place of performing
  a [[Get]] of "length".

  22.1.3.19 Array.prototype.reduce ( callbackfn [ , initialValue ] )

  ...
  3. If IsCallable(callbackfn) is false, throw a TypeError exception.
  4. If len is 0 and initialValue is not present, throw a TypeError exception.
  ...
includes: [testTypedArray.js]
features: [BigInt, Symbol, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(1));

  assert.throws(TypeError, function() {
    sample.reduce();
  }, "no arg");

  assert.throws(TypeError, function() {
    sample.reduce(undefined);
  }, "undefined");

  assert.throws(TypeError, function() {
    sample.reduce(null);
  }, "null");

  assert.throws(TypeError, function() {
    sample.reduce({});
  }, "{}");

  assert.throws(TypeError, function() {
    sample.reduce(1);
  }, "1");

  assert.throws(TypeError, function() {
    sample.reduce(NaN);
  }, "NaN");

  assert.throws(TypeError, function() {
    sample.reduce("");
  }, "string");

  assert.throws(TypeError, function() {
    sample.reduce(false);
  }, "false");

  assert.throws(TypeError, function() {
    sample.reduce(true);
  }, "true");

  assert.throws(TypeError, function() {
    sample.reduce(Symbol(""));
  }, "symbol");
});
