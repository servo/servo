// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.toArray
description: >
  Iterator.prototype.toArray throws TypeError when its this value is a non-object
info: |
  %Iterator.prototype%.toArray ( )

features: [iterator-helpers]
flags: []
---*/
assert.throws(TypeError, function () {
  Iterator.prototype.toArray.call(null);
});

Object.defineProperty(Number.prototype, 'next', {
  get: function () {
    throw new Test262Error();
  },
});
assert.throws(TypeError, function () {
  Iterator.prototype.toArray.call(0);
});
