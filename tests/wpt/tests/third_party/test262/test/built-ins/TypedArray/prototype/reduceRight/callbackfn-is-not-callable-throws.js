// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.reduceright
description: >
  Throws TypeError if callbackfn is not callable
info: |
  22.2.3.21 %TypedArray%.prototype.reduceRight ( callbackfn [ , initialValue ] )

  %TypedArray%.prototype.reduceRight is a distinct function that implements the
  same algorithm as Array.prototype.reduceRight as defined in 22.1.3.20 except
  that the this object's [[ArrayLength]] internal slot is accessed in place of
  performing a [[Get]] of "length".

  22.1.3.20 Array.prototype.reduceRight ( callbackfn [ , initialValue ] )

  ...
  3. If IsCallable(callbackfn) is false, throw a TypeError exception.
  4. If len is 0 and initialValue is not present, throw a TypeError exception.
  ...
includes: [testTypedArray.js]
features: [Symbol, TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(1));

  assert.throws(TypeError, function() {
    sample.reduceRight();
  }, "no arg");

  assert.throws(TypeError, function() {
    sample.reduceRight(undefined);
  }, "undefined");

  assert.throws(TypeError, function() {
    sample.reduceRight(null);
  }, "null");

  assert.throws(TypeError, function() {
    sample.reduceRight({});
  }, "{}");

  assert.throws(TypeError, function() {
    sample.reduceRight(1);
  }, "1");

  assert.throws(TypeError, function() {
    sample.reduceRight(NaN);
  }, "NaN");

  assert.throws(TypeError, function() {
    sample.reduceRight("");
  }, "string");

  assert.throws(TypeError, function() {
    sample.reduceRight(false);
  }, "false");

  assert.throws(TypeError, function() {
    sample.reduceRight(true);
  }, "true");

  assert.throws(TypeError, function() {
    sample.reduceRight(Symbol(""));
  }, "symbol");
});
