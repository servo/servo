// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.find
description: >
  Throws a TypeError exception if predicate is not callable.
info: |
  22.1.3.8 Array.prototype.find ( predicate[ , thisArg ] )

  ...
  5. If IsCallable(predicate) is false, throw a TypeError exception.
  ...
---*/

assert.throws(TypeError, function() {
  [].find({});
});

assert.throws(TypeError, function() {
  [].find(null);
});

assert.throws(TypeError, function() {
  [].find(undefined);
});

assert.throws(TypeError, function() {
  [].find(true);
});

assert.throws(TypeError, function() {
  [].find(1);
});

assert.throws(TypeError, function() {
  [].find('');
});

assert.throws(TypeError, function() {
  [].find(1);
});

assert.throws(TypeError, function() {
  [].find([]);
});

assert.throws(TypeError, function() {
  [].find(/./);
});
