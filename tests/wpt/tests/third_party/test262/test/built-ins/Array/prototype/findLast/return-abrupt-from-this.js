// Copyright (C) 2021 Microsoft. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.findlast
description: >
  Return abrupt from ToObject(this value).
info: |
  Array.prototype.findLast ( predicate[ , thisArg ] )

  1. Let O be ? ToObject(this value).
features: [array-find-from-last]
---*/

// predicate fn is given to avoid false positives
assert.throws(TypeError, function() {
  Array.prototype.findLast.call(undefined, function() {});
});

assert.throws(TypeError, function() {
  Array.prototype.findLast.call(null, function() {});
});
