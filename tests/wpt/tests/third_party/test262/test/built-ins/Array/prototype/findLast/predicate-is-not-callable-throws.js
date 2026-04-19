// Copyright (C) 2021 Microsoft. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.findlast
description: >
  Throws a TypeError exception if predicate is not callable.
info: |
  Array.prototype.findLast ( predicate[ , thisArg ] )

  ...
  3. If IsCallable(predicate) is false, throw a TypeError exception.
  ...
features: [array-find-from-last]
---*/

assert.throws(TypeError, function() {
  [].findLast({});
});

assert.throws(TypeError, function() {
  [].findLast(null);
});

assert.throws(TypeError, function() {
  [].findLast(undefined);
});

assert.throws(TypeError, function() {
  [].findLast(true);
});

assert.throws(TypeError, function() {
  [].findLast(1);
});

assert.throws(TypeError, function() {
  [].findLast('');
});

assert.throws(TypeError, function() {
  [].findLast(1);
});

assert.throws(TypeError, function() {
  [].findLast([]);
});

assert.throws(TypeError, function() {
  [].findLast(/./);
});
