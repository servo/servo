// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.take
description: >
  Iterator.prototype.take throws TypeError when its this value is a non-object
info: |
  %Iterator.prototype%.take ( limit )

  7. Let iterated be ? GetIteratorDirect(this value).

features: [iterator-helpers]
flags: []
---*/
assert.throws(TypeError, function () {
  Iterator.prototype.take.call(null, 1);
});

assert.throws(TypeError, function () {
  Iterator.prototype.take.call(null, {
    valueOf: function () {
      throw new Test262Error();
    },
  });
});

Object.defineProperty(Number.prototype, 'next', {
  get: function () {
    throw new Test262Error();
  },
});
assert.throws(TypeError, function () {
  Iterator.prototype.take.call(0, 1);
});
