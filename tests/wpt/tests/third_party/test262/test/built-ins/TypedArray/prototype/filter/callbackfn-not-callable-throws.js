// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.filter
description: Throws TypeError if callbackfn is not callable
info: |
  22.2.3.9 %TypedArray%.prototype.filter ( callbackfn [ , thisArg ] )

  ...
  4. If IsCallable(callbackfn) is false, throw a TypeError exception.
  ...
includes: [testTypedArray.js]
features: [Symbol, TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(4));

  assert.throws(TypeError, function() {
    sample.filter();
  }, "no arg");

  assert.throws(TypeError, function() {
    sample.filter(undefined);
  }, "undefined");

  assert.throws(TypeError, function() {
    sample.filter(null);
  }, "null");

  assert.throws(TypeError, function() {
    sample.filter(true);
  }, "true");

  assert.throws(TypeError, function() {
    sample.filter(false);
  }, "false");

  assert.throws(TypeError, function() {
    sample.filter({});
  }, "{}");

  assert.throws(TypeError, function() {
    sample.filter([]);
  }, "[]");

  assert.throws(TypeError, function() {
    sample.filter(1);
  }, "Number");

  assert.throws(TypeError, function() {
    sample.filter(Symbol(""));
  }, "symbol");

  assert.throws(TypeError, function() {
    sample.filter("");
  }, "string");
});
