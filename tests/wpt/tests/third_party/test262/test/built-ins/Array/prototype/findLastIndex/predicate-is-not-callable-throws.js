// Copyright (C) 2021 Microsoft. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.findlastindex
description: >
  Throws a TypeError exception if predicate is not callable.
info: |
  Array.prototype.findLastIndex ( predicate[ , thisArg ] )

  ...
  3. If IsCallable(predicate) is false, throw a TypeError exception.
  ...
features: [array-find-from-last]
---*/

assert.throws(TypeError, function() {
  [].findLastIndex({});
});

assert.throws(TypeError, function() {
  [].findLastIndex(null);
});

assert.throws(TypeError, function() {
  [].findLastIndex(undefined);
});

assert.throws(TypeError, function() {
  [].findLastIndex(true);
});

assert.throws(TypeError, function() {
  [].findLastIndex(1);
});

assert.throws(TypeError, function() {
  [].findLastIndex('');
});

assert.throws(TypeError, function() {
  [].findLastIndex(1);
});

assert.throws(TypeError, function() {
  [].findLastIndex([]);
});

assert.throws(TypeError, function() {
  [].findLastIndex(/./);
});
