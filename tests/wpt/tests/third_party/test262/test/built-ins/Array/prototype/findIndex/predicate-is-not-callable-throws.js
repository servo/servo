// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.findindex
description: >
  Throws a TypeError exception if predicate is not callable.
info: |
  22.1.3.9 Array.prototype.findIndex ( predicate[ , thisArg ] )

  ...
  5. If IsCallable(predicate) is false, throw a TypeError exception.
  ...
---*/

assert.throws(TypeError, function() {
  [].findIndex({});
});

assert.throws(TypeError, function() {
  [].findIndex(null);
});

assert.throws(TypeError, function() {
  [].findIndex(undefined);
});

assert.throws(TypeError, function() {
  [].findIndex(true);
});

assert.throws(TypeError, function() {
  [].findIndex(1);
});

assert.throws(TypeError, function() {
  [].findIndex('');
});

assert.throws(TypeError, function() {
  [].findIndex(1);
});

assert.throws(TypeError, function() {
  [].findIndex([]);
});

assert.throws(TypeError, function() {
  [].findIndex(/./);
});
