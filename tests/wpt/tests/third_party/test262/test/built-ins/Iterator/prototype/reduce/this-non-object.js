// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.reduce
description: >
  Iterator.prototype.reduce throws TypeError when its this value is a non-object
info: |
  %Iterator.prototype%.reduce ( reducer )

features: [iterator-helpers]
flags: []
---*/
assert.throws(TypeError, function () {
  Iterator.prototype.reduce.call(null, () => {});
});

assert.throws(TypeError, function () {
  Iterator.prototype.reduce.call(null, () => {}, 0);
});

Object.defineProperty(Number.prototype, 'next', {
  get: function () {
    throw new Test262Error();
  },
});

assert.throws(TypeError, function () {
  Iterator.prototype.reduce.call(0, () => {});
});

assert.throws(TypeError, function () {
  Iterator.prototype.reduce.call(0, () => {}, 0);
});
