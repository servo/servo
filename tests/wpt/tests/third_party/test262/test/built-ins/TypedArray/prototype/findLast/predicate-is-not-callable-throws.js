// Copyright (C) 2021 Microsoft. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.findlast
description: >
  Throws a TypeError exception if predicate is not callable.
info: |
  %TypedArray%.prototype.findLast (predicate [ , thisArg ] )

  ...
  4. If IsCallable(predicate) is false, throw a TypeError exception.
  ...
includes: [testTypedArray.js]
features: [TypedArray, array-find-from-last]
---*/

testWithTypedArrayConstructors(function(TA) {
  var sample = new TA();

  assert.throws(TypeError, function() {
    sample.findLast({});
  }, "object");

  assert.throws(TypeError, function() {
    sample.findLast(null);
  }, "null");

  assert.throws(TypeError, function() {
    sample.findLast(undefined);
  }, "undefined");

  assert.throws(TypeError, function() {
    sample.findLast(false);
  }, "false");

  assert.throws(TypeError, function() {
    sample.findLast(true);
  }, "true");

  assert.throws(TypeError, function() {
    sample.findLast(1);
  }, "number");

  assert.throws(TypeError, function() {
    sample.findLast("");
  }, "string");

  assert.throws(TypeError, function() {
    sample.findLast([]);
  }, "array");

  assert.throws(TypeError, function() {
    sample.findLast(/./);
  }, "regexp");
}, null, ["passthrough"]);
