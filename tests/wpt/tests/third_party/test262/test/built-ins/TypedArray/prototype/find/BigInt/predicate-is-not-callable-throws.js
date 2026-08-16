// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.find
description: >
  Throws a TypeError exception if predicate is not callable.
info: |
  22.2.3.10 %TypedArray%.prototype.find (predicate [ , thisArg ] )

  %TypedArray%.prototype.find is a distinct function that implements the same
  algorithm as Array.prototype.find as defined in 22.1.3.8 except that the this
  object's [[ArrayLength]] internal slot is accessed in place of performing a
  [[Get]] of "length". The implementation of the algorithm may be optimized with
  the knowledge that the this value is an object that has a fixed length and
  whose integer indexed properties are not sparse.

  ...

  22.1.3.8 Array.prototype.find ( predicate[ , thisArg ] )

  ...
  3. If IsCallable(predicate) is false, throw a TypeError exception.
  ...
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(0));

  assert.throws(TypeError, function() {
    sample.find({});
  }, "object");

  assert.throws(TypeError, function() {
    sample.find(null);
  }, "null");

  assert.throws(TypeError, function() {
    sample.find(undefined);
  }, "undefined");

  assert.throws(TypeError, function() {
    sample.find(false);
  }, "false");

  assert.throws(TypeError, function() {
    sample.find(true);
  }, "true");

  assert.throws(TypeError, function() {
    sample.find(1);
  }, "number");

  assert.throws(TypeError, function() {
    sample.find("");
  }, "string");

  assert.throws(TypeError, function() {
    sample.find([]);
  }, "array");

  assert.throws(TypeError, function() {
    sample.find(/./);
  }, "regexp");
});
