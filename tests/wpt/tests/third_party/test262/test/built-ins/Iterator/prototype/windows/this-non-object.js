// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.windows
description: >
  Iterator.prototype.windows throws TypeError when its this value is a non-object
info: |
  Iterator.prototype.windows ( windowSize [ , undersized ] )

  1. Let O be the this value.
  2. If O is not an Object, throw a TypeError exception.

features: [iterator-chunking]
---*/
assert.throws(TypeError, function () {
  Iterator.prototype.windows.call(null, 1);
});

Object.defineProperty(Number.prototype, 'next', {
  get: function () {
    throw new Test262Error();
  }
});
assert.throws(TypeError, function () {
  Iterator.prototype.windows.call(0, 1);
});
