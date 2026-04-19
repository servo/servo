// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.findindex
description: >
  Throws a TypeError exception if predicate is not callable.
info: |
  22.2.3.11 %TypedArray%.prototype.findIndex ( predicate [ , thisArg ] )

  %TypedArray%.prototype.findIndex is a distinct function that implements the
  same algorithm as Array.prototype.findIndex as defined in 22.1.3.9 except that
  the this object's [[ArrayLength]] internal slot is accessed in place of
  performing a [[Get]] of "length".

  ...

  22.1.3.9 Array.prototype.findIndex ( predicate[ , thisArg ] )

  ...
  3. If IsCallable(predicate) is false, throw a TypeError exception.
  ...
includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA) {
  var sample = new TA();
  assert.throws(TypeError, function() {
    sample.findIndex({});
  }, "{}");

  assert.throws(TypeError, function() {
    sample.findIndex(null);
  }, "null");

  assert.throws(TypeError, function() {
    sample.findIndex(undefined);
  }, "undefined");

  assert.throws(TypeError, function() {
    sample.findIndex(false);
  }, "false");

  assert.throws(TypeError, function() {
    sample.findIndex(true);
  }, "true");

  assert.throws(TypeError, function() {
    sample.findIndex(1);
  }, "1");

  assert.throws(TypeError, function() {
    sample.findIndex("");
  }, "string");

  assert.throws(TypeError, function() {
    sample.findIndex([]);
  }, "[]");

  assert.throws(TypeError, function() {
    sample.findIndex(/./);
  }, "/./");
}, null, ["passthrough"]);

