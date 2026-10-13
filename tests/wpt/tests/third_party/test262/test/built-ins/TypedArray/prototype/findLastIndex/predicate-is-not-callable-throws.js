// Copyright (C) 2021 Microsoft. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.findlastindex
description: >
  Throws a TypeError exception if predicate is not callable.
info: |
  %TypedArray%.prototype.findLastIndex ( predicate [ , thisArg ] )

  ...
  4. If IsCallable(predicate) is false, throw a TypeError exception.
  ...
includes: [testTypedArray.js]
features: [TypedArray, array-find-from-last]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(0));
  assert.throws(TypeError, function() {
    sample.findLastIndex({});
  }, "{}");

  assert.throws(TypeError, function() {
    sample.findLastIndex(null);
  }, "null");

  assert.throws(TypeError, function() {
    sample.findLastIndex(undefined);
  }, "undefined");

  assert.throws(TypeError, function() {
    sample.findLastIndex(false);
  }, "false");

  assert.throws(TypeError, function() {
    sample.findLastIndex(true);
  }, "true");

  assert.throws(TypeError, function() {
    sample.findLastIndex(1);
  }, "1");

  assert.throws(TypeError, function() {
    sample.findLastIndex("");
  }, "string");

  assert.throws(TypeError, function() {
    sample.findLastIndex([]);
  }, "[]");

  assert.throws(TypeError, function() {
    sample.findLastIndex(/./);
  }, "/./");
});
